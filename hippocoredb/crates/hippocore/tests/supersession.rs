//! Supersession lifecycle regression tests.
//!
//! Verifies that superseded memories are hidden from default recall, visible
//! with `include_superseded = true`, and that the supersedes/superseded_by
//! linkage is bidirectional, symmetric, and durable across WAL compaction.

use hippocore::model::MemoryType;
use hippocore::{Config, Hippocore, RecallRequest, RememberRequest};
use tempfile::TempDir;

fn open_seeded(dir: &TempDir) -> Hippocore {
    let mut db = Hippocore::open(Config::new(dir.path())).expect("open");
    db.create_tenant("acme", "Acme").unwrap();
    db.create_collection("acme", "support", "").unwrap();
    db
}

/// Store memory A, then memory B that supersedes A. Return (id_a, id_b).
fn seed_supersession(db: &mut Hippocore) -> (String, String) {
    let id_a = db
        .remember(RememberRequest::new(
            "acme",
            "support",
            MemoryType::Semantic,
            "postgresql connection pool configuration v1",
        ))
        .unwrap()
        .id;

    let id_b = db
        .remember(RememberRequest {
            id: None,
            supersedes: vec![id_a.clone()],
            ..RememberRequest::new(
                "acme",
                "support",
                MemoryType::Semantic,
                "postgresql connection pool configuration v2 updated",
            )
        })
        .unwrap()
        .id;

    (id_a, id_b)
}

// ── Default recall excludes superseded ───────────────────────────────────────

#[test]
fn superseded_memory_hidden_from_default_recall() {
    let dir = TempDir::new().unwrap();
    let mut db = open_seeded(&dir);
    let (id_a, id_b) = seed_supersession(&mut db);

    let hits = db
        .recall(RecallRequest::new(
            "acme",
            "postgresql connection pool configuration",
        ))
        .unwrap();

    let ids: Vec<&str> = hits.iter().map(|h| h.id.as_str()).collect();
    assert!(
        ids.contains(&id_b.as_str()),
        "superseding memory B should appear in default recall"
    );
    assert!(
        !ids.contains(&id_a.as_str()),
        "superseded memory A must NOT appear in default recall"
    );
}

// ── include_superseded = true shows full history ──────────────────────────────

#[test]
fn superseded_memory_visible_with_include_superseded() {
    let dir = TempDir::new().unwrap();
    let mut db = open_seeded(&dir);
    let (id_a, id_b) = seed_supersession(&mut db);

    let mut req = RecallRequest::new("acme", "postgresql connection pool configuration");
    req.include_superseded = true;
    let hits = db.recall(req).unwrap();

    let ids: Vec<&str> = hits.iter().map(|h| h.id.as_str()).collect();
    assert!(
        ids.contains(&id_a.as_str()),
        "superseded memory A should appear when include_superseded=true"
    );
    assert!(
        ids.contains(&id_b.as_str()),
        "superseding memory B should also appear when include_superseded=true"
    );
}

// ── Bidirectional linkage ─────────────────────────────────────────────────────

#[test]
fn supersession_linkage_is_bidirectional() {
    let dir = TempDir::new().unwrap();
    let mut db = open_seeded(&dir);
    let (id_a, id_b) = seed_supersession(&mut db);

    let mem_a = db
        .get_memory("acme", "support", &id_a)
        .expect("A must exist");
    let mem_b = db
        .get_memory("acme", "support", &id_b)
        .expect("B must exist");

    assert_eq!(
        mem_a.superseded_by.as_deref(),
        Some(id_b.as_str()),
        "A.superseded_by must point to B"
    );
    assert!(
        mem_b.supersedes.contains(&id_a),
        "B.supersedes must contain A's id"
    );
}

// ── Durability across WAL compaction ─────────────────────────────────────────

#[test]
fn supersession_survives_wal_compaction() {
    let dir = TempDir::new().unwrap();
    let mut db = open_seeded(&dir);
    let (id_a, id_b) = seed_supersession(&mut db);

    // Force a WAL compaction.
    db.compact().unwrap();

    // Drop the handle and re-open (cold recovery from snapshot + WAL).
    drop(db);
    let db = Hippocore::open(Config::new(dir.path())).expect("reopen");

    // Default recall must still exclude superseded A.
    let hits = db
        .recall(RecallRequest::new(
            "acme",
            "postgresql connection pool configuration",
        ))
        .unwrap();
    let ids: Vec<&str> = hits.iter().map(|h| h.id.as_str()).collect();
    assert!(
        !ids.contains(&id_a.as_str()),
        "superseded A must still be hidden after compaction + reopen"
    );
    assert!(
        ids.contains(&id_b.as_str()),
        "superseding B must still be visible after compaction + reopen"
    );

    // Linkage must also survive.
    let mem_a = db
        .get_memory("acme", "support", &id_a)
        .expect("A must persist after compaction");
    assert_eq!(
        mem_a.superseded_by.as_deref(),
        Some(id_b.as_str()),
        "A.superseded_by must survive compaction"
    );
}

// ── Superseded_by is a durable tombstone ────────────────────────────────────
//
// Forgetting the superseding memory (B) does NOT un-supersede the original (A).
// The superseded_by field on A persists as a durable history marker so the
// supersession event is not silently erased when the superseding memory is
// later deleted.

#[test]
fn forgetting_superseding_memory_does_not_restore_superseded() {
    let dir = TempDir::new().unwrap();
    let mut db = open_seeded(&dir);
    let (id_a, id_b) = seed_supersession(&mut db);

    // Forget B (the superseding memory).
    db.forget("acme", "support", &id_b).unwrap();

    // A's superseded_by field is still set (durable tombstone).
    let mem_a = db
        .get_memory("acme", "support", &id_a)
        .expect("A must still exist");
    assert_eq!(
        mem_a.superseded_by.as_deref(),
        Some(id_b.as_str()),
        "A.superseded_by must persist as a tombstone even after B is deleted"
    );

    // A remains hidden from default recall because superseded_by is non-None.
    let hits = db
        .recall(RecallRequest::new(
            "acme",
            "postgresql connection pool configuration",
        ))
        .unwrap();
    assert!(
        !hits.iter().any(|h| h.id == id_a),
        "superseded A must stay hidden even after superseding B is deleted"
    );
}
