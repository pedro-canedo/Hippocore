//! Deterministic retrieval-quality validation.
//!
//! This is not a model-generation test. It validates whether Hippocore retrieves
//! the memories that would ground the expected answer, and reports ranking
//! metrics that can be tracked as retrieval changes.

use std::collections::HashMap;

use hippocore::model::MemoryType;
use hippocore::{Config, Hippocore, RecallRequest, RememberRequest};
use serde::Deserialize;

const FIXTURE: &str = include_str!("fixtures/retrieval_quality_v01.json");
const TENANT: &str = "eval";
const COLLECTION: &str = "support";
const TOP_K: usize = 5;

#[derive(Debug, Deserialize)]
struct Fixture {
    version: u32,
    memories: Vec<MemoryFixture>,
    cases: Vec<CaseFixture>,
    thresholds: Thresholds,
}

#[derive(Debug, Deserialize)]
struct MemoryFixture {
    id: String,
    text: String,
    metadata: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct CaseFixture {
    name: String,
    query: String,
    relevant_ids: Vec<String>,
    expected_first_ids: Vec<String>,
    forbidden_first_ids: Vec<String>,
    answer_must_include: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Thresholds {
    min_hit_at_1: f32,
    min_hit_at_k: f32,
    min_mrr: f32,
}

#[derive(Debug, Default)]
struct Metrics {
    cases: usize,
    hit_at_1: usize,
    hit_at_k: usize,
    reciprocal_rank_sum: f32,
}

impl Metrics {
    fn record(&mut self, rank: Option<usize>) {
        self.cases += 1;
        if rank == Some(1) {
            self.hit_at_1 += 1;
        }
        if rank.is_some_and(|r| r <= TOP_K) {
            self.hit_at_k += 1;
        }
        if let Some(rank) = rank {
            self.reciprocal_rank_sum += 1.0 / rank as f32;
        }
    }

    fn hit_at_1(&self) -> f32 {
        self.hit_at_1 as f32 / self.cases as f32
    }

    fn hit_at_k(&self) -> f32 {
        self.hit_at_k as f32 / self.cases as f32
    }

    fn mrr(&self) -> f32 {
        self.reciprocal_rank_sum / self.cases as f32
    }
}

#[test]
fn retrieval_quality_fixture_meets_thresholds() {
    let fixture: Fixture = serde_json::from_str(FIXTURE).expect("valid retrieval quality fixture");
    assert_eq!(fixture.version, 1);

    let dir = tempfile::tempdir().unwrap();
    let mut cfg = Config::new(dir.path());
    cfg.sync_writes = false;
    let mut db = Hippocore::open(cfg).unwrap();
    db.create_tenant(TENANT, "Retrieval Eval").unwrap();
    db.create_collection(TENANT, COLLECTION, "Support retrieval fixture")
        .unwrap();

    for memory in &fixture.memories {
        let mut req = RememberRequest::new(
            TENANT,
            COLLECTION,
            MemoryType::Semantic,
            memory.text.clone(),
        );
        req.id = Some(memory.id.clone());
        req.metadata = memory.metadata.clone();
        db.remember(req).unwrap();
    }

    let mut metrics = Metrics::default();
    let mut failures = Vec::new();

    for case in &fixture.cases {
        let mut req = RecallRequest::new(TENANT, &case.query);
        req.top_k = TOP_K;
        let hits = db.recall(req).unwrap();
        let ids: Vec<&str> = hits.iter().map(|hit| hit.id.as_str()).collect();
        let first = ids.first().copied();
        let rank = ids
            .iter()
            .position(|id| case.relevant_ids.iter().any(|relevant| relevant == id))
            .map(|idx| idx + 1);
        metrics.record(rank);

        if !first.is_some_and(|id| {
            case.expected_first_ids
                .iter()
                .any(|expected| expected == id)
        }) {
            failures.push(format!(
                "{}: first={first:?}, expected one of {:?}, top={ids:?}",
                case.name, case.expected_first_ids
            ));
        }

        if first.is_some_and(|id| {
            case.forbidden_first_ids
                .iter()
                .any(|forbidden| forbidden == id)
        }) {
            failures.push(format!(
                "{}: forbidden first result {first:?}, top={ids:?}",
                case.name
            ));
        }

        let retrieved_text = hits
            .iter()
            .map(|hit| hit.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        for required in &case.answer_must_include {
            if !retrieved_text
                .to_lowercase()
                .contains(&required.to_lowercase())
            {
                failures.push(format!(
                    "{}: retrieved context missing answer term {:?}, top={ids:?}",
                    case.name, required
                ));
            }
        }
    }

    let hit_at_1 = metrics.hit_at_1();
    let hit_at_k = metrics.hit_at_k();
    let mrr = metrics.mrr();
    println!(
        "retrieval_quality: cases={} hit@1={hit_at_1:.3} hit@{TOP_K}={hit_at_k:.3} mrr={mrr:.3}",
        metrics.cases
    );

    if hit_at_1 < fixture.thresholds.min_hit_at_1 {
        failures.push(format!(
            "hit@1 {hit_at_1:.3} below threshold {:.3}",
            fixture.thresholds.min_hit_at_1
        ));
    }
    if hit_at_k < fixture.thresholds.min_hit_at_k {
        failures.push(format!(
            "hit@{TOP_K} {hit_at_k:.3} below threshold {:.3}",
            fixture.thresholds.min_hit_at_k
        ));
    }
    if mrr < fixture.thresholds.min_mrr {
        failures.push(format!(
            "mrr {mrr:.3} below threshold {:.3}",
            fixture.thresholds.min_mrr
        ));
    }

    assert!(
        failures.is_empty(),
        "retrieval quality failed: metrics hit@1={hit_at_1:.3} hit@{TOP_K}={hit_at_k:.3} mrr={mrr:.3}\n{}",
        failures.join("\n")
    );
}
