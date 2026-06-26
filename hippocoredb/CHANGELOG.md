# Changelog

All notable changes to Hippocore DB are documented here. This project adheres to
[Semantic Versioning](https://semver.org/) (pre-1.0: minor versions may break).

## [Unreleased]

### Added — Context data model v0.1

- Added first-class structured JSON `Record`s under a logical table namespace.
- Added `Hippocore::put_record` / `delete_record` and CLI `put-record` /
  `delete-record`.
- Records preserve the original JSON payload and store a deterministic text
  projection for retrieval.
- Record projections are indexed alongside memories and document chunks and can
  be filtered with metadata or `--kind record`.
- Record writes/deletes are durable through WAL/snapshot recovery and covered by
  restart, delete, metadata-filter, tenant-isolation and CLI tests.

### Added — RAG Quality Layer v0.1

- Query normalization for PostgreSQL aliases/typos (`postgres`, `postgress`) and
  connection variants (`conexão`, `conexao`, `connection`) before lexical
  scoring and built-in query embedding.
- Simple technology/entity detection for Oracle, PostgreSQL, Python, listener,
  vacuum, and connection signals.
- Explainable retrieval boosts/penalties so clearly PostgreSQL queries rank
  PostgreSQL memories over Oracle memories, clearly Oracle queries rank Oracle
  memories over PostgreSQL memories, and mixed Oracle/PostgreSQL queries can
  still retrieve both.
- Regression tests for PostgreSQL, Oracle, mixed Oracle/PostgreSQL, typo
  normalization, Python/PostgreSQL connection ranking, and vector dimension
  mismatch safety.
- Deterministic retrieval-quality fixture with hit@1, hit@k, and MRR thresholds
  for Oracle/PostgreSQL RAG scenarios.
- The TypeScript Ollama RAG example now includes stronger PostgreSQL seed
  memories, richer retrieval debug output, a stricter grounding prompt, and
  cleanup for obsolete demo memory ids during ingestion.
- Docs now distinguish documents, chunks, memories, indexed entries, and WAL
  entries, including why memory-only examples can show `documents=0` with
  `memories>0`.

### Changed — retrieval performance hygiene

- Pure vector search now skips query normalization and entity/tag detection.
  Entity detection no longer allocates a set for query tags, keeping the RAG
  quality layer out of the vector-only hot path.

### Documentation — product direction

- Added `docs/en/DATA_MODEL.md` and `docs/pt-br/DATA_MODEL.md` to define Hippocore as a database plus native
  context projection layer for documents, memories, future records, files and
  context items.
- Added `docs/en/ADMIN_INTERFACE.md` and `docs/pt-br/ADMIN_INTERFACE.md` to define the CLI/admin UI direction,
  including the future Hippocore Studio concept for human management of context
  data.
- Moved project docs under `docs/en/` and `docs/pt-br/`; every document under
  `docs/` must now have same-named English and Portuguese versions.
- Updated the roadmap and next-feature plan toward `Context data model v0.1`
  before server mode, SQL compatibility or a full UI.

### Added — user-provided document embeddings

- `StoreDocumentRequest.chunks: Option<Vec<ChunkInput>>` (+ public `ChunkInput`):
  store a document with caller-supplied, pre-embedded chunks instead of the
  built-in auto chunk/embed path. Each chunk's text and embedding are validated.
  `None` keeps the previous auto behavior. Makes documents first-class for RAG
  with real embedding models.

### Added — delete / forget

- `Hippocore::forget(tenant, collection, id)` removes a memory;
  `Hippocore::delete_document(tenant, collection, id)` removes a document and all
  its chunks. Both write a durable tombstone (`Operation::DeleteMemory` /
  `DeleteDocument`), update the index, and are replayed on recovery. Deleting a
  missing id is a safe no-op. Compaction drops tombstones and dead records.
- CLI: `hippocore forget` and `hippocore delete-document`.

### Added — WAL durability hardening + automatic compaction

- **Per-line CRC32 checksums** in the WAL (`<crc32-hex>\t<json>`, format v2,
  dependency-free CRC32). Recovery verifies every line and stops safely on a
  checksum mismatch, so on-disk corruption is detected and never silently
  loaded. Legacy v1 (unchecksummed) lines are still read.
- **Automatic compaction policy**: `Config.auto_compact_after_ops` (default
  1000) and `Config.auto_compact_after_bytes` (default 8 MiB); `0` disables.
  The engine compacts transparently once a threshold is crossed, keeping the
  WAL and recovery time bounded without manual `compact`.

### Added — AI-native memory database

- **Multi-tenant model**: `Tenant`, `Collection`, `Document`, `Chunk`, `Memory`,
  `Source`, `MemoryType`, `ItemKind`, `RecallResult`.
- **Memories** (`remember`) and **auto-chunked documents** (`store_document`).
- **Retrieval**: exact vector (cosine), text (TF-IDF inverted index), and
  **hybrid** recall, with filters by tenant, collection, user, memory type, item
  kind, and metadata. Every result reports `score`, `vector_score`,
  `text_score`, and a human-readable `reason`.
- **Deterministic, offline embedder** (signed feature hashing); user-provided
  embeddings accepted for memories and queries.
- **Persistence**: JSON-lines write-ahead log + atomic snapshot, startup
  recovery, and `compact()`. Torn trailing WAL lines recover safely.
- **Tenant isolation** enforced in the query layer.
- **Public API**: `open`, `create_tenant`, `create_collection`,
  `store_document`, `remember`, `recall`, `search`, `stats`, `compact`, `close`.
- **CLI**: `init`, `put-document`, `remember`, `recall`, `stats`, `inspect`,
  `compact`. `recall` also accepts `--embedding` (caller-supplied query vector)
  and `--json` (machine-readable output) to enable external integrations;
  `--embedding` allows negative components. `compact` folds the WAL into a fresh
  snapshot and truncates the log (bounds disk/recovery growth).
- **Integration example**: `examples/ts-ollama-rag/` — a TypeScript RAG demo
  using local Ollama (embeddings + generation) on top of Hippocore via the CLI.
- **Docs**: `PLAN.md`, localized docs under `docs/en/` and `docs/pt-br/`.
- Deterministic test suite (30 tests), example, and baseline benchmarks.

### Changed

- Reorganized the crate into `config`, `errors`, `model`, `storage`, `index`,
  `memory`, `query`, `cli` modules with the engine in `lib.rs`.
- Replaced the v0.1 document-only binary log with the tenant-aware WAL +
  snapshot engine.
- CLI is now driven by `hippocore::cli::run()`; the binary crate is a thin
  wrapper.

### Removed

- `crc32fast` dependency (binary record framing retired in favor of JSON-lines
  WAL; per-line checksums are tracked as a follow-up in `docs/en/NEXT_FEATURE.md`).

## [0.1.0]

- Initial embedded document store: append-only binary log, in-memory index,
  brute-force cosine search, metadata filtering, recovery, and CLI.
