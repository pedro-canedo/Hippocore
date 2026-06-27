# Next Feature

## Feature name

**Control Plane i18n Completeness v0.1**

## Why it matters

The language selector currently translates only part of the interface. Hardcoded
English remains in navigation, onboarding, page headers, empty states, forms,
toasts, and errors, so Portuguese users see a mixed-language product. Completing
the existing PT/EN boundary is a P0 usability requirement before adding more UI.

## Behavior

- Move all user-facing Control Plane strings into the existing `I18N` catalog.
- Translate navigation groups/pages, onboarding, data pages, SQL, files,
  integrations, prompts, observability, settings, actions, errors, and toasts.
- Keep technical identifiers such as `tenant`, `collection`, `record`, SQL,
  endpoint paths, field names, and provider names unchanged where translation
  would reduce precision.
- Switching PT/EN rerenders the active page without changing tenant, collection,
  query results, drafts, or session state.

## Likely files

- `crates/hippocore-server/src/admin/app.js`
- `crates/hippocore-server/tests/server_integration.rs`
- bilingual Control Plane status/interface docs and `CHANGELOG.md`

## Acceptance criteria

- Every visible interface sentence and command label follows the selected language.
- No mixed PT/EN copy remains except documented technical vocabulary.
- Language switching preserves active workflow state.
- Existing endpoint behavior and authentication remain unchanged.
- JavaScript syntax check and the complete Rust quality gate are green.

## Tests

- Asset assertions cover both English and Portuguese catalog markers.
- A static audit confirms user-facing render paths use translation keys.
- Existing server integration tests remain green.

## Out of scope

- Adding a third language or external localization framework.
- Translating API field names, SQL syntax, ids, or stored user data.
- Migrating the frontend to React/Vite.
