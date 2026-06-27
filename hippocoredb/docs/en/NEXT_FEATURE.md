# Next Feature

## Feature name

**Control Plane TS — Navigation Shell + Tenants & Collections v0.2**

## Why it matters

The TypeScript console (`/console`) currently has only a Login → Dashboard
slice. The next highest-value increment is the navigation shell plus the first
data-mutation pages — Tenants and Collections — which establish the sidebar,
routing, global tenant selector, and the create/list form pattern that every
later page reuses.

## Behavior

- Add a persistent sidebar shell (brand, grouped navigation, sign-out) and a
  client-side route/view switch driven by component state (no router library
  required yet).
- A global tenant selector in the topbar persists the active tenant in local
  storage and is shared with the rest of the console.
- **Tenants page**: list tenants from `GET /admin/tenants`; create a tenant via
  `POST /admin/tenants`; show empty state and inline validation errors.
- **Collections page**: list collections for the active tenant from
  `GET /admin/collections?tenant_id=`; create one via
  `POST /admin/tenants/{tid}/collections`; require an active tenant first.
- All requests go through the typed API client; success refreshes the list and
  shows a toast; typed `ApiError` surfaces server messages.
- Dashboard gains links into the new pages.

## Likely files

- `crates/hippocore-server/admin-ui/src/api.ts` (add tenants/collections calls)
- `crates/hippocore-server/admin-ui/src/App.tsx` (route/view state + shell)
- `crates/hippocore-server/admin-ui/src/components/Shell.tsx` (new)
- `crates/hippocore-server/admin-ui/src/views/TenantsView.tsx` (new)
- `crates/hippocore-server/admin-ui/src/views/CollectionsView.tsx` (new)
- `crates/hippocore-server/admin-ui/src/styles.css`
- Rebuilt assets in `crates/hippocore-server/src/console/`
- `crates/hippocore-server/tests/server_integration.rs`
- Bilingual status/CHANGELOG and `CONTROL_PLANE_TS.md`

## Acceptance criteria

- Sidebar navigation switches between Dashboard, Tenants, and Collections
  without a full reload.
- Creating a tenant and a collection persists and immediately appears in the
  lists; tenant isolation is respected for collections.
- The active tenant is remembered across reloads and shared with `/admin`.
- `pnpm type-check` and `pnpm build` pass; the Rust gate is green.
- New copy exists in both English and Portuguese where user-facing.

## Tests

- Integration assertions that `/console` assets still serve and remain public.
- Existing admin/server tests stay green (tenants/collections endpoints already
  covered server-side).
- `tsc --noEmit` strict passes; build succeeds and updates `src/console/`.

## Out of scope

- Porting Records, Memories, Documents, Files, SQL Editor, Recall, Integrations,
  Prompts, or Observability (later increments).
- Adding a router library, state-management library, or component kit.
- Retiring `/admin` or redirecting `/` (only after full parity).
