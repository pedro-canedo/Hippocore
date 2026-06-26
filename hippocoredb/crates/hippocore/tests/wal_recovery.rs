//! WAL torn-write and corruption recovery regression tests.
//!
//! The WAL format is `<crc32-hex>\t<json>\n`. On open, `Storage::replay_wal`
//! stops at the first truncated, unparseable, or checksum-mismatched line —
//! it never panics. These tests lock that invariant.

use hippocore::model::MemoryType;
use hippocore::{Config, Hippocore, RecallRequest, RememberRequest};
use std::fs::OpenOptions;
use std::io::Write;
use tempfile::TempDir;

fn open(dir: &TempDir) -> Hippocore {
    Hippocore::open(Config::new(dir.path())).expect("open")
}

fn wal_path(dir: &TempDir) -> std::path::PathBuf {
    dir.path().join("wal.log")
}

// ── Clean WAL opens successfully ──────────────────────────────────────────────

#[test]
fn clean_wal_opens_successfully() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    db.create_tenant("acme", "Acme").unwrap();
    db.create_collection("acme", "kb", "").unwrap();
    drop(db);

    // Cold reopen succeeds.
    let db2 = open(&dir);
    assert!(
        !db2.collections("acme").is_empty(),
        "acme tenant must survive cold reopen"
    );
}

// ── Truncated WAL line is skipped, open succeeds ──────────────────────────────

#[test]
fn truncated_wal_trailing_line_skipped_on_open() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    db.create_tenant("acme", "Acme").unwrap();
    db.create_collection("acme", "kb", "").unwrap();
    db.remember(RememberRequest::new(
        "acme",
        "kb",
        MemoryType::Semantic,
        "before the crash",
    ))
    .unwrap();
    drop(db);

    // Append a half-written entry (no newline terminator — simulates power loss
    // mid-write).
    let mut f = OpenOptions::new()
        .append(true)
        .open(wal_path(&dir))
        .unwrap();
    f.write_all(b"deadbeef\t{\"CreateTenant\":{\"id\":\"trunc\"")
        .unwrap();
    f.flush().unwrap();
    drop(f);

    // Must open without error and see the entry written before the torn write.
    let db2 = open(&dir);
    let hits = db2
        .recall(RecallRequest::new("acme", "before the crash"))
        .unwrap();
    assert!(
        !hits.is_empty(),
        "data written before the torn WAL entry must survive recovery"
    );
}

// ── Corrupt (checksum-mismatch) WAL entry stops replay safely ────────────────

#[test]
fn checksum_mismatch_stops_replay_does_not_panic() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    db.create_tenant("acme", "Acme").unwrap();
    db.create_collection("acme", "kb", "").unwrap();
    drop(db);

    // Append a line with a valid CRC format but wrong checksum for the payload.
    let mut f = OpenOptions::new()
        .append(true)
        .open(wal_path(&dir))
        .unwrap();
    writeln!(
        f,
        "00000000\t{{\"CreateTenant\":{{\"id\":\"corrupt\",\"display_name\":\"C\"}}}}"
    )
    .unwrap();
    drop(f);

    // Must open without panicking.
    let db2 = open(&dir);
    // The corrupt entry must not have been applied.
    let tenants: Vec<_> = db2.all_collections();
    assert!(
        !tenants.iter().any(|c| c.tenant_id == "corrupt"),
        "corrupt WAL entry must not be applied"
    );
}

// ── State preceding the torn entry is fully intact after recovery ─────────────

#[test]
fn state_before_torn_entry_is_intact() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    db.create_tenant("acme", "Acme").unwrap();
    db.create_collection("acme", "notes", "").unwrap();

    let id = db
        .remember(RememberRequest::new(
            "acme",
            "notes",
            MemoryType::Semantic,
            "neural network regularization techniques",
        ))
        .unwrap()
        .id;
    drop(db);

    // Append a partially-written entry.
    let mut f = OpenOptions::new()
        .append(true)
        .open(wal_path(&dir))
        .unwrap();
    f.write_all(b"feedcafe\t{\"partial").unwrap();
    drop(f);

    let db2 = open(&dir);
    let mem = db2.get_memory("acme", "notes", &id);
    assert!(
        mem.is_some(),
        "memory written before the torn entry must be retrievable by id"
    );

    let hits = db2
        .recall(RecallRequest::new("acme", "neural network"))
        .unwrap();
    assert!(
        hits.iter().any(|h| h.id == id),
        "memory must appear in recall after recovery"
    );
}
