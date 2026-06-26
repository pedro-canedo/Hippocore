# Next Feature

## Feature name

**RRF Hybrid Fusion** — replace the current min-max normalization + alpha linear
fusion for hybrid search with Reciprocal Rank Fusion (RRF).

## Why it matters

The current hybrid mode normalizes vector and BM25 scores independently with
min-max and combines them as `alpha * vector_norm + (1-alpha) * text_norm`. This
approach has two known weaknesses:

1. **Score scale sensitivity**: min-max normalization collapses a gap of 0.9
   between ranks 1 and 2 to the same width as a gap of 0.001, making adjacent
   ranks indistinguishable when one score dominates.
2. **Manual alpha**: `hybrid_alpha` requires tuning per use-case. A fixed default
   of 0.5 is arbitrary.

Reciprocal Rank Fusion is parameter-free, rank-based, and well-studied:

```
RRF(d) = 1 / (k + rank_vector(d))  +  1 / (k + rank_text(d))
```

where `k = 60` is the standard smoothing constant, `rank_vector(d)` is the
1-based rank of document `d` in the vector-sorted list, and `rank_text(d)` is
its rank in the BM25-sorted list (unranked items receive a penalty rank beyond
the candidate set).

## Expected behavior

- `SearchMode::Hybrid` uses RRF internally instead of alpha-weighted
  normalization.
- `hybrid_alpha` in `Config` becomes unused/deprecated for hybrid mode; it is
  still accepted without error for backwards compatibility but has no effect.
- `RecallResult.vector_score` and `text_score` continue to carry the raw
  (unnormalized) cosine and BM25 scores for transparency.
- `RecallResult.reason` for hybrid mode reports the RRF score:
  `hybrid/rrf(vector_rank=N, text_rank=M)`.
- `SearchMode::Vector` and `SearchMode::Text` are unchanged.
- All existing tests must pass; the retrieval quality fixture thresholds must
  remain met or improve.

## Affected modules / files

- `crates/hippocore/src/query.rs` — replace fusion logic with RRF.
- `crates/hippocore/tests/database.rs` — verify hybrid mode tests still pass
  (behavior-only assertions should be stable; score assertions may need review).
- `crates/hippocore/tests/retrieval_quality.rs` — run fixture; verify thresholds.
- `docs/en/STATUS.md` and `docs/pt-br/STATUS.md`.
- `CHANGELOG.md`, `ROADMAP.md`.

## Acceptance criteria

- `SearchMode::Hybrid` uses RRF.
- The reason string includes "rrf" in hybrid mode.
- Existing retrieval quality thresholds (hit@1, hit@5, MRR) are met.
- All 67+ tests pass.
- `cargo fmt`, `cargo clippy -D warnings` clean.
- No new dependencies.

## Non-goals

- Changing `SearchMode::Vector` or `SearchMode::Text`.
- Removing `hybrid_alpha` from the public API (keep for backwards compat,
  silently unused).
- HNSW, ANN, server mode.

## Follow-up

After RRF, the next step is the `eval-quality` CLI command to let operators run
retrieval fixture evaluations against any deployed database.
