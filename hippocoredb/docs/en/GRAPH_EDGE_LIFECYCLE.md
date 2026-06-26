# Graph Edge Lifecycle Tests v0.1

## Overview

A dedicated test suite verifying the full graph edge lifecycle: add, list,
relation type storage, delete, durability through compact + reopen, and tenant
isolation. No production code was changed; this feature is purely additive tests.

## Test file

`crates/hippocore/tests/graph_edge_lifecycle.rs`

## What is tested

| Test | Scenario |
|---|---|
| `added_edge_appears_in_list` | `add_graph_edge` → `list_graph_edges` returns the edge |
| `relation_type_is_stored` | Relation string round-trips through storage |
| `deleted_edge_absent_from_list` | `delete_graph_edge` removes edge from `list_graph_edges` |
| `edges_survive_compact_reopen` | Edge present after `compact()` + cold reopen |
| `graph_edges_are_tenant_isolated` | `list_graph_edges("beta")` returns empty when only "alpha" has edges |

## Edge API summary

- `add_graph_edge(AddGraphEdgeRequest)` — links two items (any `ItemKind`) with
  a `relation` string.
- `list_graph_edges(tenant_id, from_id)` — returns edges for a tenant, optionally
  filtered by `from_id`. Returns `Vec<GraphEdge>` directly (not a `Result`).
- `delete_graph_edge(tenant_id, edge_id)` — removes the edge durably.
