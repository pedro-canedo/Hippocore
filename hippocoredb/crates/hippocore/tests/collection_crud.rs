//! Collection lifecycle regression tests.
//!
//! Collections scope all items within a tenant. These tests verify: creation
//! appears in the list, duplicate names within a tenant are rejected, tenants
//! can share collection names without interference, collection lists are
//! tenant-isolated, and recall scoped to a non-existent collection returns
//! empty (not an error).

use hippocore::model::MemoryType;
use hippocore::{Config, Hippocore, RecallRequest, RememberRequest};
use tempfile::TempDir;

fn open(dir: &TempDir) -> Hippocore {
    Hippocore::open(Config::new(dir.path())).expect("open")
}

// ── Created collection appears in list ───────────────────────────────────────

#[test]
fn created_collection_appears_in_list() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    db.create_tenant("acme", "Acme").unwrap();

    db.create_collection("acme", "support", "Support knowledge")
        .unwrap();

    let cols = db.collections("acme");
    assert!(
        cols.iter().any(|c| c.name == "support"),
        "collection 'support' must appear after creation"
    );
}

// ── Duplicate create_collection is idempotent ────────────────────────────────
//
// create_collection returns the existing collection rather than an error so
// callers can safely call it as an upsert without pre-checking existence.

#[test]
fn duplicate_collection_is_idempotent() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    db.create_tenant("acme", "Acme").unwrap();

    db.create_collection("acme", "support", "First").unwrap();
    let second = db.create_collection("acme", "support", "Second");

    assert!(
        second.is_ok(),
        "duplicate create_collection must succeed (idempotent)"
    );
    // Only one collection exists — idempotent, not a duplicate.
    let cols = db.collections("acme");
    assert_eq!(
        cols.iter().filter(|c| c.name == "support").count(),
        1,
        "exactly one 'support' collection must exist after two creates"
    );
}

// ── Two tenants can share a collection name ───────────────────────────────────

#[test]
fn tenants_can_share_collection_names() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    db.create_tenant("alpha", "Alpha").unwrap();
    db.create_tenant("beta", "Beta").unwrap();

    db.create_collection("alpha", "shared", "").unwrap();
    db.create_collection("beta", "shared", "").unwrap();

    assert!(db.collections("alpha").iter().any(|c| c.name == "shared"));
    assert!(db.collections("beta").iter().any(|c| c.name == "shared"));
}

// ── Collection list is tenant-isolated ───────────────────────────────────────

#[test]
fn collection_list_is_tenant_isolated() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    db.create_tenant("alpha", "Alpha").unwrap();
    db.create_tenant("beta", "Beta").unwrap();

    db.create_collection("alpha", "alpha-only", "").unwrap();
    db.create_collection("beta", "beta-only", "").unwrap();

    let alpha_cols = db.collections("alpha");
    let beta_cols = db.collections("beta");

    assert!(
        !alpha_cols.iter().any(|c| c.name == "beta-only"),
        "alpha's collection list must not contain beta's collections"
    );
    assert!(
        !beta_cols.iter().any(|c| c.name == "alpha-only"),
        "beta's collection list must not contain alpha's collections"
    );
}

// ── Recall scoped to nonexistent collection returns empty ─────────────────────

#[test]
fn recall_scoped_to_nonexistent_collection_returns_empty() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    db.create_tenant("acme", "Acme").unwrap();
    db.create_collection("acme", "support", "").unwrap();

    db.remember(RememberRequest::new(
        "acme",
        "support",
        MemoryType::Semantic,
        "postgresql connection pool configuration",
    ))
    .unwrap();

    let mut req = RecallRequest::new("acme", "postgresql connection pool");
    req.collection = Some("nonexistent-collection".into());
    let hits = db.recall(req).unwrap();

    assert!(
        hits.is_empty(),
        "recall scoped to nonexistent collection must return empty, got {}",
        hits.len()
    );
}

// ── Collection description is stored and returned ─────────────────────────────

#[test]
fn collection_description_is_stored() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    db.create_tenant("acme", "Acme").unwrap();

    db.create_collection("acme", "support", "Customer support knowledge base")
        .unwrap();

    let col = db
        .collections("acme")
        .into_iter()
        .find(|c| c.name == "support")
        .expect("support collection must exist");

    assert_eq!(
        col.description, "Customer support knowledge base",
        "stored description must round-trip"
    );
}
