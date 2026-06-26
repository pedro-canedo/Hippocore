# Next Feature

## Feature name

**Confidence-Weighted Recall v0.1**

## Why it matters

Memories can carry an explicit confidence score set by the caller or the
human-in-the-loop `rate-memory` command, but the hybrid recall engine does not
use it. High-confidence memories that are semantically similar to the query can
be buried under lower-confidence but noisier matches. Factoring confidence into
the final ranking score improves RAG output quality measurably and without
changing the data model.

## Behavior

- After the hybrid score (vector + text fusion) is computed, multiply it by a
  confidence weight derived from each item's `confidence` field.
- Items with no confidence set (the common case) are treated as neutral weight
  (`1.0`) — no regressions for existing data.
- Weight formula: `final_score = hybrid_score * (1.0 + alpha * (confidence - 0.5))`
  where `alpha` is a tunable constant (start with `0.4`). This gives a ±20%
  boost/penalty at the extremes (0.0 and 1.0).
- Only `Memory` items carry a confidence score; `DocumentChunk` and `Record`
  items use neutral weight.
- The change lives entirely in the `query` module; `storage` and `lib.rs` are
  unchanged.
- The `RecallResult` struct gains no new fields; confidence is an internal
  ranking signal, not a returned metadata field.

## Likely files

- `crates/hippocore/src/query.rs`
- `crates/hippocore/tests/retrieval_quality.rs`
- `docs/en/SCORE_NORMALIZATION.md`
- `docs/pt-br/SCORE_NORMALIZATION.md`
- `docs/en/STATUS.md`
- `docs/pt-br/STATUS.md`
- `CHANGELOG.md`

## Acceptance criteria

- A high-confidence memory (`confidence=0.9`) that scores identically to a
  zero-confidence memory in hybrid recall appears above it in results.
- A zero-confidence memory (`confidence=0.0`) that scores identically to a
  neutral memory (no confidence set) appears below it.
- Items with no confidence set retain their original relative ranking.
- The retrieval quality fixture still passes all thresholds (MRR, hit@1,
  hit@k).
- `cargo fmt --all --check`, `cargo test --workspace`, and
  `cargo clippy --workspace --all-targets -- -D warnings` pass.

## Out of scope

- Confidence on `DocumentChunk` or `Record` items.
- Exposing `confidence` as a filter in `RecallRequest`.
- Making `alpha` a configurable server parameter.
- Changing the `RecallResult` struct.
- Any change to `storage`, `memory`, or `lib.rs`.
