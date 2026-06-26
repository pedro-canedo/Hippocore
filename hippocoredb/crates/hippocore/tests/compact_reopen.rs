//! Compact + cold-reopen invariant regression tests.
//!
//! `compact()` folds all WAL entries into a snapshot and resets the WAL to
//! zero. If compaction silently lost data or left the index incomplete, recall
//! and retrieval after a cold reopen would degrade without any test catching
//! it. These tests lock the invariant.

use hippocore::model::MemoryType;
use hippocore::{Config, Hippocore, RecallRequest, RememberRequest, StoreDocumentRequest};
use tempfile::TempDir;

fn open(dir: &TempDir) -> Hippocore {
    Hippocore::open(Config::new(dir.path())).expect("open")
}

fn seed(db: &mut Hippocore) {
    db.create_tenant("acme", "Acme").unwrap();
    db.create_collection("acme", "kb", "").unwrap();
}

// ── WAL length is zero after compact ─────────────────────────────────────────

#[test]
fn wal_length_is_zero_after_compact() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    seed(&mut db);
    db.remember(RememberRequest::new(
        "acme",
        "kb",
        MemoryType::Semantic,
        "gradient descent converges",
    ))
    .unwrap();

    assert!(
        db.stats().unwrap().wal_entries > 0,
        "WAL must have entries before compact"
    );

    db.compact().unwrap();

    assert_eq!(
        db.stats().unwrap().wal_entries,
        0,
        "WAL must be empty immediately after compact"
    );
}

// ── Collections survive compact + cold reopen ─────────────────────────────────

#[test]
fn collections_survive_compact_reopen() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    seed(&mut db);
    db.create_collection("acme", "notes", "").unwrap();
    db.compact().unwrap();
    drop(db);

    let db2 = open(&dir);
    let cols = db2.collections("acme");
    assert!(cols.iter().any(|c| c.name == "kb"), "kb must survive");
    assert!(cols.iter().any(|c| c.name == "notes"), "notes must survive");
}

// ── Memories survive compact + cold reopen ────────────────────────────────────

#[test]
fn memories_survive_compact_reopen() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    seed(&mut db);

    let id = db
        .remember(RememberRequest::new(
            "acme",
            "kb",
            MemoryType::Semantic,
            "transformer attention mechanism",
        ))
        .unwrap()
        .id;

    db.compact().unwrap();
    drop(db);

    let db2 = open(&dir);
    assert!(
        db2.get_memory("acme", "kb", &id).is_some(),
        "memory must be retrievable by id after compact + reopen"
    );

    let hits = db2
        .recall(RecallRequest::new("acme", "transformer attention"))
        .unwrap();
    assert!(
        hits.iter().any(|h| h.id == id),
        "memory must appear in recall after compact + reopen"
    );
}

// ── Documents survive compact + cold reopen ───────────────────────────────────

#[test]
fn documents_survive_compact_reopen() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    seed(&mut db);

    let doc = db
        .store_document(StoreDocumentRequest {
            tenant_id: "acme".into(),
            collection: "kb".into(),
            id: Some("doc-1".into()),
            text: "backpropagation through time for RNNs".into(),
            content_type: None,
            raw: None,
            metadata: Default::default(),
            source: None,
            chunks: None,
            valid_from: None,
            valid_until: None,
        })
        .unwrap();

    db.compact().unwrap();
    drop(db);

    let db2 = open(&dir);
    assert!(
        db2.get_document("acme", "kb", &doc.id).is_some(),
        "document must be retrievable after compact + reopen"
    );
}

// ── Multiple compact cycles do not lose data ──────────────────────────────────

#[test]
fn multiple_compact_cycles_preserve_state() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    seed(&mut db);

    let mut ids = Vec::new();
    for i in 0..3u8 {
        let id = db
            .remember(RememberRequest::new(
                "acme",
                "kb",
                MemoryType::Semantic,
                format!("unique fact number {i}"),
            ))
            .unwrap()
            .id;
        ids.push(id);
        db.compact().unwrap();
    }
    drop(db);

    let db2 = open(&dir);
    for id in &ids {
        assert!(
            db2.get_memory("acme", "kb", id).is_some(),
            "memory {id} must survive multiple compact cycles"
        );
    }
}
