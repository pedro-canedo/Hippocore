# Next Feature

## Feature name

**Matched Terms v0.1**

## Why it matters

When debugging recall quality, operators need to know which query tokens
actually appeared in each retrieved result. Currently the `reason` field
exposes score decomposition but not text coverage. Adding `matched_terms`
lets the LLM cite which terms drove the match and lets developers identify
whether low recall is due to vocabulary mismatch vs. vector distance.

## Behavior

- Add `matched_terms: Vec<String>` to `RecallResult` (serde default `[]`).
- Populated in `query.rs`: intersect the tokenized query with the tokenized
  result text. Only terms that appear in both sets are listed; duplicates
  are removed; empty when mode is pure vector (no query tokens).
- The field is purely informational; it never affects ranking or filtering.
- Existing callers that omit the field from their deserialization are unaffected
  thanks to `#[serde(default)]`.

## Likely files

- `crates/hippocore/src/model.rs` (RecallResult)
- `crates/hippocore/src/query.rs` (build_result / execute)
- `docs/en/STATUS.md`
- `docs/pt-br/STATUS.md`
- `CHANGELOG.md`

## Acceptance criteria

- `RecallResult` has `matched_terms: Vec<String>` defaulting to `[]`.
- For a hybrid or text recall, terms that appear in both query and result text
  are listed (lowercase, deduplicated).
- For a pure vector recall, `matched_terms` is empty.
- Existing serialized results that omit the field deserialize without error.
- `cargo fmt --all --check`, `cargo test --workspace`, and
  `cargo clippy --workspace --all-targets -- -D warnings` pass.

## Out of scope

- Highlighting offsets or character positions.
- Term weighting or IDF scores on the matched list.
- Passing matched_terms to the LLM as structured metadata (caller's choice).
- Any change to storage or index.
