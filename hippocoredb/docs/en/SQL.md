# SQL-like Record Queries

Hippocore has a small, typed SQL-like command layer for structured records.
It is intentionally not a full SQL engine.

## Supported syntax

```sql
SELECT * FROM <table>
WHERE <field> = <value> [AND ...]
LIMIT <n>;
```

Examples:

```bash
hippocore query --db ./data --tenant acme \
  --sql "select * from systems where engine = 'postgresql' limit 20" \
  --json
```

```sql
SELECT * FROM systems
WHERE engine = 'postgresql'
  AND metadata.tier = 'database'
LIMIT 5;
```

## Semantics

- Queries are always scoped to the supplied tenant.
- `FROM <table>` maps to `Record.table`.
- Bare fields such as `engine` map to `payload.engine`.
- Explicit fields supported in `WHERE`:
  - `id`
  - `collection`
  - `table`
  - `payload.<key>`
  - `metadata.<key>`
- Conditions are exact equality only.
- Values may be quoted strings or simple identifiers.
- Default limit is `100`; maximum limit is capped at `500`.

## Compatibility

The older restricted form still works:

```sql
select * from records where table = 'systems' and payload.engine = 'postgresql'
```

Use `hippocore query-records` or `/admin/query-records` for the legacy response
shape. Use `hippocore query` or `/admin/sql` for the new `SqlResult` response.

## Out of scope

No joins, ordering, projections, aggregates, nested JSON predicates, writes, or
SQL imports are implemented yet.
