# Next Feature

## Feature name

**WAL Recovery Tests v0.1** — deterministic tests verifying WAL torn-write
handling on database open.

## Why it matters

`CLAUDE.md` and `docs/en/ARCHITECTURE.md` document that "a torn trailing line
[in the WAL] is skipped, not fatal." This invariant is critical for data safety
(power loss after a partial write must not corrupt the database), but there are
no regression tests locking it down. A future refactor of the storage layer
could accidentally change recovery behavior without any test catching it.

## Behaviour

No new production code. The test suite will verify:

1. A fresh database with a clean WAL opens successfully.
2. A database whose WAL file has been truncated mid-record opens successfully
   (partial last record is skipped, not fatal).
3. After a torn-WAL open, operations that preceded the torn entry are intact in
   the recovered state.
4. A WAL with an invalid (corrupted) JSON record at the end opens without panic
   and skips the bad entry.

## Files

- `crates/hippocore/tests/wal_recovery.rs` — new dedicated test file.
- `docs/en/WAL_RECOVERY.md` and `docs/pt-br/WAL_RECOVERY.md`.
- `docs/en/STATUS.md` and `docs/pt-br/STATUS.md`.

## Acceptance criteria

- At least 4 deterministic integration tests using `TempDir` and direct WAL
  file manipulation (truncate / corrupt via `std::fs`).
- All quality gates pass:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Out of scope

- WAL compaction changes.
- Server mode.
- Production code changes.
