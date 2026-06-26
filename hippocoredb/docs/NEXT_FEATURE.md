# Next Feature

## Feature name

**Context data model v0.1** — define and implement the first structured data
surface that can become native context.

## Why it matters

Hippocore DB must be a database, not only a memory/vector retrieval library.
Users should be able to insert and manage different information shapes, and the
database should project those shapes into context for SDKs and agents.

Documents and memories already do this. The next step is to introduce a minimal
structured-data path without becoming SQL-first or pulling in server mode.

## Expected behavior

- Add a small first-class structured record model:
  - tenant,
  - collection,
  - table/name namespace,
  - id,
  - JSON object payload,
  - metadata,
  - source,
  - created/updated/version fields.
- Store records durably through WAL/snapshot/recovery.
- Project each record into searchable context text deterministically.
- Index record-derived context alongside memories and document chunks.
- Allow recall/search to return the record-derived context with kind/source
  information.
- Add CLI commands for basic management if scope remains small:
  - `put-record`,
  - `delete-record`,
  - `list-records` or `inspect --json` coverage.

## Affected modules / files

- `model.rs` — `Record` and item kind/type representation.
- `storage.rs` — record operations and state persistence.
- `index.rs` — project records into `IndexEntry`.
- `lib.rs` — public request/response APIs.
- `cli.rs` — minimal record commands or inspection path.
- `docs/DATA_MODEL.md`, `README.md`, `STATUS.md`, `CHANGELOG.md`.
- Tests for store/search/restart/delete/filter behavior.

## Acceptance criteria

- A JSON record can be inserted and survives reopen.
- Record fields are projected into searchable context.
- Recall can retrieve a record-derived context item.
- Metadata filters still apply.
- Tenant isolation holds for records.
- Deletes are durable and remove indexed record context.
- `cargo fmt --all --check`, `cargo test --workspace`, and clippy pass.

## Non-goals

- SQL engine.
- Joins.
- JDBC/ODBC/PostgreSQL wire protocol.
- Server mode.
- Complex schema management.
- Full admin UI.

## Follow-up

After this, improve CLI administration and evaluate a local Hippocore Studio
prototype once there are enough data types to manage visually.
