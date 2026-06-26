# Next Feature

## Feature name

**Metadata Filter Regression Suite v0.1** — deterministic tests covering exact
and prefix metadata filter combinations across all item kinds.

## Why it matters

Metadata filtering is the primary mechanism for scoping recall to a user,
session, or context. While the filter logic exists, there is no focused test
suite that covers all item kinds × filter combinations × edge cases. A gap here
risks silent regressions when the query or storage layers change.

## Behaviour

No new production code. The test suite will cover:

1. Exact-match filter (`key == value`) on memories, document chunks, and records.
2. Multi-key filters (all keys must match, i.e. AND semantics).
3. A filter that matches zero items (empty result, not an error).
4. A filter scoped to a specific collection within a tenant.
5. Verify that filtering on one tenant does not affect another tenant with the
   same metadata keys and values.
6. `build_context` with a metadata filter respects the filter (no leakage).

## Files

- `crates/hippocore/tests/metadata_filter.rs` — new dedicated test file.
- `docs/en/METADATA_FILTER_REGRESSION.md` and
  `docs/pt-br/METADATA_FILTER_REGRESSION.md`.
- `docs/en/STATUS.md` and `docs/pt-br/STATUS.md`.

## Acceptance criteria

- At least 6 deterministic integration tests.
- All quality gates pass:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Out of scope

- New filter operators (prefix, range, regex).
- Server mode.
- Production code changes.
