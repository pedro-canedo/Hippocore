# Graph Memory

Graph Memory v0.1 adds durable direct relationship edges between stored context
items. It is intentionally small: edges improve provenance and auditability, but
do not yet change recall ranking or perform GraphRAG traversal.

## Model

`GraphEdge` stores:

- `id`
- `tenant_id`
- `from_id` / `from_kind`
- `to_id` / `to_kind`
- `relation`
- `metadata`
- `created_at`
- `updated_at`

Supported endpoint kinds are `memory`, `record`, and `document_chunk`.

Endpoint validation is scoped to a tenant and uses `(tenant_id, kind, id)`.
Because the CLI intentionally avoids a graph query language in v0.1, endpoints
that match more than one item in a tenant are rejected as ambiguous.

## API

```rust
let edge = db.add_graph_edge(AddGraphEdgeRequest::new(
    "acme",
    ItemKind::Memory,
    "postgres-policy",
    ItemKind::Memory,
    "python-client",
    "mentions",
))?;

let edges = db.list_graph_edges("acme", Some("postgres-policy"));
let neighbours = db.graph_neighbors("acme", ItemKind::Memory, "postgres-policy");
db.delete_graph_edge("acme", &edge.id)?;
```

Edges are persisted through the same WAL/snapshot model as documents, memories,
records and files. Reopening the database restores them.

## CLI

```bash
hippocore add-edge \
  --db ./data \
  --tenant acme \
  --from-kind memory \
  --from-id postgres-policy \
  --to-kind memory \
  --to-id python-client \
  --relation mentions \
  --json

hippocore list-edges --db ./data --tenant acme --from-id postgres-policy --json
hippocore delete-edge --db ./data --tenant acme --id edge-123
```

## Context And Audit

`ContextItem` and `AuditItem` include `related_item_ids`, the ids of direct graph
neighbours known at context-build time.

Graph-Aware Context v0.1 can optionally include direct neighbours in the final
context with `BuildContextRequest::include_related`. This is still direct
expansion only; it does not boost recall scores or perform multi-hop traversal.

## Deletion Semantics

Deleting a memory or record removes edges touching that endpoint. Deleting a
document or imported file removes edges touching its derived document chunks.
Missing edge deletion is a safe no-op.

## Non-Goals

- Full GraphRAG traversal.
- Graph-aware ranking.
- Entity extraction.
- Pattern query language.
- Server mode.
