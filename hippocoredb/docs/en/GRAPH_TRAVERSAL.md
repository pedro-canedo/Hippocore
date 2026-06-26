# GraphRAG Multi-hop Traversal — Phase 10 completion

## Overview

`traverse_graph` implements multi-hop BFS graph traversal (GraphRAG-style),
completing Phase 10. Starting from a set of seed items (typically the top
recall hits), the traversal follows `GraphEdge` links breadth-first up to
`max_hops` deep, collecting at most `max_nodes` discovered items.

## New public API

### `TraverseGraphRequest`

| Field | Type | Default | Description |
|---|---|---|---|
| `tenant_id` | `String` | — | Tenant scope (mandatory) |
| `seed_ids` | `Vec<(ItemKind, String)>` | — | BFS starting points |
| `max_hops` | `usize` | `2` | Maximum depth |
| `max_nodes` | `usize` | `50` | Hard cap on results |
| `relation_filter` | `Option<String>` | `None` | Only follow edges with this `relation` |

### `TraversalNode`

| Field | Type | Description |
|---|---|---|
| `id` | `String` | Id of the discovered item |
| `kind` | `ItemKind` | Memory / DocumentChunk / Record |
| `hop` | `usize` | Hop distance from nearest seed |
| `via_edge_id` | `String` | Graph edge that introduced this node |

### `Hippocore::traverse_graph(req: TraverseGraphRequest) -> Vec<TraversalNode>`

Returns nodes in BFS order (all hop-1 nodes before any hop-2 nodes).
Seed items are **not** included in the result set.

## Test file

`crates/hippocore/tests/graph_traversal.rs` — 6 tests covering 1-hop
discovery, hop distance, `max_hops=1` boundary, relation filter, `max_nodes`
cap, and tenant isolation.

## Relationship to `include_related`

`build_context`'s `include_related` is 1-hop expansion baked into the context
assembly path. `traverse_graph` is a standalone multi-hop API for callers who
want to explore the graph without the token-budget assembly step.
