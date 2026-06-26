//! Graph edge lifecycle regression tests.
//!
//! Graph edges are the backbone of the graph-aware context ranking. These
//! tests verify the full edge lifecycle: add, list, relation type storage,
//! delete, durability through compact + reopen, and tenant isolation.

use hippocore::model::{ItemKind, MemoryType};
use hippocore::{AddGraphEdgeRequest, Config, Hippocore, RememberRequest};
use tempfile::TempDir;

fn open(dir: &TempDir) -> Hippocore {
    Hippocore::open(Config::new(dir.path())).expect("open")
}

fn seed_two_memories(db: &mut Hippocore, tenant: &str) -> (String, String) {
    db.create_tenant(tenant, tenant).unwrap();
    db.create_collection(tenant, "kb", "").unwrap();
    let a = db
        .remember(RememberRequest::new(
            tenant,
            "kb",
            MemoryType::Semantic,
            "alpha node",
        ))
        .unwrap()
        .id;
    let b = db
        .remember(RememberRequest::new(
            tenant,
            "kb",
            MemoryType::Semantic,
            "beta node",
        ))
        .unwrap()
        .id;
    (a, b)
}

// ── Added edge appears in list_graph_edges ────────────────────────────────────

#[test]
fn added_edge_appears_in_list() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    let (a, b) = seed_two_memories(&mut db, "acme");

    db.add_graph_edge(AddGraphEdgeRequest::new(
        "acme",
        ItemKind::Memory,
        &a,
        ItemKind::Memory,
        &b,
        "related",
    ))
    .unwrap();

    let edges = db.list_graph_edges("acme", None);
    assert!(
        edges.iter().any(|e| e.from_id == a && e.to_id == b),
        "added edge must appear in list_graph_edges"
    );
}

// ── Relation type is stored and returned ─────────────────────────────────────

#[test]
fn relation_type_is_stored() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    let (a, b) = seed_two_memories(&mut db, "acme");

    db.add_graph_edge(AddGraphEdgeRequest::new(
        "acme",
        ItemKind::Memory,
        &a,
        ItemKind::Memory,
        &b,
        "supersedes",
    ))
    .unwrap();

    let edges = db.list_graph_edges("acme", None);
    let edge = edges
        .into_iter()
        .find(|e| e.from_id == a && e.to_id == b)
        .expect("edge must exist");
    assert_eq!(edge.relation, "supersedes", "relation type must round-trip");
}

// ── delete_graph_edge removes the edge ────────────────────────────────────────

#[test]
fn deleted_edge_absent_from_list() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    let (a, b) = seed_two_memories(&mut db, "acme");

    let edge = db
        .add_graph_edge(AddGraphEdgeRequest::new(
            "acme",
            ItemKind::Memory,
            &a,
            ItemKind::Memory,
            &b,
            "related",
        ))
        .unwrap();

    db.delete_graph_edge("acme", &edge.id).unwrap();

    let edges = db.list_graph_edges("acme", None);
    assert!(
        !edges.iter().any(|e| e.id == edge.id),
        "deleted edge must not appear in list"
    );
}

// ── Edges survive compact + cold reopen ──────────────────────────────────────

#[test]
fn edges_survive_compact_reopen() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    let (a, b) = seed_two_memories(&mut db, "acme");

    let edge = db
        .add_graph_edge(AddGraphEdgeRequest::new(
            "acme",
            ItemKind::Memory,
            &a,
            ItemKind::Memory,
            &b,
            "linked",
        ))
        .unwrap();

    db.compact().unwrap();
    drop(db);

    let db2 = open(&dir);
    let edges = db2.list_graph_edges("acme", None);
    assert!(
        edges.iter().any(|e| e.id == edge.id),
        "edge must survive compact + cold reopen"
    );
}

// ── list_graph_edges is tenant-isolated ──────────────────────────────────────

#[test]
fn graph_edges_are_tenant_isolated() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);

    let (alpha_a, alpha_b) = seed_two_memories(&mut db, "alpha");
    seed_two_memories(&mut db, "beta"); // beta has no edges

    db.add_graph_edge(AddGraphEdgeRequest::new(
        "alpha",
        ItemKind::Memory,
        &alpha_a,
        ItemKind::Memory,
        &alpha_b,
        "alpha-edge",
    ))
    .unwrap();

    let beta_edges = db.list_graph_edges("beta", None);
    assert!(
        beta_edges.is_empty(),
        "beta must not see alpha's graph edges"
    );
}
