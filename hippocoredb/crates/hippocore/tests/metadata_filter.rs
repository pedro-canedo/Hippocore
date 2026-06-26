//! Metadata filter regression tests.
//!
//! Exact-match metadata filtering must scope recall results to only items that
//! carry all specified key-value pairs. These tests cover all item kinds
//! (memories, document chunks, records), multi-key AND semantics, empty results,
//! collection-scoped filters, and cross-tenant non-interference.

use hippocore::model::MemoryType;
use hippocore::{
    BuildContextRequest, Config, Hippocore, PutRecordRequest, RecallRequest, RememberRequest,
};
use serde_json::json;
use std::collections::HashMap;
use tempfile::TempDir;

fn open_seeded(dir: &TempDir) -> Hippocore {
    let mut db = Hippocore::open(Config::new(dir.path())).expect("open");
    db.create_tenant("acme", "Acme").unwrap();
    db.create_collection("acme", "support", "").unwrap();
    db
}

fn meta(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

// ── Memory exact-match filter ─────────────────────────────────────────────────

#[test]
fn metadata_filter_exact_match_memories() {
    let dir = TempDir::new().unwrap();
    let mut db = open_seeded(&dir);

    db.remember(RememberRequest {
        metadata: meta(&[("env", "prod"), ("component", "db")]),
        ..RememberRequest::new(
            "acme",
            "support",
            MemoryType::Semantic,
            "postgresql connection pool configuration",
        )
    })
    .unwrap();
    db.remember(RememberRequest {
        metadata: meta(&[("env", "staging"), ("component", "db")]),
        ..RememberRequest::new(
            "acme",
            "support",
            MemoryType::Semantic,
            "postgresql connection pool tuning",
        )
    })
    .unwrap();

    let mut req = RecallRequest::new("acme", "postgresql connection pool");
    req.metadata = meta(&[("env", "prod")]);
    let hits = db.recall(req).unwrap();

    assert!(
        !hits.is_empty(),
        "expected at least one result with env=prod"
    );
    for hit in &hits {
        assert_eq!(
            hit.metadata.get("env").map(String::as_str),
            Some("prod"),
            "result has unexpected env metadata: {:?}",
            hit.metadata
        );
    }
}

// ── Multi-key AND semantics ───────────────────────────────────────────────────

#[test]
fn metadata_filter_multi_key_and_semantics() {
    let dir = TempDir::new().unwrap();
    let mut db = open_seeded(&dir);

    db.remember(RememberRequest {
        metadata: meta(&[("env", "prod"), ("tier", "api")]),
        ..RememberRequest::new(
            "acme",
            "support",
            MemoryType::Semantic,
            "nginx request routing configuration",
        )
    })
    .unwrap();
    db.remember(RememberRequest {
        metadata: meta(&[("env", "prod"), ("tier", "db")]),
        ..RememberRequest::new(
            "acme",
            "support",
            MemoryType::Semantic,
            "nginx upstream database timeout",
        )
    })
    .unwrap();
    db.remember(RememberRequest {
        metadata: meta(&[("env", "staging"), ("tier", "api")]),
        ..RememberRequest::new(
            "acme",
            "support",
            MemoryType::Semantic,
            "nginx staging api routing",
        )
    })
    .unwrap();

    // Only the first memory should match: env=prod AND tier=api.
    let mut req = RecallRequest::new("acme", "nginx");
    req.metadata = meta(&[("env", "prod"), ("tier", "api")]);
    let hits = db.recall(req).unwrap();

    assert!(!hits.is_empty(), "expected env=prod AND tier=api to match");
    for hit in &hits {
        assert_eq!(
            hit.metadata.get("env").map(String::as_str),
            Some("prod"),
            "multi-key filter returned wrong env: {:?}",
            hit.metadata
        );
        assert_eq!(
            hit.metadata.get("tier").map(String::as_str),
            Some("api"),
            "multi-key filter returned wrong tier: {:?}",
            hit.metadata
        );
    }
}

// ── Empty result is not an error ──────────────────────────────────────────────

#[test]
fn metadata_filter_no_match_returns_empty_not_error() {
    let dir = TempDir::new().unwrap();
    let mut db = open_seeded(&dir);

    db.remember(RememberRequest::new(
        "acme",
        "support",
        MemoryType::Semantic,
        "postgresql connection pool configuration",
    ))
    .unwrap();

    let mut req = RecallRequest::new("acme", "postgresql");
    req.metadata = meta(&[("env", "nonexistent-value-xyz")]);
    let hits = db.recall(req).unwrap();
    assert!(
        hits.is_empty(),
        "expected empty result for non-matching filter"
    );
}

// ── Collection-scoped filter ──────────────────────────────────────────────────

#[test]
fn metadata_filter_scoped_to_collection() {
    let dir = TempDir::new().unwrap();
    let mut db = open_seeded(&dir);
    db.create_collection("acme", "infra", "").unwrap();

    db.remember(RememberRequest {
        metadata: meta(&[("env", "prod")]),
        ..RememberRequest::new(
            "acme",
            "support",
            MemoryType::Semantic,
            "postgresql support configuration",
        )
    })
    .unwrap();
    db.remember(RememberRequest {
        metadata: meta(&[("env", "prod")]),
        ..RememberRequest::new(
            "acme",
            "infra",
            MemoryType::Semantic,
            "postgresql infra provisioning",
        )
    })
    .unwrap();

    let mut req = RecallRequest::new("acme", "postgresql");
    req.metadata = meta(&[("env", "prod")]);
    req.collection = Some("support".into());
    let hits = db.recall(req).unwrap();

    for hit in &hits {
        assert_eq!(
            hit.collection, "support",
            "collection-scoped filter returned item from collection '{}'",
            hit.collection
        );
    }
}

// ── Record exact-match filter ─────────────────────────────────────────────────

#[test]
fn metadata_filter_records() {
    let dir = TempDir::new().unwrap();
    let mut db = open_seeded(&dir);

    db.put_record(PutRecordRequest {
        metadata: meta(&[("region", "us-east-1")]),
        ..PutRecordRequest::new(
            "acme",
            "support",
            "server",
            json!({"host": "db-01", "role": "primary"}),
        )
    })
    .unwrap();
    db.put_record(PutRecordRequest {
        metadata: meta(&[("region", "eu-west-1")]),
        ..PutRecordRequest::new(
            "acme",
            "support",
            "server",
            json!({"host": "db-02", "role": "replica"}),
        )
    })
    .unwrap();

    let mut req = RecallRequest::new("acme", "db primary role");
    req.metadata = meta(&[("region", "us-east-1")]);
    let hits = db.recall(req).unwrap();

    for hit in &hits {
        assert_eq!(
            hit.metadata.get("region").map(String::as_str),
            Some("us-east-1"),
            "record filter returned wrong region: {:?}",
            hit.metadata
        );
    }
}

// ── Cross-tenant filter non-interference ─────────────────────────────────────

#[test]
fn metadata_filter_does_not_cross_tenant_boundary() {
    let dir = TempDir::new().unwrap();
    let mut db = Hippocore::open(Config::new(dir.path())).expect("open");

    for tenant in ["alpha", "beta"] {
        db.create_tenant(tenant, tenant).unwrap();
        db.create_collection(tenant, "shared", "").unwrap();
        db.remember(RememberRequest {
            metadata: meta(&[("env", "prod")]),
            ..RememberRequest::new(
                tenant,
                "shared",
                MemoryType::Semantic,
                "postgresql connection pool configuration",
            )
        })
        .unwrap();
    }

    let mut req = RecallRequest::new("alpha", "postgresql");
    req.metadata = meta(&[("env", "prod")]);
    let hits = db.recall(req).unwrap();

    for hit in &hits {
        assert_eq!(
            hit.tenant_id, "alpha",
            "metadata filter returned item from tenant '{}'",
            hit.tenant_id
        );
    }
}

// ── build_context respects metadata filter ────────────────────────────────────

#[test]
fn build_context_metadata_filter_respected() {
    let dir = TempDir::new().unwrap();
    let mut db = open_seeded(&dir);

    db.remember(RememberRequest {
        metadata: meta(&[("env", "prod")]),
        ..RememberRequest::new(
            "acme",
            "support",
            MemoryType::Semantic,
            "postgresql connection pool configuration",
        )
    })
    .unwrap();
    db.remember(RememberRequest {
        metadata: meta(&[("env", "dev")]),
        ..RememberRequest::new(
            "acme",
            "support",
            MemoryType::Semantic,
            "postgresql development connection setup",
        )
    })
    .unwrap();

    let mut req = BuildContextRequest::new("acme", "postgresql connection pool", 4096);
    req.metadata_filter = meta(&[("env", "prod")]);
    let block = db.build_context(req).unwrap();

    // Cross-check: all returned items should be in the prod-filtered recall set.
    let mut recall_req = RecallRequest::new("acme", "postgresql connection pool");
    recall_req.metadata = meta(&[("env", "prod")]);
    let prod_hits = db.recall(recall_req).unwrap();
    let prod_ids: std::collections::HashSet<&str> =
        prod_hits.iter().map(|h| h.id.as_str()).collect();

    for item in &block.items_included {
        assert!(
            prod_ids.contains(item.id.as_str()),
            "build_context returned item '{}' not in metadata-filtered recall set",
            item.id
        );
    }
}
