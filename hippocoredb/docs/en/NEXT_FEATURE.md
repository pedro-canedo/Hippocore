# Next Feature

## Feature name

**Database-like UX foundation: SQL command layer**

## Implemented slice

Hippocore now has a small SQL-like foundation for read-only structured record
queries:

- public `Hippocore::execute_sql(tenant_id, sql)`;
- typed `SqlCommand` and `SqlResult` API;
- `SELECT * FROM <table> WHERE <field> = <value> [AND ...] [LIMIT n]`;
- bare fields mapped to record payload keys;
- explicit `id`, `collection`, `table`, `payload.<key>`, and `metadata.<key>`;
- tenant-scoped execution;
- CLI command: `hippocore query --db ... --tenant ... --sql ... --json`;
- admin endpoint: `POST /admin/sql`;
- legacy `query-records` behavior preserved.

## Next recommended slice

Add a lightweight table/schema catalog over the existing `Record.table`
namespace. This should persist table metadata, expose list/create table flows,
and keep records JSON-first until SQL writes are justified.

## Out of scope for the SQL MVP

- Full SQL parsing or execution.
- Joins, projections, ordering, aggregates, and nested JSON predicates.
- `CREATE TABLE`, `INSERT`, `AI SEARCH`, and `IMPORT` SQL commands.
- Replacing the existing record CRUD API.
