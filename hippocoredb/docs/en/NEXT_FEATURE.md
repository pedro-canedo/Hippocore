# Next Feature

## Feature name

**SQL Editor Workbench v0.1**

## Why it matters

The SQL Editor executes queries, but it still behaves like a large textarea.
Developers need an efficient workbench that teaches Hippocore's payload mapping,
preserves recent work, and makes results easy to inspect or export.

## Behavior

- Give the editor a code-oriented toolbar with Run and `Ctrl/Cmd + Enter` support.
- Keep the current query in page state and make examples fill the editor without
  losing tenant scope.
- Store the latest 20 successful queries locally by data directory and tenant;
  selecting a history entry restores it without executing automatically.
- Render record payload fields as deterministic result columns, with Table and
  Raw JSON tabs.
- Show structured validation errors and client-observed execution duration.
- Add Copy JSON and Export JSON actions for successful results.
- Explain that bare fields map to `payload.<field>` with a concise inline example.

## Likely files

- `crates/hippocore-server/src/admin/app.js`
- `crates/hippocore-server/src/admin/styles.css`
- `crates/hippocore-server/tests/server_integration.rs`
- bilingual Control Plane status/interface docs and `CHANGELOG.md`

## Acceptance criteria

- Run button and keyboard shortcut execute the same tenant-scoped request.
- Examples and history restore query text without automatic execution.
- Successful results expose payload columns in deterministic order and remain
  available as complete JSON.
- Copy/export actions use exactly the current result object.
- History is capped, local-only, and scoped by database and tenant.
- All new copy is complete in English and Portuguese; quality gate is green.

## Tests

- Asset assertions cover keyboard execution, history scope/cap, payload columns,
  copy/export, and error rendering.
- Existing SQL endpoint parser, executor, and tenant-isolation tests remain green.
- JavaScript syntax check and complete Rust quality gate pass.

## Out of scope

- Expanding the SQL grammar or adding write statements.
- Server-side query history, saved queries, pagination, or CSV export.
- Monaco/CodeMirror, npm dependencies, or React/Vite migration.
