# RAG Audit Engine

Phase 9 adds a local audit trail for context assembly.

Every `Hippocore::build_context(req)` call appends one JSON object to
`<data_dir>/audit.log`. The record captures:

- `timestamp_ms`
- `tenant_id`
- original `query`
- recall `mode`
- optional `collection`
- requested `max_tokens`
- final `token_count`
- `latency_ms` for context compilation before audit persistence
- `items_dropped`
- retrieved item ids, kinds, collections, scores, confidence, token counts and
  whether each item was included in the final context
- `inclusion_source` (`recalled`, `graph_expanded`, or `not_included`)

The audit log is append-only operational history. It is intentionally separate
from `wal.log` because a query does not mutate the live database state.

## API

```rust
let block = db.build_context(req)?;
let records = db.query_audit(0, i64::MAX, "acme")?;
```

`query_audit(from_ms, to_ms, tenant_id)` returns records for a single tenant in
an inclusive epoch-millisecond range.

## CLI

```bash
hippocore build-context --db ./data --tenant acme --query "postgresql python" --json
hippocore audit --db ./data --tenant acme --from 0 --to 9223372036854775807 --json
```

Without `--json`, `audit` prints a compact human-readable summary.

## Confidence Feedback

The existing `rate_memory` API and `rate-memory` CLI command are part of this
audit loop. Confidence is stored durably on `Memory` and surfaced through
`RecallResult`, `ContextItem` and audit items.

## Non-Goals

- Automatic confidence calibration.
- Full provenance graph.
- HTTP/server audit ingestion.
- Audit retention or rotation policy.
