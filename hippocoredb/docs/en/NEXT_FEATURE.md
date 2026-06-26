# Next Feature

## Feature name

**Admin CLI v0.1**

## Why it matters

The Control Plane and the REST API now cover the main data-ingestion flows.
The next highest-value slice is to add CLI sub-commands that mirror those flows
so operators can script and automate without running an HTTP server. The existing
`hippocore-cli` binary already has `stats` and `bench`; it needs data-plane
commands.

## Behavior

- `hippocore tenants list` — print all tenants in a data directory.
- `hippocore tenants create <id> [--name <n>]` — create a tenant.
- `hippocore collections list --tenant <tid>` — list collections for a tenant.
- `hippocore collections create --tenant <tid> <name>` — create a collection.
- `hippocore memories add --tenant <tid> --collection <col> <text>` — remember.
- `hippocore records put --tenant <tid> --collection <col> --table <tbl>
  [--payload <json>|--file <path>]` — put a structured JSON record.
- `hippocore recall --tenant <tid> --collection <col> <query>` — hybrid recall.
- All commands read from the data directory (`--db` flag, default
  `./hippocore-data`).
- Output is plain text or JSON (`--json` flag).

## Likely files

- `crates/hippocore-cli/src/main.rs`
- `crates/hippocore/src/cli.rs`
- `crates/hippocore/tests/record_crud.rs`
- `docs/en/STATUS.md`
- `docs/pt-br/STATUS.md`
- `CHANGELOG.md`

## Acceptance criteria

- All listed sub-commands are implemented and respond to `--help`.
- `hippocore tenants list` lists tenants after `tenants create`.
- `hippocore records put` creates a record readable via `hippocore sql`.
- `hippocore recall` returns text results.
- Tenant isolation: commands fail with a clear error if the tenant does not
  exist.
- `cargo fmt --all --check`, `cargo test --workspace`, and
  `cargo clippy --workspace --all-targets -- -D warnings` pass.

## Out of scope

- Interactive TUI.
- Shell completion.
- Streaming output.
- CSV import.
- Graph traversal CLI.
- HTTP server sub-commands (those live in `hippocore serve`).
