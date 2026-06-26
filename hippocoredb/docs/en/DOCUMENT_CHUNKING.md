# Document Chunking Tests v0.1

## Overview

A dedicated test suite verifying that `store_document` correctly chunks and
indexes document text, and that the chunk lifecycle (compact + reopen, delete)
works correctly. No production code was changed; this feature is purely
additive tests.

## Key API detail

In `RecallResult`, `id` is the **chunk id** (not the document id). The parent
document id is in `document_id: Option<String>`. Tests must compare against
`result.document_id` when searching for a document via its chunks.

## Test file

`crates/hippocore/tests/document_chunking.rs`

## What is tested

| Test | Scenario |
|---|---|
| `stored_document_produces_chunks` | `get_document_chunks` returns ≥1 chunk after storing a document |
| `recall_returns_parent_document_id` | `recall()` results contain the correct `document_id` |
| `two_documents_produce_separate_chunks` | Chunk ids from two documents are disjoint |
| `chunks_survive_compact_reopen` | Chunks present after `compact()` + cold reopen |
| `deleting_document_removes_chunks` | `delete_document` clears all its chunks from `get_document_chunks` |

## Non-goals

- Custom chunker configuration.
- Server mode.
- Production code changes.
