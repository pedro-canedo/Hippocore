# Next Feature

## Feature name

**Graph-Aware Ranking v0.1** — boost recall scores using graph edge density.

## Why it matters

Graph-Aware Context v0.1 already expands the candidate set via direct graph
neighbours, and Audit Retention v0.1 bounds the audit log. The remaining gap
in retrieval quality is that items connected by many edges to other high-scoring
recalled items are not yet preferred over isolated high-scoring items. A
graph-aware ranking pass can lift genuinely connected context above coincidental
lexical matches.

## Behaviour

After the hybrid recall pass produces a ranked candidate list, apply a
graph-connectivity bonus:

- For each candidate, count how many of its direct neighbours are also in the
  candidate set.
- Scale the bonus by a configurable `graph_rank_weight` (default `0.1`).
- `effective_score = recall_score * (1 - graph_rank_weight) + connectivity_bonus * graph_rank_weight`.
- Items with zero neighbours in the candidate set are unaffected.

This keeps the ranking generic (no hard-coded entity or domain knowledge) and
respects the existing confidence/contradiction re-ranking already in
`build_context`.

## Files

- `crates/hippocore/src/lib.rs` — graph-aware re-ranking in `build_context` after `run_query`.
- `crates/hippocore/src/config.rs` — new `graph_rank_weight: f32` field (default `0.1`).
- `crates/hippocore/tests/database.rs` — deterministic tests.
- `docs/en/GRAPH_AWARE_RANKING.md` and `docs/pt-br/GRAPH_AWARE_RANKING.md`.
- `docs/en/STATUS.md` and `docs/pt-br/STATUS.md`.

## Acceptance criteria

- `recall_score` is unchanged when `graph_rank_weight = 0.0`.
- An item with more neighbours in the candidate set is ranked above an equally-
  scored item with none.
- `graph_rank_weight = 0.0` in config keeps the current ordering exactly.
- Setting `graph_rank_weight` out of range `[0.0, 1.0]` returns a typed
  validation error.
- At least 3 deterministic integration tests.
- All quality gates pass:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Out of scope

- Multi-hop traversal.
- Graph-based clustering or community detection.
- Server mode.
- Learning-to-rank / ML-based ranking.
