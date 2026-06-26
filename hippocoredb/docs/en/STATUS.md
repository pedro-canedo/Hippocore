# Hippocore DB — Status

_Last updated: 2026-06-26._

## What was implemented last

**File ingestion v0.1**:

- `FileObject`s are now durable stored metadata objects for imported text-like
  local files.
- `import_file` / `delete_file` APIs and `import-file` / `delete-file` CLI
  commands support `.txt`, `.md`, `.json`, and `.csv`.
- Imported files store path, name, media type, CRC32 checksum, byte size,
  metadata, source, timestamps and version.
- Extracted text is projected into a derived `Document` (`file:<id>`) whose
  chunks are indexed and recallable with file metadata.
- Import, recall, metadata filter, JSON projection, restart/delete, unsupported
  extension and CLI flows are covered by tests.

Previous increment: Context data model v0.1:

- Structured JSON `Record`s are now first-class stored objects under a logical
  table namespace.
- `put_record` / `delete_record` APIs and `put-record` / `delete-record` CLI
  commands persist records through the existing WAL/snapshot storage model.
- Each record keeps its original JSON payload and a deterministic text projection
  that is indexed as `ItemKind::Record`.
- Recall/search can return record-derived context, including `record_table`,
  source, metadata, score components and reason.
- Record metadata filters, tenant isolation, restart recovery and delete
  behavior are covered by tests.
- Documentation under `docs/` is now bilingual by rule: every file must exist
  under both `docs/en/` and `docs/pt-br/`.

Earlier increment: RAG Quality Layer v0.1:

- Query normalization maps common PostgreSQL typos/aliases (`postgress`,
  `postgres`) to `postgresql` and connection variants (`conexão`, `conexao`,
  `connection`) to a shared lexical signal.
- The query layer detects simple technology/entity tags (`oracle`,
  `postgresql`, `python`, `listener`, `vacuum`, `connection`) on queries and
  indexed entries.
- Hybrid/text/vector final scores receive small explainable boosts/penalties
  when a query clearly targets Oracle or PostgreSQL, preventing Oracle listener
  memories from outranking Python/PostgreSQL connection memories unless Oracle
  is explicitly mentioned.
- The TypeScript Ollama RAG example now seeds PostgreSQL service, port, `pg_ctl`,
  Python `psycopg`/`psycopg2`, and "not Oracle listener/lsnrctl" memories; it
  also prunes obsolete demo memory ids, prints retrieval debug details, and uses
  a stricter grounded answer prompt.
- A deterministic retrieval-quality fixture now validates Oracle/PostgreSQL RAG
  scenarios with hit@1, hit@5, and MRR thresholds, plus expected answer terms in
  retrieved context.
- Pure vector search skips query normalization and entity detection, keeping the
  quality layer out of the vector-only hot path.

Earlier increments (still current): multi-tenant model; JSON-lines WAL with
per-line CRC32 + atomic snapshot; automatic + manual compaction; delete/forget
(`forget`, `delete_document`); deterministic embedder + chunker;
vector/text/hybrid recall; CLI (`init`/`put-document`/`remember`/`recall`/
`forget`/`delete-document`/`stats`/`inspect`/`compact`, with
`recall --embedding/--json`); and the `examples/ts-ollama-rag/` demo (local
Ollama embeddings + generation, idempotent ingestion).

## What is working

- Store/recall for documents (chunked), memories, structured records and
  imported text-like files.
- Vector, BM25 text, and hybrid modes; every result carries `score`,
  `vector_score`, `text_score`, and a `reason` that includes entity tags and
  boost/penalty explanations when applied.
- Tenant isolation (verified by test).
- Durable restart: snapshot + WAL replay; torn trailing line AND checksum
  mismatch both recover safely (no panic, no silently-loaded corruption).
- Manual `compact()` and automatic threshold-based compaction; both survive
  reopen.
- User-supplied embeddings for memories and queries; built-in embedder otherwise.
- Typed errors for unknown tenant/collection, validation, empty/mismatched
  embeddings, and corruption.
- Retrieval quality evaluation through
  `crates/hippocore/tests/fixtures/retrieval_quality_v01.json`.
- Product direction docs now define Hippocore as a database plus native context
  projection layer, with records/files and future human administration surfaces.

## What is partial

- Retrieval is exact brute force (O(n) per tenant). Correct, not yet scalable.
- The in-memory index is rebuilt fully on open (incremental during runtime).
- Per-chunk external embeddings are a library API; the CLI `put-document` still
  auto-embeds (CLI ergonomics for many chunk vectors are deferred).
- Record field-level indexes, schema management, full blob storage, PDF/OCR and
  a user-facing admin interface are product-direction documents only; they are
  not implemented yet.

## What is broken or missing

- No ANN/HNSW, no server, no auth — by design (see MVP_SCOPE.md).
- Retrieval quality is still heuristic. It is not a learned ranker; the current
  evaluation fixture is intentionally small and should grow with real use cases.

## Commands run

```
cargo fmt --all
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p hippocore
cd examples/ts-ollama-rag && npm run typecheck
cd examples/ts-ollama-rag && npm start -- "Como inicio o listener do Oracle e postgress?"
cd examples/ts-ollama-rag && npm start -- "Como postgress funciona?"
cd examples/ts-ollama-rag && npm start -- "como faço uma conexão python no postgress?"
```

## Current test status

**All green.** `cargo test --workspace` passes 64 tests:
- 16 unit (embedder/chunker, cosine/index/BM25, query normalization, CRC32 + WAL
  line decode),
- 39 library integration (store/recall, chunking, retrieval quality layer,
  structured records, imported files, user-supplied document embeddings +
  validation, modes, sorting, metadata & type/user filters, tenant isolation,
  restart, compaction, auto-compaction bounding the WAL, checksum-mismatch
  recovery, delete/forget + restart + compaction, user embeddings, empty DB,
  error paths, dimension-mismatch, torn-WAL recovery),
- 1 retrieval-quality fixture test (hit@1/hit@5/MRR thresholds),
- 7 CLI subprocess smoke tests (incl. `compact`, `forget`, `put-record`,
  `import-file`),
- 1 doctest.

`cargo clippy … -D warnings` passes with zero warnings; `cargo fmt --all --check`
is clean.

`cargo bench -p hippocore` completed. Final run:
- `remember`: 6.8615-6.9277 ms, no statistically significant change.
- `recall_hybrid`: 696.41-707.12 us, no statistically significant change.
- `search_vector`: 348.91-350.96 us, within the noise threshold.

The performance fix avoids normalization/tag work in pure vector search and
removes a set allocation from query tag detection.

## Current architectural decisions

See DECISIONS.md. Highlights: JSON-lines WAL with per-line CRC32 + atomic
snapshot; threshold-based auto-compaction; unified `IndexEntry` over chunks and
memories; deterministic feature-hashing embedder; min-max hybrid fusion; tenant
isolation enforced in the query layer.

## Next recommended feature

**Admin CLI v0.1** — see NEXT_FEATURE.md.
