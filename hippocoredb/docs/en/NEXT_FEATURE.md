# Next Feature

## Feature name

**Compact + Reopen Invariant Tests v0.1** — deterministic tests verifying that
`compact()` followed by a cold reopen produces exactly the same observable state.

## Why it matters

`compact()` folds all WAL entries into a snapshot and truncates the WAL to zero.
If this operation introduced any silent loss (e.g. truncating the snapshot before
the atomic rename completes, or re-indexing from an incomplete state), the
database would silently degrade without any test catching it. The compact +
reopen path is exercised incidentally but not explicitly locked.

## Behaviour

No new production code. The test suite will verify:

1. After `compact()`, `wal_len()` (or equivalent) reports zero entries.
2. A cold reopen after `compact()` produces the same `collections()` list.
3. A cold reopen after `compact()` returns the same memories in `recall()`.
4. Multiple compact → reopen cycles don't lose data.
5. Documents stored before `compact()` are retrievable after a cold reopen.

## Files

- `crates/hippocore/tests/compact_reopen.rs` — new dedicated test file.
- `docs/en/COMPACT_REOPEN.md` and `docs/pt-br/COMPACT_REOPEN.md`.
- `docs/en/STATUS.md` and `docs/pt-br/STATUS.md`.

## Acceptance criteria

- At least 5 deterministic integration tests using `TempDir`.
- All quality gates pass:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Out of scope

- WAL compaction strategy changes.
- Server mode.
- Production code changes.
