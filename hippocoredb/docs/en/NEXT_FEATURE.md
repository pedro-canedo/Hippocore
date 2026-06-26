# Next Feature

## Feature name

**Graph Edge Lifecycle Tests v0.1** — deterministic tests for the graph edge
lifecycle.

## Why it matters

Graph edges are used in `build_context` for the graph-aware ranking feature.
The edge APIs (`add_graph_edge`, `list_graph_edges`, `remove_graph_edge`) are
exercised incidentally in `tenant_isolation.rs` and `database.rs`, but no
dedicated test suite verifies the full lifecycle: add, list, relation types,
durability through compact + reopen, and tenant isolation.

## Behaviour

No new production code. The test suite will cover:

1. `add_graph_edge` between two memories; `list_graph_edges` returns it.
2. Adding an edge with a `relation` type string; the relation survives.
3. `remove_graph_edge` removes the edge from the list.
4. Edges survive `compact()` + cold reopen.
5. `list_graph_edges` for one tenant does not return edges from another tenant.

## Files

- `crates/hippocore/tests/graph_edge_lifecycle.rs` — new dedicated test file.
- `docs/en/GRAPH_EDGE_LIFECYCLE.md` and `docs/pt-br/GRAPH_EDGE_LIFECYCLE.md`.
- `docs/en/STATUS.md` and `docs/pt-br/STATUS.md`.

## Acceptance criteria

- At least 5 deterministic integration tests using `TempDir`.
- All quality gates pass:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Out of scope

- Graph traversal algorithms.
- Server mode.
- Production code changes.
