//! Contradiction advisory regression tests.
//!
//! The `contradicts` field on a memory marks two memories as conflicting.
//! This is advisory: both items are still returned by recall, but the
//! `contradictions` field on each RecallResult is populated so callers can
//! surface the conflict. When contradictions are present in a build_context
//! candidate set, confidence-aware re-ranking promotes the higher-confidence
//! item above the lower-confidence one.

use hippocore::model::MemoryType;
use hippocore::{BuildContextRequest, Config, Hippocore, RecallRequest, RememberRequest};
use tempfile::TempDir;

fn open_seeded(dir: &TempDir) -> Hippocore {
    let mut db = Hippocore::open(Config::new(dir.path())).expect("open");
    db.create_tenant("acme", "Acme").unwrap();
    db.create_collection("acme", "support", "").unwrap();
    db
}

/// Store A (low confidence), then B (high confidence, contradicts A).
/// Returns (id_a, id_b).
fn seed_contradiction(
    db: &mut Hippocore,
    conf_a: Option<f32>,
    conf_b: Option<f32>,
) -> (String, String) {
    let id_a = db
        .remember(RememberRequest {
            confidence: conf_a,
            ..RememberRequest::new(
                "acme",
                "support",
                MemoryType::Semantic,
                "postgresql max_connections should be set to 100",
            )
        })
        .unwrap()
        .id;

    let id_b = db
        .remember(RememberRequest {
            confidence: conf_b,
            contradicts: vec![id_a.clone()],
            ..RememberRequest::new(
                "acme",
                "support",
                MemoryType::Semantic,
                "postgresql max_connections should be set to 200",
            )
        })
        .unwrap()
        .id;

    (id_a, id_b)
}

// ── Both items appear in recall (advisory only) ───────────────────────────────

#[test]
fn contradicting_memories_both_appear_in_recall() {
    let dir = TempDir::new().unwrap();
    let mut db = open_seeded(&dir);
    let (id_a, id_b) = seed_contradiction(&mut db, Some(0.3), Some(0.9));

    let mut req = RecallRequest::new("acme", "postgresql max_connections");
    req.include_superseded = true; // ensure nothing is suppressed
    let hits = db.recall(req).unwrap();

    let ids: Vec<&str> = hits.iter().map(|h| h.id.as_str()).collect();
    assert!(ids.contains(&id_a.as_str()), "A must appear in recall");
    assert!(ids.contains(&id_b.as_str()), "B must appear in recall");
}

// ── contradictions advisory is populated ─────────────────────────────────────

#[test]
fn contradictions_advisory_populated_in_recall_result() {
    let dir = TempDir::new().unwrap();
    let mut db = open_seeded(&dir);
    let (id_a, id_b) = seed_contradiction(&mut db, Some(0.3), Some(0.9));

    let hits = db
        .recall(RecallRequest::new("acme", "postgresql max_connections"))
        .unwrap();

    // B contradicts A — B's result should carry id_a in contradictions.
    let hit_b = hits
        .iter()
        .find(|h| h.id == id_b)
        .expect("B must be in hits");
    assert!(
        hit_b.contradictions.contains(&id_a),
        "B's recall result must list A in contradictions, got {:?}",
        hit_b.contradictions
    );
}

// ── build_context confidence re-ranking ──────────────────────────────────────

#[test]
fn build_context_confidence_reranking_prefers_higher_confidence() {
    let dir = TempDir::new().unwrap();
    let mut db = open_seeded(&dir);
    // A has very low confidence; B has high confidence and contradicts A.
    let (id_a, id_b) = seed_contradiction(&mut db, Some(0.1), Some(0.95));

    let block = db
        .build_context(BuildContextRequest::new(
            "acme",
            "postgresql max_connections",
            4096,
        ))
        .unwrap();

    let ids: Vec<&str> = block.items_included.iter().map(|i| i.id.as_str()).collect();
    let pos_a = ids.iter().position(|id| *id == id_a.as_str());
    let pos_b = ids.iter().position(|id| *id == id_b.as_str());

    if let (Some(a), Some(b)) = (pos_a, pos_b) {
        assert!(
            b < a,
            "higher-confidence B (pos {b}) must rank above lower-confidence A (pos {a})"
        );
    }
    // If only one appears, it should be B (higher confidence).
    if pos_a.is_none() {
        assert!(pos_b.is_some(), "at least B must be in context");
    }
}

// ── No contradiction, no re-ranking ─────────────────────────────────────────

#[test]
fn build_context_no_contradiction_no_reranking() {
    let dir = TempDir::new().unwrap();
    let mut db = open_seeded(&dir);

    // Store two independent memories (no contradicts relationship).
    db.remember(RememberRequest {
        confidence: Some(0.1),
        ..RememberRequest::new(
            "acme",
            "support",
            MemoryType::Semantic,
            "postgresql max_connections tuning for high load",
        )
    })
    .unwrap();
    db.remember(RememberRequest {
        confidence: Some(0.9),
        ..RememberRequest::new(
            "acme",
            "support",
            MemoryType::Semantic,
            "postgresql max_connections default value is 100",
        )
    })
    .unwrap();

    // Plain recall order (no contradictions → no confidence re-ranking in
    // build_context). Both items must appear; no error.
    let plain_hits = db
        .recall(RecallRequest::new("acme", "postgresql max_connections"))
        .unwrap();
    let block = db
        .build_context(BuildContextRequest::new(
            "acme",
            "postgresql max_connections",
            4096,
        ))
        .unwrap();

    // All items from recall must appear in context (token budget is large).
    for hit in &plain_hits {
        assert!(
            block.items_included.iter().any(|i| i.id == hit.id),
            "item '{}' in recall must appear in build_context (no contradiction suppression)",
            hit.id
        );
    }
}
