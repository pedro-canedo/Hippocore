# Next Feature

## Feature name

**Guided Workspace Onboarding v0.1**

## Why it matters

The Dashboard currently shows onboarding only when there are no tenants and
then immediately becomes a statistics grid. New developers need a persistent,
state-aware path from an empty database to their first query and recall without
knowing which page or object type comes next.

## Behavior

- Show a five-step setup checklist: create tenant, create collection, add first
  data, run SQL, and test recall/context.
- Derive tenant, collection, and data completion from bootstrap stats; record
  successful SQL and recall actions locally for UI progress only.
- Give the first incomplete step one primary action that routes to the correct
  workflow, while completed steps remain visibly checked.
- Keep operational status, active tenant, storage, and API health separate from
  data counts and quick actions.
- Move raw stats out of the Dashboard focus and link to Observability instead.
- Show the next setup action in the sidebar while onboarding is incomplete.

## Likely files

- `crates/hippocore-server/src/admin/app.js`
- `crates/hippocore-server/src/admin/styles.css`
- `crates/hippocore-server/tests/server_integration.rs`
- bilingual Control Plane status/interface docs and `CHANGELOG.md`

## Acceptance criteria

- A new user always sees one clear next action until all five steps are complete.
- Progress updates immediately after successful tenant, collection, ingest, SQL,
  and recall workflows.
- Dashboard raw JSON is no longer a primary panel.
- Existing stats, navigation, auth, and tenant isolation remain unchanged.
- All new copy is complete in English and Portuguese; quality gate is green.

## Tests

- Asset assertions cover setup-step derivation, next-action routing, and local
  SQL/recall progress flags.
- Existing admin workflow integration tests remain green.
- JavaScript syntax check and complete Rust quality gate pass.

## Out of scope

- Server-persisted per-user onboarding state or analytics.
- A product tour overlay, command palette, or framework migration.
- Changing database schemas, retrieval, or persistence.
