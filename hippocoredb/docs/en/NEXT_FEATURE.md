# Next Feature

## Feature name

**Hybrid Search Ranking Tests v0.1** — deterministic tests verifying that
`SearchMode::Hybrid` blends vector and text scores correctly.

## Why it matters

The hybrid search path fuses TF-IDF text scores with cosine vector scores. If
the fusion weight or normalisation is wrong, either purely textual matches or
purely semantic matches can be silently suppressed. No test currently locks the
relative ordering guarantee: a memory that matches both text and vector should
rank above one that matches only text.

## Behaviour

No new production code. The test suite will verify:

1. `SearchMode::Vector` returns a semantically-close memory that doesn't share
   exact keywords with the query.
2. `SearchMode::Text` returns a memory that shares exact words with the query.
3. `SearchMode::Hybrid` returns both kinds; the result set is a superset of
   `Vector`-only and `Text`-only hits when those hits exist.
4. A memory that exactly matches the query appears in the top 3 results under
   `Hybrid` mode.
5. Changing from `Hybrid` to `Vector` does not return a result whose text is
   exactly the query but semantically unrelated.

## Files

- `crates/hippocore/tests/hybrid_search.rs` — new dedicated test file.
- `docs/en/HYBRID_SEARCH.md` and `docs/pt-br/HYBRID_SEARCH.md`.
- `docs/en/STATUS.md` and `docs/pt-br/STATUS.md`.

## Acceptance criteria

- At least 5 deterministic integration tests using `TempDir`.
- All quality gates pass:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Out of scope

- Tuning fusion weights.
- Server mode.
- Production code changes.
