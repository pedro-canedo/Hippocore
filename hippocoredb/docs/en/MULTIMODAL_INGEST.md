# Multimodal Ingestion — Phase 12

## Overview

Phase 12 extends `StoreDocumentRequest` with a `content_type` field and a `raw`
binary payload field, enabling the database to ingest content beyond plain text.
A new `ingest` module routes documents through the appropriate extractor before
the normal chunking/embedding path.

## New fields on `StoreDocumentRequest`

| Field | Type | Description |
|---|---|---|
| `content_type` | `Option<String>` | MIME type hint. `None` = plain text. |
| `raw` | `Option<Vec<u8>>` | Binary payload (required for PDF). |

## New convenience constructors

### `StoreDocumentRequest::from_pdf(tenant, collection, bytes)`

Builds a request with `content_type = "application/pdf"` and `raw = bytes`.
The `text` field is left empty — text is extracted from the PDF during ingestion.

### `StoreDocumentRequest::from_code(tenant, collection, text, language)`

Builds a request with `content_type = "text/x-<language>"`. Uses the heuristic
code chunker instead of the default token chunker.

## Supported content types

### `application/pdf`

Requires `raw` bytes. Text is extracted via `pdf-extract` (pure Rust, no C
dependencies). Pages are extracted as a single concatenated string.

If extraction fails (malformed PDF), a `HippocoreError::Validation` is returned
— never a panic.

### `text/x-<language>` and `application/x-<language>`

Supported language keys: `rust`, `python`, `javascript`, `typescript`, `js`,
`ts`, `go`, `java`, `kotlin`.

The heuristic chunker splits at non-indented lines that begin with common
declaration keywords (e.g. `fn `, `def `, `class `, `func `). Segments are
merged until they fill the token budget; oversized segments are split by the
standard token chunker.

Unrecognised language keys fall back to the standard token chunker.

## Implementation

- `crates/hippocore/src/ingest.rs` — new module:
  - `extract_pdf(bytes: &[u8]) -> Result<String>`
  - `chunk_code(text, language, chunk_tokens) -> Vec<String>`
  - `is_pdf(content_type) -> bool`
  - `code_language(content_type) -> Option<String>`
  - `normalise_mime(content_type) -> String`
- `Hippocore::store_document` dispatches on `content_type` before building chunks.
- New private method `build_chunks_code` uses `ingest::chunk_code`.

## Test file

`crates/hippocore/tests/multimodal_ingest.rs` — 8 integration tests:
- `plain_text_document_produces_chunks`
- `rust_code_document_splits_at_fn_boundaries`
- `python_code_document_splits_at_def_boundaries`
- `unknown_language_falls_back_to_token_chunker`
- `code_document_recalled_by_function_name`
- `pdf_text_is_extracted_and_chunked`
- `missing_raw_bytes_for_pdf_returns_error_not_panic`
- `content_type_stored_in_document_text_field`

Plus 6 unit tests in `ingest.rs` for MIME routing and code chunker boundary logic.
