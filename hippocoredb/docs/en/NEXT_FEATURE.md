# Next Feature

## Feature name

**Admin CLI v0.1** — make database contents easier to inspect, list and export
without introducing server mode or a UI yet.

## Why it matters

Hippocore DB now stores documents, memories, structured records and imported
files. Before building a local Studio, users need a practical database
management surface similar in spirit to the basic workflows of pgAdmin/DBeaver:
see what exists, inspect objects, export data, and debug context.

## Expected behavior

- Add list commands for stored object types:
  - `list-tenants`;
  - `list-collections`;
  - `list-documents`;
  - `list-memories`;
  - `list-records`;
  - `list-files`.
- Add `inspect --json` or equivalent machine-readable output.
- Add object detail commands where useful, such as `show-record`,
  `show-document`, and `show-file`.
- Add export commands for JSON snapshots of selected objects or collections.
- Keep all commands local-first and embedded; no server mode.
- Preserve tenant isolation and metadata visibility.

## Affected modules / files

- `crates/hippocore/src/lib.rs` — public read/list APIs where missing.
- `crates/hippocore/src/cli.rs` — admin commands and JSON output.
- `crates/hippocore-cli/tests/cli.rs` — subprocess coverage for admin flows.
- `docs/en/ADMIN_INTERFACE.md` and `docs/pt-br/ADMIN_INTERFACE.md`.
- `README.md`, `STATUS.md`, `CHANGELOG.md`.

## Acceptance criteria

- A user can list tenants, collections and each stored object type.
- A user can inspect a specific record/file/document enough to understand what
  was stored and what context projection exists.
- JSON output is stable enough for scripts and future Studio integration.
- No server process is required.
- `cargo fmt --all --check`, `cargo test --workspace`, and clippy pass.

## Non-goals

- Web UI.
- PostgreSQL wire protocol.
- SQL query language.
- Authentication/authorization.
- Remote server mode.

## Follow-up

After Admin CLI v0.1, revisit a local Hippocore Studio prototype and add a
small retrieval-quality evaluation command powered by the existing fixture
format.
