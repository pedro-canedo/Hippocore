# Next Feature

## Feature name

**Data Explorer Workspace v0.1**

## Why it matters

Data Explorer has logical, JSON, and physical tabs, but still behaves like an
empty records page. A developer should be able to select a tenant, navigate its
collections and data types, inspect structured payload fields as columns, and
open full object details without first understanding Hippocore internals.

## Behavior

- Add collection search and a contextual create-collection action to the tree.
- Show Records, Memories, Documents, and Files under the selected collection,
  with counts derived from the loaded tenant-scoped lists.
- Selecting a data type loads and displays that type without leaving Explorer.
- Render record payload keys as logical table columns while keeping stable
  object identifiers and table names visible.
- Keep Raw JSON for complete persisted objects and Physical Info for lineage,
  kind, timestamps, source, metadata, document/chunk references, and version.
- Empty states guide users to create a collection or ingest the first item.

## Likely files

- `crates/hippocore-server/src/admin/app.js`
- `crates/hippocore-server/src/admin/styles.css`
- `crates/hippocore-server/tests/server_integration.rs`
- bilingual Control Plane status/interface docs and `CHANGELOG.md`

## Acceptance criteria

- Explorer navigates tenant -> collection -> data type from one screen.
- Collection search filters the tree without changing tenant scope.
- Record payload fields appear as logical columns with deterministic ordering.
- JSON and physical views expose complete selected data without mixing storage
  details into the default logical table.
- All new copy is complete in English and Portuguese.
- Existing auth and tenant isolation remain unchanged; quality gate is green.

## Tests

- Asset assertions cover collection search, type selection, payload columns,
  and the three explorer views.
- Existing tenant-isolation and admin-list integration tests remain green.
- JavaScript syntax check and complete Rust quality gate pass.

## Out of scope

- Editing or deleting arbitrary objects from the Explorer table.
- New backend aggregation/count endpoints.
- Virtualized tables, schema migrations, or React/Vite migration.
