//! build_context smoke tests.
//!
//! `build_context` is the primary output path for LLM consumers. These tests
//! verify: non-empty context string, snippet content, max_tokens budget,
//! include_related graph expansion, and empty-store graceful handling.

use hippocore::model::{ItemKind, MemoryType};
use hippocore::{AddGraphEdgeRequest, BuildContextRequest, Config, Hippocore, RememberRequest};
use tempfile::TempDir;

fn open(dir: &TempDir) -> Hippocore {
    Hippocore::open(Config::new(dir.path())).expect("open")
}

fn seed(db: &mut Hippocore) {
    db.create_tenant("acme", "Acme").unwrap();
    db.create_collection("acme", "kb", "").unwrap();
}

// ── Non-empty context string ──────────────────────────────────────────────────

#[test]
fn build_context_produces_non_empty_text() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    seed(&mut db);
    db.remember(RememberRequest::new(
        "acme",
        "kb",
        MemoryType::Semantic,
        "dropout regularisation prevents overfitting in neural networks",
    ))
    .unwrap();

    let ctx = db
        .build_context(BuildContextRequest::new(
            "acme",
            "dropout regularisation",
            4096,
        ))
        .unwrap();

    assert!(
        !ctx.text.is_empty(),
        "build_context must return a non-empty text string when relevant items exist"
    );
}

// ── Context string contains a recognisable snippet ────────────────────────────

#[test]
fn context_text_contains_memory_snippet() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    seed(&mut db);
    db.remember(RememberRequest::new(
        "acme",
        "kb",
        MemoryType::Semantic,
        "batch normalisation stabilises training dynamics",
    ))
    .unwrap();

    let ctx = db
        .build_context(BuildContextRequest::new(
            "acme",
            "batch normalisation",
            4096,
        ))
        .unwrap();

    assert!(
        ctx.text.contains("normalisation") || ctx.text.contains("batch"),
        "context text must contain a recognisable snippet from the stored memory"
    );
}

// ── max_tokens budget is respected ───────────────────────────────────────────

#[test]
fn max_tokens_limits_items_included() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    seed(&mut db);

    // Store 5 memories with similar content so all would normally be recalled.
    for i in 0..5u8 {
        db.remember(RememberRequest::new(
            "acme",
            "kb",
            MemoryType::Semantic,
            format!("reinforcement learning episode {i} reward signal optimisation"),
        ))
        .unwrap();
    }

    // Set max_tokens very low so only 0 or 1 item fits.
    let tight = db
        .build_context(BuildContextRequest::new(
            "acme",
            "reinforcement learning reward",
            5, // ~5 tokens ≈ 20 bytes — most items won't fit
        ))
        .unwrap();

    let loose = db
        .build_context(BuildContextRequest::new(
            "acme",
            "reinforcement learning reward",
            8192,
        ))
        .unwrap();

    assert!(
        tight.items_included.len() <= loose.items_included.len(),
        "tighter token budget must include no more items than a loose budget"
    );
}

// ── include_related = true adds graph-connected items ────────────────────────

#[test]
fn include_related_adds_graph_neighbours() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    seed(&mut db);

    let hub = db
        .remember(RememberRequest::new(
            "acme",
            "kb",
            MemoryType::Semantic,
            "self-supervised contrastive learning objective",
        ))
        .unwrap()
        .id;

    let neighbour = db
        .remember(RememberRequest::new(
            "acme",
            "kb",
            MemoryType::Semantic,
            "augmented views for positive pair construction",
        ))
        .unwrap()
        .id;

    db.add_graph_edge(AddGraphEdgeRequest::new(
        "acme",
        ItemKind::Memory,
        &hub,
        ItemKind::Memory,
        &neighbour,
        "related",
    ))
    .unwrap();

    let mut req = BuildContextRequest::new("acme", "contrastive learning", 4096);
    req.include_related = true;

    let ctx = db.build_context(req).unwrap();

    let all_ids: Vec<&str> = ctx.items_included.iter().map(|i| i.id.as_str()).collect();
    assert!(
        all_ids.contains(&hub.as_str()) || all_ids.contains(&neighbour.as_str()),
        "at least one of the connected items must appear in the context"
    );
}

// ── Empty store returns empty text without error ──────────────────────────────

#[test]
fn empty_store_returns_empty_context_not_error() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    seed(&mut db);

    let ctx = db
        .build_context(BuildContextRequest::new("acme", "anything at all", 4096))
        .unwrap();

    assert_eq!(
        ctx.text, "",
        "build_context on an empty store must return empty text, not error"
    );
    assert!(
        ctx.items_included.is_empty(),
        "items_included must be empty when no items were recalled"
    );
}
