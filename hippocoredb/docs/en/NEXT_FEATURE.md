# Next Feature

## Feature name

**Temporal Decay v0.1** — apply a recency bias to recall scores.

## Why it matters

Graph-Aware Ranking v0.1 rewards connectivity; the remaining gap is staleness.
A memory about "the staging config" stored six months ago should score lower
than an equivalent memory added yesterday, because recent context is more likely
to be accurate and actionable. Without recency weighting, semantic similarity
alone determines rank — a stale but highly similar memory can displace a
fresher, slightly less similar one.

## Behaviour

After hybrid recall produces scored candidates, apply a time-decay multiplier
before graph-aware re-ranking and the token-budget pass:

- `age_seconds = now_ms - item.updated_at_ms / 1000`.
- `decay = exp(-decay_rate × age_seconds / 86400)` (exponential, per day).
- `effective_score = recall_score * (1 - temporal_weight) + decay * temporal_weight`.

When `temporal_weight = 0.0` (default) the multiplier is a no-op and ordering
is unchanged. When `temporal_weight = 1.0` only recency matters.

## Files

- `crates/hippocore/src/lib.rs` — temporal decay blend in `build_context`
  after `run_query` and before graph-aware ranking.
- `crates/hippocore/src/config.rs` — new `temporal_weight: f32` (default `0.0`)
  and `temporal_decay_days: f32` (default `30.0`).
- `crates/hippocore/tests/database.rs` — deterministic tests.
- `docs/en/TEMPORAL_DECAY.md` and `docs/pt-br/TEMPORAL_DECAY.md`.
- `docs/en/STATUS.md` and `docs/pt-br/STATUS.md`.

## Acceptance criteria

- `temporal_weight = 0.0` keeps the exact same ordering as plain recall.
- A memory updated recently ranks above an otherwise identical (content,
  vector) memory that is older when `temporal_weight > 0`.
- `temporal_weight` or `temporal_decay_days` outside valid range returns a
  typed validation error.
- At least 3 deterministic integration tests.
- All quality gates pass:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Out of scope

- Periodic background reindexing based on age.
- Server mode.
- Automatic eviction of stale memories.
- Multi-hop graph traversal.
