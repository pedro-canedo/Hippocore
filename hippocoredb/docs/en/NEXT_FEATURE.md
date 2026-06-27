# Next Feature

## Feature name

**Control Plane TS — Data Explorer (read) v0.3**

## Why it matters

With tenants and collections manageable in `/console`, the next highest-value
step is seeing the data inside them. A read-only Data Explorer lets operators
browse Records, Memories, Documents, and Files for the active tenant and a
chosen collection, establishing the entity-list pattern every later management
page reuses.

## Behavior

- Add a **Data Explorer** page to the sidebar, gated on an active tenant.
- A collection selector (populated from `GET /admin/collections?tenant_id=`)
  chooses the scope; remember the last collection per tenant locally.
- Entity-type tabs — Records, Memories, Documents, Files — load in parallel for
  the active collection and show per-type counts on the tabs.
- Records render their payload keys as deterministic columns (id and table stay
  as stable system columns); other types show their core fields. A detail
  drawer shows the full JSON of a selected row.
- Empty states guide the user to create a collection or ingest first data via
  the classic console while ingestion is not yet ported.

## Likely files

- `crates/hippocore-server/admin-ui/src/api.ts` (list records/memories/
  documents/files; reuse existing `/admin/{records,memories,documents,files}`)
- `crates/hippocore-server/admin-ui/src/views/DataExplorerView.tsx` (new)
- `crates/hippocore-server/admin-ui/src/components/Drawer.tsx` (new)
- `crates/hippocore-server/admin-ui/src/App.tsx`, `views.ts`, `styles.css`
- Rebuilt assets in `crates/hippocore-server/src/console/`
- `crates/hippocore-server/tests/server_integration.rs`
- Bilingual status/CHANGELOG and `CONTROL_PLANE_TS.md`

## Acceptance criteria

- Selecting a tenant and collection lists each type with correct counts.
- Record payload keys appear as deterministic columns; the drawer shows full
  JSON; tenant isolation is respected (only active tenant's data shows).
- The chosen collection is remembered per tenant across reloads.
- `pnpm type-check` and `pnpm build` pass; the Rust gate is green.
- New user-facing copy exists in English and Portuguese.

## Tests

- `/console` assets stay public (existing integration test).
- Existing server list endpoints stay green.
- `tsc --noEmit` strict passes; build updates `src/console/`.

## Out of scope

- Creating/editing/deleting entities from the Explorer (read-only for now).
- Porting SQL Editor, Ingestion & Recall, Integrations, Prompts, Observability.
- Pagination, server-side search, or CSV export.
