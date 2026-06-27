# Next Feature

## Feature name

**Control Plane TS — Integrations (LLM) v0.8**

## Why it matters

The console covers the full data loop but not the LLM "brain" configuration. The
classic console has an Integrations page (providers + validate + live ping + API
exposure). Porting provider configuration to `/console` unlocks the later Chat
(RAG → LLM) feature and lets operators wire the brain in the new typed UI.

## Behavior

- Add an **Integrations** page to the sidebar.
- List configured providers from `GET /admin/llm-providers`; add/update one via
  `POST /admin/llm-providers` (id, kind, base_url, model, optional api_key).
- **Validate** a provider config locally (`POST /admin/llm-providers/validate`)
  and **Ping** it live (`POST /admin/llm-providers/ping`), showing ok/message
  and latency without crashing on network failure.
- Show API exposure from `GET /admin/api-info` (base URL, masked key hint) with
  a curl recall example.

## Likely files

- `crates/hippocore-server/admin-ui/src/api.ts` (provider + api-info calls)
- `crates/hippocore-server/admin-ui/src/views/IntegrationsView.tsx` (new)
- `crates/hippocore-server/admin-ui/src/App.tsx`, `views.ts`, `styles.css`
- Rebuilt assets in `crates/hippocore-server/src/console/`
- `crates/hippocore-server/tests/server_integration.rs`
- Bilingual status/CHANGELOG and `CONTROL_PLANE_TS.md`

## Acceptance criteria

- Providers list, add/update, validate, and ping all work; the stored API key is
  never shown (only a set/length indicator and masked hint).
- A bad ping URL returns ok:false inline (no crash, no 500 surfaced as fatal).
- `pnpm type-check` and `pnpm build` pass; the Rust gate is green.
- New user-facing copy exists in English and Portuguese.

## Tests

- `/console` assets stay public (existing integration test).
- Existing provider-registry tests (secret masking) stay green.
- `tsc --noEmit` strict passes; build updates `src/console/`.

## Out of scope

- Prompts and Chat (the next increment after this).
- Traefik/Docker config generators (can follow later).
- Observability, Graph, Settings.
