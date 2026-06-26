# Next Feature

## Feature name

**Document Chunking Tests v0.1** — deterministic tests verifying that
`store_document` produces correct chunks and that chunk-level recall works.

## Why it matters

`store_document` auto-chunks the document text and embeds each chunk
independently. If the chunker breaks, the vector index receives zero entries
and recall silently returns nothing. No dedicated test suite locks this path.

## Behaviour

No new production code. The test suite will verify:

1. Storing a document with long text produces at least 1 chunk (via
   `get_document_chunks`).
2. Recall with a substring query returns the parent document's id.
3. Multiple documents stored in the same collection each produce their own
   chunk set — no cross-contamination.
4. Chunks survive `compact()` + cold reopen.
5. Deleting a document removes its chunks from `get_document_chunks`.

## Files

- `crates/hippocore/tests/document_chunking.rs` — new dedicated test file.
- `docs/en/DOCUMENT_CHUNKING.md` and `docs/pt-br/DOCUMENT_CHUNKING.md`.
- `docs/en/STATUS.md` and `docs/pt-br/STATUS.md`.

## Acceptance criteria

- At least 5 deterministic integration tests using `TempDir`.
- All quality gates pass:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Out of scope

- Custom chunker configuration.
- Server mode.
- Production code changes.
