# Score Normalization v0.1

## Overview

Score Normalization is an internal pre-processing step in `build_context` that
clamps raw hybrid recall scores to the `[0.0, 1.0]` range before temporal decay
and graph-aware ranking blends are applied. Without this step the blend weights
(`temporal_weight`, `graph_rank_weight`) have asymmetric effects when recall
scores exceed `1.0`, which can happen with certain query/corpus shapes.

This step is transparent to the caller and adds no new configuration fields.

## How it works

Immediately after the candidate list is built (recalled items plus optional
graph-expanded items), and only when at least one blend weight is non-zero:

1. Find `max_score = max(candidates.score)`.
2. If `max_score > 0`, divide every `candidate.score` by `max_score`.

The result: the highest-scoring candidate has a normalised score of exactly
`1.0`; all others are in `(0.0, 1.0]`. The relative ordering is unchanged.

When both blend weights are `0.0` (default) normalisation is skipped entirely —
`build_context` is identical to its pre-normalisation behaviour.

## Invariants

- Relative ordering within the candidate set is always preserved by
  normalisation.
- With a single candidate the normalised score is `1.0` (assuming a non-zero
  raw score).
- The normalisation is query-scoped; scores from different `build_context`
  calls are not comparable.
- This step only affects the internal blending path; scores returned to the
  caller via `ContextItem.score` reflect the post-blend effective score, not
  the raw recall score.

## Non-goals

- Normalisation of scores returned by `recall()` or `search()`.
- Cross-query score calibration.
- Server mode.
