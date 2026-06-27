# Next Feature

## Feature name

**Control Plane TS — File Upload (drag-and-drop) v0.7**

## Why it matters

Ingestion in `/console` covers Memory, Record, and Document, but not files. The
classic console has drag-and-drop upload (PDF/CSV/TXT/MD/JSON) via
`POST /admin/tenants/{tid}/files`. Porting it completes ingestion parity and is
the last common write path missing from the TypeScript console.

## Behavior

- Add a drop zone to the Ingestion page (or a dedicated Files tab) accepting
  `.pdf`, `.txt`, `.md`, `.csv`, `.json`, gated on tenant + collection.
- Upload via `XMLHttpRequest` to surface real upload progress (a progress bar),
  posting multipart `file` + `collection` to `POST /admin/tenants/{tid}/files`.
- Show a success banner with the server's dispatch result
  (`{ file_id, kind, count, name }`) and refresh Dashboard counts; clicking the
  zone also opens a file picker.
- Unsupported types and server errors render inline.

## Likely files

- `crates/hippocore-server/admin-ui/src/api.ts` (XHR upload helper + progress)
- `crates/hippocore-server/admin-ui/src/views/IngestionView.tsx` (drop zone) or
  a new `views/FilesView.tsx`
- `crates/hippocore-server/admin-ui/src/components/DropZone.tsx` (new)
- `crates/hippocore-server/admin-ui/src/App.tsx`, `views.ts`, `styles.css`
- Rebuilt assets in `crates/hippocore-server/src/console/`
- `crates/hippocore-server/tests/server_integration.rs`
- Bilingual status/CHANGELOG and `CONTROL_PLANE_TS.md`

## Acceptance criteria

- Dropping or selecting a supported file uploads it with a visible progress bar
  and shows the dispatch result; the data appears in the Data Explorer.
- Unsupported extensions and server errors render inline without crashing.
- `pnpm type-check` and `pnpm build` pass; the Rust gate is green.
- New user-facing copy exists in English and Portuguese.

## Tests

- `/console` assets stay public (existing integration test).
- Existing file-upload endpoint tests stay green.
- `tsc --noEmit` strict passes; build updates `src/console/`.

## Out of scope

- Integrations (LLM), Prompts, Context builder/Chat, Observability, Graph.
- Bulk/multi-file queues beyond sequential uploads.
- Retiring `/admin`.
