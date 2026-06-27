# Next Feature

## Feature name

**Control Plane TS — Recall v0.6**

## Why it matters

The console can now create, browse, and query data. The signature capability —
RAG recall — is still only in `/admin`. Porting Recall to `/console` lets
operators test retrieval (vector/text/hybrid) with scores and matched terms
right where they ingested the data, completing the product's core demo loop in
the new typed UI.

## Behavior

- Add a **Recall** page to the sidebar, gated on an active tenant.
- A query box with an optional collection scope, mode selector
  (vector/text/hybrid), and `top_k`, calling `POST /admin/tenants/{tid}/recall`.
- Render results as cards showing kind, score, matched terms, and a text
  snippet, plus a Raw JSON toggle for the full response.
- Surface typed `ApiError` inline; empty results show a clear empty state.

## Likely files

- `crates/hippocore-server/admin-ui/src/api.ts` (add `recall` + result types)
- `crates/hippocore-server/admin-ui/src/views/RecallView.tsx` (new)
- `crates/hippocore-server/admin-ui/src/App.tsx`, `views.ts`, `styles.css`
- Rebuilt assets in `crates/hippocore-server/src/console/`
- `crates/hippocore-server/tests/server_integration.rs`
- Bilingual status/CHANGELOG and `CONTROL_PLANE_TS.md`

## Acceptance criteria

- Running a query returns ranked results with scores and matched terms for the
  active tenant; mode and top_k are honored; tenant isolation holds.
- Empty results and errors render without crashing the view.
- `pnpm type-check` and `pnpm build` pass; the Rust gate is green.
- New user-facing copy exists in English and Portuguese.

## Tests

- `/console` assets stay public (existing integration test).
- Existing recall endpoint tests stay green.
- `tsc --noEmit` strict passes; build updates `src/console/`.

## Out of scope

- Context builder / chat (later increments).
- File upload / drag-and-drop, Integrations, Prompts, Observability.
- Tuning MMR/dedup/min-score knobs beyond mode and top_k.
