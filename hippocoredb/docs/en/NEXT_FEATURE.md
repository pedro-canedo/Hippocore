# Next Feature

## Feature name

**Control Plane TS — SQL Editor v0.4**

## Why it matters

The Data Explorer browses records by collection; the next step is querying them.
Porting the read-only SQL Editor to `/console` gives operators the existing
`POST /admin/sql` power (tenant-scoped `SELECT` over record payloads) inside the
new typed console, completing the core "see and query your data" loop.

## Behavior

- Add a **SQL Editor** page to the sidebar, gated on an active tenant.
- A query textarea with a Run button and `Ctrl/Cmd + Enter` to execute against
  `POST /admin/sql` with `{ tenant_id, sql }`.
- Render successful results as a table with deterministic payload columns and a
  Raw JSON toggle; show the client-observed duration and row count.
- Surface structured validation/parse errors from the typed `ApiError`.
- Keep the current query in view state; example snippets populate the editor
  without auto-running. Briefly explain that unprefixed fields map to
  `payload.<field>`.

## Likely files

- `crates/hippocore-server/admin-ui/src/api.ts` (add `runSql`)
- `crates/hippocore-server/admin-ui/src/views/SqlEditorView.tsx` (new)
- `crates/hippocore-server/admin-ui/src/App.tsx`, `views.ts`, `styles.css`
- Rebuilt assets in `crates/hippocore-server/src/console/`
- `crates/hippocore-server/tests/server_integration.rs`
- Bilingual status/CHANGELOG and `CONTROL_PLANE_TS.md`

## Acceptance criteria

- Run button and `Ctrl/Cmd + Enter` execute the same tenant-scoped request.
- Results show deterministic payload columns and a full JSON view; duration and
  row count appear.
- Parse/validation errors render inline without crashing the view.
- `pnpm type-check` and `pnpm build` pass; the Rust gate is green.
- New user-facing copy exists in English and Portuguese.

## Tests

- `/console` assets stay public (existing integration test).
- Existing `/admin/sql` endpoint tests stay green.
- `tsc --noEmit` strict passes; build updates `src/console/`.

## Out of scope

- Expanding the SQL grammar or adding writes.
- Query history, saved queries, pagination, or CSV export.
- Porting Ingestion & Recall, Integrations, Prompts, Observability.
