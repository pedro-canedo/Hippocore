//! Recall latency regression guard.
//!
//! Measures hybrid recall latency over a 500-memory dataset, compares the
//! result against a committed baseline, and exits non-zero if the mean
//! latency regresses beyond `HIPPO_BENCH_THRESHOLD` (default: 0.20 = 20%).
//!
//! # Usage
//!
//! ```
//! # Check against committed baseline (exits non-zero on regression):
//! cargo run --release --example bench_regression
//!
//! # Update the committed baseline (after intentional performance changes):
//! HIPPO_BENCH_UPDATE=1 cargo run --release --example bench_regression
//! ```

use std::path::PathBuf;
use std::time::{Duration, Instant};

use hippocore::model::MemoryType;
use hippocore::{Config, Hippocore, RecallRequest, RememberRequest, SearchMode};
use serde::{Deserialize, Serialize};

const N_MEMORIES: usize = 500;
const WARMUP_ITERS: usize = 5;
const MEASURE_ITERS: usize = 50;
const DEFAULT_THRESHOLD: f64 = 0.20; // 20% slower than baseline = failure

fn baseline_path() -> PathBuf {
    // CARGO_MANIFEST_DIR is the hippocore crate (crates/hippocore).
    // Two levels up is the workspace root (hippocoredb/).
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(manifest)
        .join("..")
        .join("..")
        .join("benches")
        .join("baseline.json")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BenchResult {
    name: String,
    mean_ns: u64,
    samples: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Baseline {
    benchmarks: Vec<BenchResult>,
}

fn build_db() -> (tempfile::TempDir, Hippocore) {
    let dir = tempfile::tempdir().unwrap();
    let mut cfg = Config::new(dir.path());
    cfg.sync_writes = false;
    let mut db = Hippocore::open(cfg).unwrap();
    db.create_tenant("t", "t").unwrap();
    db.create_collection("t", "c", "").unwrap();
    for i in 0..N_MEMORIES {
        let req = RememberRequest::new(
            "t",
            "c",
            MemoryType::Note,
            format!("memory {i} about oracle networking and postgresql listeners"),
        );
        db.remember(req).unwrap();
    }
    (dir, db)
}

fn measure(db: &Hippocore, mode: SearchMode) -> Duration {
    let query = "oracle listener networking postgresql";
    // Warm up.
    for _ in 0..WARMUP_ITERS {
        let mut req = RecallRequest::new("t", query);
        req.mode = mode;
        req.top_k = 10;
        std::hint::black_box(db.recall(req).unwrap().len());
    }
    // Measure.
    let start = Instant::now();
    for _ in 0..MEASURE_ITERS {
        let mut req = RecallRequest::new("t", query);
        req.mode = mode;
        req.top_k = 10;
        std::hint::black_box(db.recall(req).unwrap().len());
    }
    start.elapsed() / MEASURE_ITERS as u32
}

fn main() {
    let update = std::env::var("HIPPO_BENCH_UPDATE").is_ok();
    let threshold: f64 = std::env::var("HIPPO_BENCH_THRESHOLD")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_THRESHOLD);

    println!("Building populated DB ({N_MEMORIES} memories)…");
    let (_dir, db) = build_db();

    println!(
        "Measuring recall latency ({MEASURE_ITERS} iterations after {WARMUP_ITERS} warmups)…\n"
    );

    let benchmarks = vec![
        BenchResult {
            name: "recall_hybrid".to_string(),
            mean_ns: measure(&db, SearchMode::Hybrid).as_nanos() as u64,
            samples: MEASURE_ITERS,
        },
        BenchResult {
            name: "recall_vector".to_string(),
            mean_ns: measure(&db, SearchMode::Vector).as_nanos() as u64,
            samples: MEASURE_ITERS,
        },
        BenchResult {
            name: "recall_text".to_string(),
            mean_ns: measure(&db, SearchMode::Text).as_nanos() as u64,
            samples: MEASURE_ITERS,
        },
    ];

    for b in &benchmarks {
        println!("  {:20}  {:>10} µs", b.name, b.mean_ns / 1000);
    }
    println!();

    if update {
        let baseline = Baseline {
            benchmarks: benchmarks.clone(),
        };
        let path = baseline_path();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, serde_json::to_string_pretty(&baseline).unwrap()).unwrap();
        println!("Baseline written to {}", path.display());
        return;
    }

    let path = baseline_path();
    if !path.exists() {
        println!(
            "No baseline found at {}. Run with HIPPO_BENCH_UPDATE=1 to create one.",
            path.display()
        );
        std::process::exit(0);
    }

    let raw = std::fs::read_to_string(&path).expect("read baseline");
    let baseline: Baseline = serde_json::from_str(&raw).expect("parse baseline");

    let mut failures = Vec::new();
    for current in &benchmarks {
        if let Some(base) = baseline.benchmarks.iter().find(|b| b.name == current.name) {
            let ratio = current.mean_ns as f64 / base.mean_ns as f64;
            let status = if ratio > 1.0 + threshold {
                "FAIL"
            } else if ratio > 1.0 {
                "SLOWER"
            } else {
                "OK"
            };
            println!(
                "  {:20}  {:>10} µs  (baseline {:>10} µs, {:.1}%)  {}",
                current.name,
                current.mean_ns / 1000,
                base.mean_ns / 1000,
                (ratio - 1.0) * 100.0,
                status
            );
            if ratio > 1.0 + threshold {
                failures.push(format!(
                    "{}: {:.1}% regression (threshold {:.0}%)",
                    current.name,
                    (ratio - 1.0) * 100.0,
                    threshold * 100.0
                ));
            }
        } else {
            println!(
                "  {:20}  {:>10} µs  (no baseline)",
                current.name,
                current.mean_ns / 1000
            );
        }
    }
    println!();

    if failures.is_empty() {
        println!("Result: PASS");
    } else {
        println!("Result: FAIL");
        for f in &failures {
            println!("  ✗ {f}");
        }
        std::process::exit(1);
    }
}
