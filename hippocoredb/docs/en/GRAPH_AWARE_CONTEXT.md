# Graph-Aware Context

Graph-Aware Context v0.1 lets callers opt into adding direct graph neighbours
when building an LLM context block.

This is intentionally not GraphRAG. It does not change recall ranking, does not
traverse multiple hops and does not boost scores. It only considers direct
`GraphEdge` neighbours of recalled items and includes them when they fit the
token budget.

## API

```rust
let mut req = BuildContextRequest::new("acme", "postgresql connection", 2048);
req.top_k_candidates = 5;
req.include_related = true;
req.related_limit = 4;

let block = db.build_context(req)?;
```

Defaults:

- `include_related = false`
- `related_limit = 8`

When disabled, `build_context` behaves as before.

## Provenance

`ContextItem` now exposes:

- `related_item_ids`
- `inclusion_source`

`inclusion_source` is `recalled` for normal recall hits and `graph_expanded` for
items added through direct graph expansion.

`AuditItem` uses the same source labels and also records `not_included` for
items that were considered but did not fit the final context.

## CLI

```bash
hippocore build-context \
  --db ./data \
  --tenant acme \
  --query "postgresql connection" \
  --top-k 5 \
  --include-related \
  --related-limit 4 \
  --json
```

JSON output includes `inclusion_source` for each included item.

## Limits

- Only direct neighbours are considered.
- Expansion is tenant-isolated.
- `related_limit` caps graph-expanded candidates.
- The same item is never duplicated when it was already recalled.
- Token budget still controls the final context.
