# Next Feature

## Feature name

**Admin Data Actions v0.1**

## Why it matters

The Control Plane now separates the product domains and shows the right
workflows, but some important UI actions are intentionally marked Planned
because backend admin endpoints do not exist yet. The next highest-value slice
is to make the planned data actions real without expanding into a full SQL
engine or a large frontend framework.

## Behavior

- Add admin/server endpoints for structured record insert and file import.
- Reuse existing core APIs: `put_record` and `import_file`.
- Keep tenant isolation explicit in every route.
- Update Control Plane Ingestion & Recall so Memory, Document, Record, and File
  are all actionable when supported by backend endpoints.
- Keep missing/future flows visibly marked Planned instead of faking data.

## Likely files

- `crates/hippocore-server/src/handlers/records.rs`
- `crates/hippocore-server/src/handlers/files.rs`
- `crates/hippocore-server/src/lib.rs`
- `crates/hippocore-server/src/admin/app.js`
- `crates/hippocore-server/tests/server_integration.rs`
- `docs/en/ADMIN_INTERFACE.md`
- `docs/pt-br/ADMIN_INTERFACE.md`
- `docs/en/SERVER.md`
- `docs/pt-br/SERVER.md`

## Acceptance criteria

- `POST /admin/tenants/:tid/records` creates a JSON-first record.
- `POST /admin/tenants/:tid/files/import` imports a local text-like file path.
- Existing admin auth/login behavior remains unchanged.
- Control Plane can create Memory, Document, and Record from Ingestion & Recall.
- File import is implemented only if it can be done safely from a local path;
  otherwise it remains Planned with clear docs.
- Tests cover success, validation errors, and tenant isolation.
- `cargo fmt --all --check`, `cargo test --workspace`, and
  `cargo clippy --workspace --all-targets -- -D warnings` pass.

## Out of scope

- Browser multipart upload.
- CSV rows as records.
- PDF upload.
- Full SQL `INSERT`.
- Table/schema catalog.
- Frontend framework migration.
