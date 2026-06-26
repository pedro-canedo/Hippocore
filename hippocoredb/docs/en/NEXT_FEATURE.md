# Next Feature

## Feature name

**Phase 12 — Multimodal Storage** (text, PDF, code, images)

## Why it matters

Phase 11 (HTTP server) made Hippocore accessible to any language. Phase 12
extends the document model to handle richer content types: PDFs, source code
files, and eventually images. This closes the gap between Hippocore and
production RAG pipelines that must process real-world inputs.

## Minimum viable scope for Phase 12

1. **PDF ingestion**: accept a PDF byte payload, extract text per-page, chunk
   and embed each page. Store page number in metadata.
2. **Code file ingestion**: accept source files with a `language` hint; use
   token-aware chunking that respects function boundaries (heuristic, no tree-
   sitter required).
3. **MIME-type routing**: `StoreDocumentRequest` gains a `content_type` field;
   the core library dispatches to the appropriate extractor.
4. **Tests**: at least one PDF round-trip test and one code-file round-trip test.
5. **Docs**: bilingual documentation for the new `content_type` API.

## Out of scope for Phase 12

- Image / audio / video (later phases).
- GPU acceleration.
- Tree-sitter AST parsing.
- Cloud-hosted extractors.
