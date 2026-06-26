//! Record entity lifecycle regression tests.
//!
//! Records are first-class entities with versioning, deletion, and tenant
//! isolation. These tests verify the full CRUD contract without requiring
//! production code changes.

use hippocore::{Config, Hippocore, PutRecordRequest, RecallRequest};
use serde_json::json;
use tempfile::TempDir;

fn open(dir: &TempDir) -> Hippocore {
    Hippocore::open(Config::new(dir.path())).expect("open")
}

fn seed(db: &mut Hippocore) {
    db.create_tenant("acme", "Acme").unwrap();
    db.create_collection("acme", "data", "").unwrap();
}

// ── Store and retrieve by id ──────────────────────────────────────────────────

#[test]
fn stored_record_is_retrievable_by_id() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    seed(&mut db);

    let rec = db
        .put_record(PutRecordRequest::new(
            "acme",
            "data",
            "users",
            json!({"name": "Alice", "role": "admin"}),
        ))
        .unwrap();

    let found = db.get_record("acme", "data", "users", &rec.id);
    assert!(found.is_some(), "record must be retrievable by id");
    assert_eq!(found.unwrap().id, rec.id);
}

// ── Record appears in recall ──────────────────────────────────────────────────

#[test]
fn record_appears_in_recall() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    seed(&mut db);

    let rec = db
        .put_record(PutRecordRequest::new(
            "acme",
            "data",
            "products",
            json!({"name": "quantum flux capacitor"}),
        ))
        .unwrap();

    let hits = db
        .recall(RecallRequest::new("acme", "quantum flux capacitor"))
        .unwrap();
    assert!(
        hits.iter().any(|h| h.id == rec.id),
        "stored record must appear in recall"
    );
}

// ── Update increments version ─────────────────────────────────────────────────

#[test]
fn update_increments_version() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    seed(&mut db);

    let rec = db
        .put_record(PutRecordRequest::new(
            "acme",
            "data",
            "settings",
            json!({"theme": "dark"}),
        ))
        .unwrap();
    assert_eq!(rec.version, 0, "initial version must be 0");

    let mut update_req =
        PutRecordRequest::new("acme", "data", "settings", json!({"theme": "light"}));
    update_req.id = Some(rec.id.clone());
    let updated = db.put_record(update_req).unwrap();

    assert_eq!(updated.version, 1, "version must be 1 after first update");
    assert_eq!(updated.id, rec.id, "id must be unchanged after update");
}

// ── Delete removes the record ─────────────────────────────────────────────────

#[test]
fn delete_record_returns_none_on_get() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    seed(&mut db);

    let rec = db
        .put_record(PutRecordRequest::new(
            "acme",
            "data",
            "sessions",
            json!({"token": "abc123"}),
        ))
        .unwrap();

    db.delete_record("acme", "data", "sessions", &rec.id)
        .unwrap();

    let found = db.get_record("acme", "data", "sessions", &rec.id);
    assert!(found.is_none(), "deleted record must not be retrievable");
}

// ── Delete removes the record from recall ─────────────────────────────────────

#[test]
fn deleted_record_absent_from_recall() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    seed(&mut db);

    let rec = db
        .put_record(PutRecordRequest::new(
            "acme",
            "data",
            "logs",
            json!({"message": "neural inference pipeline started"}),
        ))
        .unwrap();

    db.delete_record("acme", "data", "logs", &rec.id).unwrap();

    let hits = db
        .recall(RecallRequest::new("acme", "neural inference pipeline"))
        .unwrap();
    assert!(
        !hits.iter().any(|h| h.id == rec.id),
        "deleted record must not appear in recall"
    );
}

// ── Records are tenant-isolated ───────────────────────────────────────────────

#[test]
fn records_are_tenant_isolated() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    db.create_tenant("alpha", "Alpha").unwrap();
    db.create_tenant("beta", "Beta").unwrap();
    db.create_collection("alpha", "data", "").unwrap();
    db.create_collection("beta", "data", "").unwrap();

    let alpha_rec = db
        .put_record(PutRecordRequest::new(
            "alpha",
            "data",
            "items",
            json!({"value": "alpha-secret"}),
        ))
        .unwrap();

    // beta must not see alpha's record.
    let found = db.get_record("beta", "data", "items", &alpha_rec.id);
    assert!(
        found.is_none(),
        "beta must not retrieve alpha's record by id"
    );

    let hits = db
        .recall(RecallRequest::new("beta", "alpha-secret"))
        .unwrap();
    assert!(
        !hits.iter().any(|h| h.id == alpha_rec.id),
        "beta's recall must not surface alpha's record"
    );
}
