//! Baseline benchmarks: remember, recall (hybrid) and pure vector search.
//!
//! Run with `cargo bench -p hippocore`. These establish a baseline rather than
//! chasing numbers.

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use hippocore::model::MemoryType;
use hippocore::{Config, Hippocore, RecallRequest, RememberRequest, SearchMode};

const N: usize = 500;

fn fresh_db() -> (tempfile::TempDir, Hippocore) {
    let dir = tempfile::tempdir().unwrap();
    let mut cfg = Config::new(dir.path());
    cfg.sync_writes = false; // measure engine work, not disk sync
    let mut db = Hippocore::open(cfg).unwrap();
    db.create_tenant("t", "t").unwrap();
    db.create_collection("t", "c", "").unwrap();
    (dir, db)
}

fn populated() -> (tempfile::TempDir, Hippocore) {
    let (dir, mut db) = fresh_db();
    for i in 0..N {
        let req = RememberRequest::new(
            "t",
            "c",
            MemoryType::Note,
            format!("memory number {i} about oracle networking and listeners"),
        );
        db.remember(req).unwrap();
    }
    (dir, db)
}

fn bench_remember(c: &mut Criterion) {
    c.bench_function("remember", |b| {
        b.iter_batched(
            fresh_db,
            |(_dir, mut db)| {
                for i in 0..N {
                    let req = RememberRequest::new(
                        "t",
                        "c",
                        MemoryType::Note,
                        format!("memory number {i} about oracle"),
                    );
                    db.remember(req).unwrap();
                }
                black_box(db.stats().unwrap().memories)
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_recall_hybrid(c: &mut Criterion) {
    let (_dir, db) = populated();
    c.bench_function("recall_hybrid", |b| {
        b.iter(|| {
            let mut req = RecallRequest::new("t", "oracle listener networking");
            req.top_k = 10;
            black_box(db.recall(req).unwrap().len())
        });
    });
}

fn bench_search_vector(c: &mut Criterion) {
    let (_dir, db) = populated();
    c.bench_function("search_vector", |b| {
        b.iter(|| {
            let mut req = RecallRequest::new("t", "oracle listener networking");
            req.mode = SearchMode::Vector;
            req.top_k = 10;
            black_box(db.search(req).unwrap().len())
        });
    });
}

criterion_group!(
    benches,
    bench_remember,
    bench_recall_hybrid,
    bench_search_vector
);
criterion_main!(benches);
