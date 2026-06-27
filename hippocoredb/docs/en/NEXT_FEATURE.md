# Next Feature

## Feature name

**Control Plane TS — Ingestion v0.5**

## Why it matters

The TypeScript console can now browse and query data but cannot create any. The
next highest-value step is ingestion: creating Memories, Records, and Documents
in the active tenant and collection, so `/console` becomes self-sufficient for
the core write loop without falling back to `/admin`.

## Behavior

- Add an **Ingestion** page to the sidebar, gated on an active tenant and a
  collection selector (reusing the per-tenant collection memory).
- Three typed forms:
  - **Memory**: text, memory type, optional confidence → `POST
    /admin/tenants/{tid}/memories`.
  - **Record**: table name + JSON payload (validated client-side) → `POST
    /admin/tenants/{tid}/records`.
  - **Document**: text (and optional title/metadata) → `POST
    /admin/tenants/{tid}/documents`.
- Each successful create clears the form, shows a toast, and refreshes the
  Dashboard counts; typed `ApiError` renders inline.
- Invalid JSON in the Record payload is caught before sending.

## Likely files

- `crates/hippocore-server/admin-ui/src/api.ts` (add ingest calls)
- `crates/hippocore-server/admin-ui/src/views/IngestionView.tsx` (new)
- `crates/hippocore-server/admin-ui/src/App.tsx`, `views.ts`, `styles.css`
- Rebuilt assets in `crates/hippocore-server/src/console/`
- `crates/hippocore-server/tests/server_integration.rs`
- Bilingual status/CHANGELOG and `CONTROL_PLANE_TS.md`

## Acceptance criteria

- Creating a Memory, Record, and Document persists to the active tenant and
  collection and appears in the Data Explorer.
- Invalid Record JSON is rejected client-side with a clear message.
- Dashboard counts update after each create.
- `pnpm type-check` and `pnpm build` pass; the Rust gate is green.
- New user-facing copy exists in English and Portuguese.

## Tests

- `/console` assets stay public (existing integration test).
- Existing ingestion endpoint tests stay green.
- `tsc --noEmit` strict passes; build updates `src/console/`.

## Out of scope

- File upload / drag-and-drop (a later increment) and Recall.
- Editing or deleting existing entities.
- Porting Integrations, Prompts, or Observability.
