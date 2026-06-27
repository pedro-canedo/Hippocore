# Next Feature

## Feature name

**Control Plane TS — Prompts + Chat (SSE) v0.9**

## Why it matters

With the LLM brain configurable in `/console`, the payoff is the RAG → LLM Chat.
The classic console has System Prompts CRUD and a streaming Chat endpoint. Porting
them realizes the "LLM as brain" vision in the new typed UI and is the last major
product surface before `/admin` can be retired.

## Behavior

- Add a **Prompts** page: list/create/update/delete system prompt templates via
  `GET/POST/PUT/DELETE /admin/prompts` (id, name, description, content with
  `{{context}}`/`{{query}}` placeholders, optional tenant scope).
- Add a **Chat** panel (gated on an active tenant): query + optional collection
  + optional prompt id, streaming the answer token-by-token from
  `POST /admin/tenants/{tid}/chat` using `fetch` + `response.body.getReader()`.
- Render streamed tokens live into an output area; show a final sources list and
  handle `event: error` from the SSE stream without crashing.

## Likely files

- `crates/hippocore-server/admin-ui/src/api.ts` (prompts CRUD + chat stream)
- `crates/hippocore-server/admin-ui/src/views/PromptsView.tsx` (new)
- `crates/hippocore-server/admin-ui/src/App.tsx`, `views.ts`, `styles.css`
- Rebuilt assets in `crates/hippocore-server/src/console/`
- `crates/hippocore-server/tests/server_integration.rs`
- Bilingual status/CHANGELOG and `CONTROL_PLANE_TS.md`

## Acceptance criteria

- Prompts CRUD works end to end; a prompt can be selected for chat.
- Chat streams tokens live and shows sources; SSE errors render inline.
- `pnpm type-check` and `pnpm build` pass; the Rust gate is green.
- New user-facing copy exists in English and Portuguese.

## Tests

- `/console` assets stay public (existing integration test).
- Existing prompts/chat endpoint tests stay green.
- `tsc --noEmit` strict passes; build updates `src/console/`.

## Out of scope

- Observability, Graph, API Reference, Settings, Documentation ports.
- Traefik/Docker config generators.
- Retiring `/admin` (only after parity across the remaining pages).
