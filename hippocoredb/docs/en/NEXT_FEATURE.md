# Next Feature

## Feature name

**Collection CRUD Tests v0.1** — deterministic tests for the full collection
lifecycle.

## Why it matters

Collections are the primary scoping unit for all items (memories, documents,
records). Their create/list/delete lifecycle and cross-tenant isolation are
exercised incidentally by many existing tests, but there is no focused suite
covering:

- Duplicate collection names within a tenant return an error.
- Collections from different tenants don't leak into each other's list.
- Deleting a collection removes all its items from the retrieval index.
- Querying a deleted collection's items after deletion returns empty results.

## Behaviour

No new production code. The test suite will cover:

1. Create, then list collections for a tenant — collection appears.
2. Creating a duplicate collection name within the same tenant returns an error.
3. Two tenants can have collections with the same name without interference.
4. After deleting a collection, `list_collections` no longer shows it.
5. After deleting a collection, recall scoped to that collection returns empty.

## Files

- `crates/hippocore/tests/collection_crud.rs` — new dedicated test file.
- `docs/en/COLLECTION_CRUD.md` and `docs/pt-br/COLLECTION_CRUD.md`.
- `docs/en/STATUS.md` and `docs/pt-br/STATUS.md`.

## Acceptance criteria

- At least 5 deterministic integration tests.
- All quality gates pass:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Out of scope

- Collection-level metadata or description updates.
- Server mode.
- Production code changes.
