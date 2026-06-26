//! Multi-hop GraphRAG traversal regression tests.
//!
//! `traverse_graph` follows GraphEdge links breadth-first up to max_hops
//! levels deep. These tests lock: 1-hop discovery, 2-hop chained discovery,
//! relation filter, max_nodes cap, seed items excluded from result, and
//! tenant isolation.

use hippocore::model::{ItemKind, MemoryType};
use hippocore::{AddGraphEdgeRequest, Config, Hippocore, RememberRequest, TraverseGraphRequest};
use tempfile::TempDir;

fn open(dir: &TempDir) -> Hippocore {
    Hippocore::open(Config::new(dir.path())).expect("open")
}

fn seed_memory(db: &mut Hippocore, tenant: &str, text: &str) -> String {
    db.remember(RememberRequest::new(
        tenant,
        "kb",
        MemoryType::Semantic,
        text,
    ))
    .unwrap()
    .id
}

fn setup(db: &mut Hippocore, tenant: &str) {
    db.create_tenant(tenant, tenant).unwrap();
    db.create_collection(tenant, "kb", "").unwrap();
}

fn link(db: &mut Hippocore, tenant: &str, from: &str, to: &str, relation: &str) {
    db.add_graph_edge(AddGraphEdgeRequest::new(
        tenant,
        ItemKind::Memory,
        from,
        ItemKind::Memory,
        to,
        relation,
    ))
    .unwrap();
}

// ── 1-hop traversal discovers direct neighbours ───────────────────────────────

#[test]
fn one_hop_discovers_neighbours() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    setup(&mut db, "acme");

    let a = seed_memory(&mut db, "acme", "node A");
    let b = seed_memory(&mut db, "acme", "node B");
    let c = seed_memory(&mut db, "acme", "node C unrelated");

    link(&mut db, "acme", &a, &b, "related");

    let req = TraverseGraphRequest::new("acme", [(ItemKind::Memory, a.clone())]);
    let nodes = db.traverse_graph(req);

    let found_ids: Vec<&str> = nodes.iter().map(|n| n.id.as_str()).collect();
    assert!(
        found_ids.contains(&b.as_str()),
        "direct neighbour B must be discovered"
    );
    assert!(
        !found_ids.contains(&c.as_str()),
        "unconnected C must not appear"
    );
    assert!(
        !found_ids.contains(&a.as_str()),
        "seed A must not appear in results"
    );
}

// ── Hop distance is correct ───────────────────────────────────────────────────

#[test]
fn hop_distance_is_correct() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    setup(&mut db, "acme");

    let a = seed_memory(&mut db, "acme", "alpha");
    let b = seed_memory(&mut db, "acme", "beta");
    let c = seed_memory(&mut db, "acme", "gamma");

    link(&mut db, "acme", &a, &b, "step");
    link(&mut db, "acme", &b, &c, "step");

    let mut req = TraverseGraphRequest::new("acme", [(ItemKind::Memory, a.clone())]);
    req.max_hops = 2;
    let nodes = db.traverse_graph(req);

    let b_node = nodes.iter().find(|n| n.id == b).expect("B must be found");
    let c_node = nodes
        .iter()
        .find(|n| n.id == c)
        .expect("C must be found at hop 2");

    assert_eq!(b_node.hop, 1, "B is 1 hop from A");
    assert_eq!(c_node.hop, 2, "C is 2 hops from A");
}

// ── max_hops = 1 stops at direct neighbours ───────────────────────────────────

#[test]
fn max_hops_one_stops_at_direct_neighbours() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    setup(&mut db, "acme");

    let a = seed_memory(&mut db, "acme", "alpha");
    let b = seed_memory(&mut db, "acme", "beta");
    let c = seed_memory(&mut db, "acme", "gamma");

    link(&mut db, "acme", &a, &b, "step");
    link(&mut db, "acme", &b, &c, "step");

    let mut req = TraverseGraphRequest::new("acme", [(ItemKind::Memory, a.clone())]);
    req.max_hops = 1;
    let nodes = db.traverse_graph(req);

    assert!(nodes.iter().any(|n| n.id == b), "B must be found at hop 1");
    assert!(
        !nodes.iter().any(|n| n.id == c),
        "C must not appear when max_hops=1"
    );
}

// ── relation_filter restricts which edges are followed ────────────────────────

#[test]
fn relation_filter_restricts_traversal() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    setup(&mut db, "acme");

    let a = seed_memory(&mut db, "acme", "root");
    let b = seed_memory(&mut db, "acme", "same-kind");
    let c = seed_memory(&mut db, "acme", "different-kind");

    link(&mut db, "acme", &a, &b, "supersedes");
    link(&mut db, "acme", &a, &c, "related");

    let mut req = TraverseGraphRequest::new("acme", [(ItemKind::Memory, a.clone())]);
    req.relation_filter = Some("supersedes".into());
    let nodes = db.traverse_graph(req);

    assert!(
        nodes.iter().any(|n| n.id == b),
        "B reached via 'supersedes' must be found"
    );
    assert!(
        !nodes.iter().any(|n| n.id == c),
        "C via 'related' must be excluded by filter"
    );
}

// ── max_nodes caps the result set ────────────────────────────────────────────

#[test]
fn max_nodes_caps_result() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    setup(&mut db, "acme");

    let root = seed_memory(&mut db, "acme", "root node");
    let mut others = Vec::new();
    for i in 0..10u8 {
        let id = seed_memory(&mut db, "acme", &format!("neighbour {i}"));
        link(&mut db, "acme", &root, &id, "fan");
        others.push(id);
    }

    let mut req = TraverseGraphRequest::new("acme", [(ItemKind::Memory, root.clone())]);
    req.max_nodes = 3;
    let nodes = db.traverse_graph(req);

    assert_eq!(nodes.len(), 3, "result must be capped at max_nodes=3");
}

// ── Traversal is tenant-isolated ─────────────────────────────────────────────

#[test]
fn traversal_is_tenant_isolated() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    setup(&mut db, "alpha");
    setup(&mut db, "beta");

    let a_root = seed_memory(&mut db, "alpha", "alpha root");
    let a_leaf = seed_memory(&mut db, "alpha", "alpha leaf");
    link(&mut db, "alpha", &a_root, &a_leaf, "link");

    // traverse from alpha root but scoped to beta — no results
    let req = TraverseGraphRequest::new("beta", [(ItemKind::Memory, a_root.clone())]);
    let nodes = db.traverse_graph(req);

    assert!(
        nodes.is_empty(),
        "beta traversal must not discover alpha's nodes"
    );
}
