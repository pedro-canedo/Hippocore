# Next Feature

## Feature name

**Benchmark regression guard** — make latency regressions in recall visible and
actionable, completing Phase 5 of the roadmap.

## Why it matters

Hippocore DB already has a retrieval *quality* gate (hit@1/hit@k/MRR). It does
not yet have a retrieval *performance* gate. As the codebase evolves — new
retrieval modes, temporal filtering, richer scoring logic — it is easy to
introduce O(n) regressions that only show up in production. A committed
benchmark baseline with per-run comparison gives contributors immediate feedback
when a change regresses recall latency beyond a configurable threshold.

## Expected behavior

A new `cargo bench` target (`recall_regression` or extending `basic_bench`)
measures recall latency with a standard fixture size (e.g. 500 memories). The
benchmark saves results to a baseline file (`benches/baseline.json`). A
companion check command (`cargo xtask bench-check` or an integration test) reads
the baseline, runs the benchmark again, and fails if any measurement exceeds the
baseline by more than a threshold (e.g. 20%).

The baseline is committed to the repo, so CI will catch regressions. The
baseline can be updated explicitly (`cargo xtask update-baseline`).

### Key design choices

- Baseline stored as JSON with `{benchmark_name, mean_ns, std_ns, timestamp}`.
- Threshold configurable via env var `HIPPO_BENCH_THRESHOLD` (default 0.20 =
  20% slower is a failure).
- The check is a separate binary/script so it does not add build complexity to
  normal `cargo test`.
- Use the existing `criterion` setup; no new bench frameworks.

## Acceptance criteria

- `cargo bench -p hippocore` writes updated timing data.
- A baseline file is committed at `benches/baseline.json`.
- A regression check script (or xtask) reads the baseline and exits non-zero
  if any recall benchmark regresses beyond the threshold.
- Recall latency is measured at a realistic fixture size (≥ 200 memories).
- All 76+ tests still pass.
- `cargo fmt`, `cargo clippy -D warnings` clean.

## Non-goals

- Measuring P95/P99 latency (mean and stddev sufficient for a guard).
- CI infrastructure changes (the user's CI will call the existing `cargo bench`).
- Profiling or flame-graph integration.
- Server mode.

## Follow-up

After the benchmark guard, Phase 5 is complete. The next major phase is
**Phase 7 — Temporal Truth Layer full spec**: `supersedes` / `contradicts`
relations, conflict resolution, and richer temporal query semantics.
