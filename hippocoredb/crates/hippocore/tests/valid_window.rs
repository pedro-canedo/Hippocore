//! Valid-window recall regression tests.
//!
//! Memories with `valid_from` / `valid_until` timestamps must be excluded from
//! recall when the query time falls outside the window and included when it falls
//! inside. The `as_of` parameter in RecallRequest allows backdating the query
//! to a specific epoch-ms timestamp.

use hippocore::model::MemoryType;
use hippocore::{Config, Hippocore, RecallRequest, RememberRequest};
use tempfile::TempDir;

const FAR_PAST: i64 = 1_000_000_000_000; // 2001-09-08, safely in the past
const FAR_FUTURE: i64 = 9_000_000_000_000; // ~year 2255, safely in the future

fn open_seeded(dir: &TempDir) -> Hippocore {
    let mut db = Hippocore::open(Config::new(dir.path())).expect("open");
    db.create_tenant("acme", "Acme").unwrap();
    db.create_collection("acme", "support", "").unwrap();
    db
}

// ── Expired memory excluded from recall ──────────────────────────────────────

#[test]
fn expired_memory_excluded_from_recall() {
    let dir = TempDir::new().unwrap();
    let mut db = open_seeded(&dir);

    let id = db
        .remember(RememberRequest {
            valid_until: Some(FAR_PAST), // already expired
            ..RememberRequest::new(
                "acme",
                "support",
                MemoryType::Semantic,
                "postgresql connection pool configuration expired",
            )
        })
        .unwrap()
        .id;

    let hits = db
        .recall(RecallRequest::new("acme", "postgresql connection pool"))
        .unwrap();
    assert!(
        !hits.iter().any(|h| h.id == id),
        "expired memory must not appear in recall"
    );
}

// ── Future-valid memory included in recall ────────────────────────────────────

#[test]
fn future_valid_memory_included_in_recall() {
    let dir = TempDir::new().unwrap();
    let mut db = open_seeded(&dir);

    let id = db
        .remember(RememberRequest {
            valid_until: Some(FAR_FUTURE), // expires far in the future
            ..RememberRequest::new(
                "acme",
                "support",
                MemoryType::Semantic,
                "postgresql connection pool valid future",
            )
        })
        .unwrap()
        .id;

    let hits = db
        .recall(RecallRequest::new("acme", "postgresql connection pool"))
        .unwrap();
    assert!(
        hits.iter().any(|h| h.id == id),
        "memory with future valid_until must appear in recall"
    );
}

// ── Future-valid_from memory excluded until that time ────────────────────────

#[test]
fn not_yet_valid_memory_excluded_from_recall() {
    let dir = TempDir::new().unwrap();
    let mut db = open_seeded(&dir);

    let id = db
        .remember(RememberRequest {
            valid_from: Some(FAR_FUTURE), // not valid yet
            ..RememberRequest::new(
                "acme",
                "support",
                MemoryType::Semantic,
                "postgresql connection pool not yet valid",
            )
        })
        .unwrap()
        .id;

    let hits = db
        .recall(RecallRequest::new("acme", "postgresql connection pool"))
        .unwrap();
    assert!(
        !hits.iter().any(|h| h.id == id),
        "memory with future valid_from must not appear in recall"
    );
}

// ── Backdated as_of query retrieves past-valid memory ────────────────────────

#[test]
fn as_of_backdating_retrieves_past_valid_memory() {
    let dir = TempDir::new().unwrap();
    let mut db = open_seeded(&dir);

    // Memory valid between FAR_PAST and (FAR_PAST + 1s) — long expired now.
    let id = db
        .remember(RememberRequest {
            valid_from: Some(FAR_PAST),
            valid_until: Some(FAR_PAST + 1_000), // 1 second window
            ..RememberRequest::new(
                "acme",
                "support",
                MemoryType::Semantic,
                "postgresql connection pool historical state",
            )
        })
        .unwrap()
        .id;

    // Current-time query → memory expired, not in results.
    let hits_now = db
        .recall(RecallRequest::new("acme", "postgresql connection pool"))
        .unwrap();
    assert!(
        !hits_now.iter().any(|h| h.id == id),
        "expired memory must not appear in current-time recall"
    );

    // Backdated query to inside the validity window → memory should appear.
    let mut req_past = RecallRequest::new("acme", "postgresql connection pool");
    req_past.as_of = Some(FAR_PAST + 500); // inside [FAR_PAST, FAR_PAST+1000)
    let hits_past = db.recall(req_past).unwrap();
    assert!(
        hits_past.iter().any(|h| h.id == id),
        "memory must appear when as_of falls inside validity window"
    );
}

// ── No validity window → always included ─────────────────────────────────────

#[test]
fn memory_with_no_validity_window_always_included() {
    let dir = TempDir::new().unwrap();
    let mut db = open_seeded(&dir);

    let id = db
        .remember(RememberRequest::new(
            "acme",
            "support",
            MemoryType::Semantic,
            "postgresql connection pool no validity window",
        ))
        .unwrap()
        .id;

    let hits = db
        .recall(RecallRequest::new("acme", "postgresql connection pool"))
        .unwrap();
    assert!(
        hits.iter().any(|h| h.id == id),
        "memory with no validity window must always appear in recall"
    );
}
