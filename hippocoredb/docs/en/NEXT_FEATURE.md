# Next Feature

## Feature name

**Graph Memory v0.1** — durable relationship edges between context items.

## Why it matters

The RAG Audit Engine can now explain which items were retrieved for a context
block, but Hippocore still treats memories, document chunks and records as a
flat set. Agents often need to know that one item explains, depends on,
mentions, conflicts with or is related to another item. A small graph layer
improves context reliability without jumping to full GraphRAG.

Graph Memory v0.1 should add:

- A typed `GraphEdge` model with `tenant_id`, `from_id`, `from_kind`, `to_id`,
  `to_kind`, `relation`, metadata and timestamps.
- Durable APIs to add, list and delete edges.
- Tenant-scoped graph traversal helpers for direct neighbours only.
- CLI commands to add/list/delete graph edges with `--json` output.
- Context compiler awareness that can optionally include directly related
  neighbour ids in provenance/audit output.

## Acceptance criteria

- `GraphEdge` is persisted through the existing WAL/snapshot model and survives
  reopen.
- Edge APIs validate tenant isolation and reject unknown endpoint items.
- Deleting an item removes or hides dangling edges deterministically.
- CLI commands work:
  - `add-edge --tenant <t> --from-kind <kind> --from-id <id> --to-kind <kind>
    --to-id <id> --relation <name>`
  - `list-edges --tenant <t> [--from-id <id>] [--json]`
  - `delete-edge --tenant <t> --id <edge_id>`
- At least 4 deterministic tests: add/list happy path, unknown endpoint error,
  tenant isolation, restart durability.
- Documentation is updated in English and Portuguese.
- All quality gates pass:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Non-goals

- Full GraphRAG traversal or ranking.
- Query language for graph patterns.
- Entity extraction.
- Server/HTTP mode.
- Distributed graph storage.

## Follow-up

After Graph Memory v0.1, decide whether the next highest-value step is graph-aware
recall ranking or audit retention/rotation.
