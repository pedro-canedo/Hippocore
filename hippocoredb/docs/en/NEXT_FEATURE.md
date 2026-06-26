# Next Feature

## Feature name

**Score Normalization v0.1** — normalize recall scores and blending inputs to
a consistent `[0.0, 1.0]` range before temporal decay and graph-aware ranking.

## Why it matters

`temporal_weight` and `graph_rank_weight` assume that both operands of the blend
are in `[0.0, 1.0]`. The decay and connectivity scores are already normalised, but
the raw recall score (a hybrid of cosine + TF-IDF) can exceed `1.0` depending on
query or corpus shape, making the blend asymmetric. Without normalisation, the
weight parameters do not have intuitive, corpus-independent semantics.

## Behaviour

Before the temporal decay and graph-aware ranking blends, clamp/normalise the raw
`recall_score` of each candidate:

- Collect all scores from the candidate set.
- `max_score = max(scores)`.
- `normalised = score / max_score` (or `0.0` when max is `0`).
- Proceed with `normalised` as the base in all blend formulas.

The normalisation is query-scoped (relative to the current candidate set), so
absolute score values across queries remain incomparable — this is intentional.

## Files

- `crates/hippocore/src/lib.rs` — normalisation pass in `build_context` before
  temporal decay and graph-aware ranking.
- `crates/hippocore/tests/database.rs` — deterministic tests.
- `docs/en/SCORE_NORMALIZATION.md` and `docs/pt-br/SCORE_NORMALIZATION.md`.
- `docs/en/STATUS.md` and `docs/pt-br/STATUS.md`.

## Acceptance criteria

- All candidate scores in the blended path are in `[0.0, 1.0]`.
- Normalisation does not change the relative ordering when no blending is active
  (all weights `= 0`).
- When a single candidate is returned, it receives a score of `1.0`.
- At least 2 deterministic integration tests.
- All quality gates pass:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Out of scope

- Normalisation of recall scores returned by `recall()` or `search()` (only
  affects `build_context` blending).
- Cross-query score persistence or calibration.
- Server mode.
