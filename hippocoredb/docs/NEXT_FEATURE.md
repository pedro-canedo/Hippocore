# Next Feature

## Feature name

**Benchmark regression guard v0.1** — lightweight performance budgets for the
current exact retrieval path.

## Why it matters

Criterion is useful during development, but today regressions are discovered by
manually reading `cargo bench` output and comparing against a local baseline.
Retrieval quality changes can accidentally add work to hot paths, especially
pure vector search. A small guard gives the project a repeatable signal before
future ranking changes land.

## Expected behavior

- Add a deterministic performance smoke test or bench helper that measures:
  - `remember` for a fixed number of memories,
  - `recall_hybrid` over a fixed in-memory corpus,
  - `search_vector` over the same corpus.
- Keep it separate from Criterion's historical baseline files. The guard should
  compare against explicit, documented budgets for this MVP and print measured
  timings when it fails.
- Ensure pure `SearchMode::Vector` does not run lexical normalization, BM25, or
  entity detection work.
- Keep default CI/runtime practical; if the guard is too noisy for regular
  `cargo test`, make it an explicit ignored test with a documented command.

## Affected modules / files

- `benches/basic_bench.rs` — keep Criterion for developer trend analysis.
- `crates/hippocore/tests/` — optional budget smoke test if stable enough.
- `docs/STATUS.md`, `CHANGELOG.md` — update with measured budget and command.

## Acceptance criteria

- Running the documented command reports timings for `remember`,
  `recall_hybrid`, and `search_vector`.
- The guard catches a clear vector-search regression caused by accidental text
  or entity work in the vector-only path.
- The current RAG Quality Layer remains within the documented budgets.
- `cargo fmt`, `cargo test --workspace`, and clippy remain green.

## Risks / open questions

- Wall-clock timing is noisy across machines. Prefer broad budgets and clear
  diagnostics over fragile microsecond thresholds.
- Avoid conflating Criterion local baseline drift with true code regression.
