# Next Feature

## Feature name

**Record CRUD Tests v0.1** — deterministic tests for the Record entity
lifecycle.

## Why it matters

`Record` is a first-class entity alongside `Memory` and `Document`, but has no
dedicated test file. Records support versioning (incrementing version on update),
deletion, and metadata filtering. These paths are exercised incidentally in
other tests but not locked as explicit regressions.

## Behaviour

No new production code. The test suite will cover:

1. Store a record and retrieve it by id.
2. Recall returns the stored record.
3. Update (re-store same id) increments the version field.
4. Delete a record; subsequent retrieval returns `None`.
5. Delete a record; subsequent recall does not return it.
6. Records from one tenant are not visible to another tenant.

## Files

- `crates/hippocore/tests/record_crud.rs` — new dedicated test file.
- `docs/en/RECORD_CRUD.md` and `docs/pt-br/RECORD_CRUD.md`.
- `docs/en/STATUS.md` and `docs/pt-br/STATUS.md`.

## Acceptance criteria

- At least 6 deterministic integration tests using `TempDir`.
- All quality gates pass:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Out of scope

- Record batch operations.
- Server mode.
- Production code changes.
