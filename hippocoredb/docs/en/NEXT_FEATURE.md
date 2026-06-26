# Next Feature

## Feature name

**File ingestion v0.1** — store text-like files as database objects and project
them into native context.

## Why it matters

Hippocore DB is moving from memory/document retrieval toward a broader database
where stored information becomes context. Structured records now exist. The next
practical data type is a file object, starting with text-like formats that can
be handled locally without external parsers or OCR.

## Expected behavior

- Add a first-class `FileObject` or equivalent model:
  - tenant,
  - collection,
  - id,
  - original path/name,
  - media type or extension,
  - checksum,
  - metadata,
  - source,
  - created/updated/version fields.
- Add a CLI command such as:

```bash
hippocore import-file --db ./data --tenant acme --collection kb --path ./notes.md
```

- Support local text-like inputs first:
  - `.txt`,
  - `.md`,
  - `.json`,
  - `.csv`.
- Store file metadata durably and project extracted text into document chunks or
  record-like context.
- Keep original binary/blob storage minimal for now; do not introduce a large
  blob engine until the model is proven.

## Affected modules / files

- `model.rs` — file metadata model.
- `storage.rs` — file operations and state persistence.
- `lib.rs` — public import/store API.
- `cli.rs` — `import-file` command.
- `memory` or a small ingestion helper — deterministic text extraction for
  supported formats.
- `DATA_MODEL.md`, `README.md`, `STATUS.md`, `CHANGELOG.md`.
- Tests for import, recall, metadata, restart, delete and unsupported formats.

## Acceptance criteria

- A `.txt` or `.md` file can be imported and recalled.
- `.json` and `.csv` have deterministic text projections.
- Imported file metadata survives reopen.
- Unsupported extensions return a typed/clear error.
- Tenant isolation and metadata filters hold.
- No network or external parser dependency is required.
- `cargo fmt --all --check`, `cargo test --workspace`, and clippy pass.

## Non-goals

- PDF parsing.
- OCR.
- Image/audio/video embeddings.
- Full blob storage engine.
- Server mode.
- Admin UI.

## Follow-up

After file ingestion, improve CLI administration (`list-*`, `inspect --json`,
export/import) and then revisit a local Hippocore Studio prototype.
