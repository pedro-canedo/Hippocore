//! Tenant isolation regression tests.
//!
//! Every public query path must return only data belonging to the queried tenant.
//! These tests deliberately store identical content in multiple tenants and assert
//! that results never cross boundaries — even when tenants share collection names
//! and the same query is issued.

use hippocore::model::{ItemKind, MemoryType};
use hippocore::{
    AddGraphEdgeRequest, BuildContextRequest, Config, Hippocore, RecallRequest, RememberRequest,
    SearchMode, StoreDocumentRequest,
};
use tempfile::TempDir;

/// Open a database and populate two tenants ("alpha", "beta") with identical
/// collection names ("shared") and semantically identical content.
fn two_tenant_db(dir: &TempDir) -> Hippocore {
    let mut db = Hippocore::open(Config::new(dir.path())).expect("open");
    for tenant in ["alpha", "beta"] {
        db.create_tenant(tenant, tenant).unwrap();
        db.create_collection(tenant, "shared", "shared collection")
            .unwrap();

        db.remember(RememberRequest::new(
            tenant,
            "shared",
            MemoryType::Semantic,
            "postgresql connection pool configuration and tuning",
        ))
        .unwrap();
        db.store_document(StoreDocumentRequest::new(
            tenant,
            "shared",
            "Oracle listener startup procedure using lsnrctl",
        ))
        .unwrap();
    }
    db
}

// ── recall() ─────────────────────────────────────────────────────────────────

#[test]
fn recall_vector_stays_within_tenant() {
    let dir = TempDir::new().unwrap();
    let db = two_tenant_db(&dir);
    let mut req = RecallRequest::new("alpha", "postgresql connection pool");
    req.mode = SearchMode::Vector;
    let hits = db.recall(req).unwrap();
    for hit in &hits {
        assert_eq!(
            hit.tenant_id, "alpha",
            "vector recall returned item from tenant '{}', expected 'alpha'",
            hit.tenant_id
        );
    }
}

#[test]
fn recall_text_stays_within_tenant() {
    let dir = TempDir::new().unwrap();
    let db = two_tenant_db(&dir);
    let mut req = RecallRequest::new("beta", "postgresql connection pool");
    req.mode = SearchMode::Text;
    let hits = db.recall(req).unwrap();
    for hit in &hits {
        assert_eq!(
            hit.tenant_id, "beta",
            "text recall returned item from tenant '{}', expected 'beta'",
            hit.tenant_id
        );
    }
}

#[test]
fn recall_hybrid_stays_within_tenant() {
    let dir = TempDir::new().unwrap();
    let db = two_tenant_db(&dir);
    let mut req = RecallRequest::new("alpha", "postgresql");
    req.mode = SearchMode::Hybrid;
    let hits = db.recall(req).unwrap();
    for hit in &hits {
        assert_eq!(
            hit.tenant_id, "alpha",
            "hybrid recall returned item from tenant '{}', expected 'alpha'",
            hit.tenant_id
        );
    }
}

// ── build_context() ──────────────────────────────────────────────────────────

#[test]
fn build_context_stays_within_tenant() {
    let dir = TempDir::new().unwrap();
    let db = two_tenant_db(&dir);
    let block = db
        .build_context(BuildContextRequest::new("alpha", "postgresql", 4096))
        .unwrap();
    // All ContextItems belong to "alpha" — verified by cross-checking IDs
    // against recall results scoped to "alpha".
    let alpha_hits = db
        .recall(RecallRequest::new("alpha", "postgresql"))
        .unwrap();
    let alpha_ids: std::collections::HashSet<&str> =
        alpha_hits.iter().map(|h| h.id.as_str()).collect();
    for item in &block.items_included {
        assert!(
            alpha_ids.contains(item.id.as_str()),
            "build_context returned item '{}' not in alpha's recall set",
            item.id
        );
    }
}

#[test]
fn build_context_with_related_stays_within_tenant() {
    let dir = TempDir::new().unwrap();
    let mut db = two_tenant_db(&dir);

    // Add cross-referencing edges within "alpha" only.
    let alpha_ids: Vec<String> = db
        .recall(RecallRequest::new("alpha", "postgresql"))
        .unwrap()
        .into_iter()
        .map(|h| h.id)
        .collect();
    if alpha_ids.len() >= 2 {
        db.add_graph_edge(AddGraphEdgeRequest::new(
            "alpha",
            ItemKind::Memory,
            &alpha_ids[0],
            ItemKind::Memory,
            &alpha_ids[0], // self-edge harmless; just need the API call to succeed
            "related",
        ))
        .unwrap();
    }

    let mut req = BuildContextRequest::new("beta", "postgresql", 4096);
    req.include_related = true;
    req.related_limit = 10;
    let block = db.build_context(req).unwrap();

    let beta_hits = db.recall(RecallRequest::new("beta", "postgresql")).unwrap();
    let beta_ids: std::collections::HashSet<&str> =
        beta_hits.iter().map(|h| h.id.as_str()).collect();
    for item in &block.items_included {
        assert!(
            beta_ids.contains(item.id.as_str()),
            "build_context with related returned item '{}' not in beta's recall set",
            item.id
        );
    }
}

// ── graph edges ──────────────────────────────────────────────────────────────

#[test]
fn list_graph_edges_stays_within_tenant() {
    let dir = TempDir::new().unwrap();
    let mut db = two_tenant_db(&dir);

    // Seed one edge in each tenant.
    for tenant in ["alpha", "beta"] {
        let ids: Vec<String> = db
            .recall(RecallRequest::new(tenant, "postgresql"))
            .unwrap()
            .into_iter()
            .map(|h| h.id)
            .collect();
        if ids.len() >= 2 {
            db.add_graph_edge(AddGraphEdgeRequest::new(
                tenant,
                ItemKind::Memory,
                &ids[0],
                ItemKind::Memory,
                &ids[0],
                "related",
            ))
            .unwrap();
        }
    }

    let alpha_edges = db.list_graph_edges("alpha", None);
    for edge in &alpha_edges {
        assert_eq!(
            edge.tenant_id, "alpha",
            "list_graph_edges returned edge from tenant '{}', expected 'alpha'",
            edge.tenant_id
        );
    }

    let beta_edges = db.list_graph_edges("beta", None);
    for edge in &beta_edges {
        assert_eq!(
            edge.tenant_id, "beta",
            "list_graph_edges returned edge from tenant '{}', expected 'beta'",
            edge.tenant_id
        );
    }
}
