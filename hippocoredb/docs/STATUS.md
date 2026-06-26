# Hippocore DB — Status

_Last updated: 2026-06-25._

## What was implemented last

**User-provided embeddings for documents** (the previously planned next
feature):

- `StoreDocumentRequest.chunks: Option<Vec<ChunkInput>>` — when set, the document
  is stored with exactly those caller-supplied, pre-embedded chunks (text +
  embedding) instead of the built-in auto chunk/embed path.
- Each `ChunkInput` is validated (non-empty text and embedding); the chunk id
  scheme (`<doc>#<ordinal>`) and persistence are unchanged.
- When `chunks` is `None`, behavior is exactly as before (auto chunk + embed).
- This makes documents first-class for production RAG with real embedding models
  (memories and queries already accepted external embeddings).

Earlier increments (still current): multi-tenant model; JSON-lines WAL with
per-line CRC32 + atomic snapshot; automatic + manual compaction; delete/forget
(`forget`, `delete_document`); deterministic embedder + chunker;
vector/text/hybrid recall; CLI (`init`/`put-document`/`remember`/`recall`/
`forget`/`delete-document`/`stats`/`inspect`/`compact`, with
`recall --embedding/--json`); and the `examples/ts-ollama-rag/` demo (local
Ollama embeddings + generation, idempotent ingestion).

## What is working

- Store/recall for both documents (chunked) and memories.
- Vector, text, and hybrid modes; every result carries `score`, `vector_score`,
  `text_score`, and a `reason`.
- Tenant isolation (verified by test).
- Durable restart: snapshot + WAL replay; torn trailing line AND checksum
  mismatch both recover safely (no panic, no silently-loaded corruption).
- Manual `compact()` and automatic threshold-based compaction; both survive
  reopen.
- User-supplied embeddings for memories and queries; built-in embedder otherwise.
- Typed errors for unknown tenant/collection, validation, empty/mismatched
  embeddings, and corruption.

## What is partial

- Retrieval is exact brute force (O(n) per tenant). Correct, not yet scalable.
- The in-memory index is rebuilt fully on open (incremental during runtime).
- Text scoring is TF-IDF, not full BM25 (no length normalization / saturation).
- Per-chunk external embeddings are a library API; the CLI `put-document` still
  auto-embeds (CLI ergonomics for many chunk vectors are deferred).

## What is broken or missing

- No ANN/HNSW, no server, no auth — by design (see docs/MVP_SCOPE.md).
- Text relevance uses TF-IDF; proper **BM25** is the next planned feature
  (see docs/NEXT_FEATURE.md).

## Commands run

```
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p hippocore --example basic_usage
cargo bench -p hippocore --no-run
```

## Current test status

**All green.** 45 tests pass:
- 12 unit (embedder/chunker, cosine/index, CRC32 + WAL line decode),
- 27 library integration (store/recall, chunking, user-supplied document
  embeddings + validation, modes, sorting, metadata & type/user filters, tenant
  isolation, restart, compaction, auto-compaction bounding the WAL,
  checksum-mismatch recovery, delete/forget + restart + compaction, user
  embeddings, empty DB, error paths, dimension-mismatch, torn-WAL recovery),
- 5 CLI subprocess smoke tests (incl. `compact`, `forget`),
- 1 doctest.

`cargo clippy … -D warnings` passes with zero warnings; `cargo fmt --all --check`
is clean.

## Current architectural decisions

See docs/DECISIONS.md. Highlights: JSON-lines WAL with per-line CRC32 + atomic
snapshot; threshold-based auto-compaction; unified `IndexEntry` over chunks and
memories; deterministic feature-hashing embedder; min-max hybrid fusion; tenant
isolation enforced in the query layer.

## Next recommended feature

**BM25 text scoring** (replace TF-IDF with length-normalized BM25 for better
text and hybrid relevance) — see docs/NEXT_FEATURE.md.
