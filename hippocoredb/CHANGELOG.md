# Changelog

All notable changes to Hippocore DB are documented here. This project adheres to
[Semantic Versioning](https://semver.org/) (pre-1.0: minor versions may break).

## [Unreleased]

### Added — Phase 7: Confidence-aware resolution

- `confidence: Option<f32>` on `Memory`, `RecallResult`, `ContextItem`, and
  `RememberRequest` (all `#[serde(default)]`, backward-compatible with existing WAL).
- `Memory::validate()` enforces `confidence ∈ [0.0, 1.0]` when set.
- `Hippocore::rate_memory(tenant, collection, id, confidence)` — human-in-the-loop
  rating API; durably persists via WAL `PutMemory`.
- `IndexEntry` propagates `confidence` through `build_result` to `RecallResult`.
- `build_context` applies confidence-aware re-ranking when contradictions exist:
  `effective_score = recall_score × 0.7 + confidence × 0.3`; unrated items use
  `confidence = 0.5` (neutral, not penalised).
- `ContextItem` gains `confidence: Option<f32>` for caller introspection.
- CLI: `remember --confidence <f32>` and new `rate-memory` subcommand.
- 6 new integration tests.

### Added — Phase 2: Group-commit / batch writes

- `Storage::append_many(ops: &[Operation])` — serializes all ops to one buffer,
  one `write_all`, one conditional `sync_all`. Cost: one fsync for N ops.
- `Hippocore::remember_many(reqs: Vec<RememberRequest>)` — validates all requests
  up-front, builds ops vec, calls `append_many`, applies state + index in loop.
  Auto-compaction check at end.
- `Hippocore::store_documents(reqs: Vec<StoreDocumentRequest>)` — same group-commit
  pattern for documents and their chunks.
- 3 new integration tests: batch memories, batch documents, batch durability across
  restart.

### Added — Context Compiler (`build_context`)

- New `BuildContextRequest` type with `tenant_id`, `query`, `max_tokens`
  (default 2048), `top_k_candidates` (default 20), `mode` (default `Hybrid`),
  optional `collection` and `metadata_filter`.
- New `ContextBlock` return type with `text` (LLM-ready string), `token_count`
  (1 token ≈ 4 UTF-8 bytes via `usize::div_ceil(4)`), `items_included:
  Vec<ContextItem>`, and `items_dropped`.
- `ContextItem` carries `id`, `kind`, `score`, `token_count`, and a 120-char
  `snippet`.
- `Hippocore::build_context(req)`: recalls up to `top_k_candidates` items,
  ranks by score descending, greedily fills the budget — items exceeding the
  ceiling are counted in `items_dropped` (not silently lost).
- Format: `[<kind>:<id>]\n<text>`, blocks separated by `\n\n`.
- CLI `build-context`: `--tenant`, `--query`, `--max-tokens`, `--top-k`,
  `--mode`, `--collection`, `--meta`, `--json`.
- 3 new integration tests: budget enforcement, score ordering + provenance,
  `token_count` self-consistency.
- No new crate dependencies.

### Added — Temporal Truth Layer full spec (supersedes / contradicts)

- `Memory` gains `supersedes: Vec<String>`, `contradicts: Vec<String>`, and
  `superseded_by: Option<String>`. All three are `#[serde(default)]` so the
  WAL/snapshot can recover existing databases without migration.
- `RecallResult` gains `contradictions: Vec<String>` (advisory; ids of memories
  this result contradicts, carried from the index entry).
- `RememberRequest` gains `supersedes` and `contradicts` fields; `RecallRequest`
  gains `include_superseded: bool` (default `false`).
- `remember` validates that every id in `supersedes`/`contradicts` exists in the
  same tenant before accepting the request. After writing the new memory, each
  superseded memory is updated in the WAL with `superseded_by = Some(new_id)`
  (durable; survives restart).
- `IndexEntry` carries `superseded: bool` and `contradicts: Vec<String>` derived
  from the parent `Memory`, so `Filter::matches` can exclude superseded entries
  in O(1) without reloading the memory.
- `Filter::matches` excludes entries where `superseded = true` unless
  `include_superseded = true` on the filter.
- CLI: `remember --supersedes <id>` (repeatable), `remember --contradicts <id>`
  (repeatable), `recall --include-superseded`.
- Fixed a race condition in the eval-quality temp dir name (now includes process
  id to prevent timestamp collision under parallel test execution).
- 5 new integration tests: supersedure exclusion, include-superseded flag,
  contradictions surface in result, unknown-id validation error, durability
  across restart.
- No new crate dependencies.

### Added — Benchmark regression guard

- Added `examples/bench_regression.rs`: standalone latency guard for hybrid,
  vector, and text recall over 500 memories (5 warmup + 50 measured iterations).
- Committed `benches/baseline.json` with mean latencies at time of authoring
  (~430–470 µs on the development machine at release profile).
- `HIPPO_BENCH_UPDATE=1 cargo run --release --example bench_regression` rewrites
  the baseline. Without that env var, the command compares current latency to the
  baseline and exits non-zero if any benchmark exceeds `HIPPO_BENCH_THRESHOLD`
  (default 0.20 = 20% regression threshold).
- Added `[[example]] bench_regression` entry to `crates/hippocore/Cargo.toml`.
- No new crate dependencies.

### Added — Temporal Truth Layer v0.1

- Added `valid_from: Option<i64>` and `valid_until: Option<i64>` to `Memory`
  and `Document`. Both fields are `#[serde(default)]` so existing serialized
  records decode as always-valid without any migration.
- `RememberRequest` and `StoreDocumentRequest` expose matching fields so callers
  can set temporal constraints at write time.
- `RecallRequest` gains `as_of: Option<i64>`. When `None` (the default),
  `run_query` resolves it to the current time before executing the query,
  meaning expired entries are excluded from all default recall operations.
- `Filter` gains `as_of: Option<i64>`; `Filter::matches` rejects entries where
  `valid_from > as_of` (not yet valid) or `valid_until <= as_of` (expired).
- `IndexEntry` carries `valid_from`/`valid_until`; document chunks inherit them
  from their parent document's temporal window.
- CLI: `remember` and `put-document` gain `--valid-from <ms>` and
  `--valid-until <ms>`; `recall` gains `--as-of <ms>`.
- 5 new integration tests: future-valid filtering, expired-memory exclusion,
  as-of point-in-time query, legacy-entry always-valid backwards compatibility,
  and document-chunk validity inheritance.

### Added — eval-quality CLI command

- Added `eval-quality --fixture <file> [--db <path>] [--top-k N] [--json]`
  subcommand to run retrieval fixture evaluations from the CLI.
- Reads the same fixture JSON format as the internal `retrieval_quality` test
  (`version`, `memories`, `cases`, `thresholds`).
- Seed memories into a fresh temp directory (auto-cleaned) or an existing
  database (`--db`); runs each query scenario and records hit@1, hit@k, MRR,
  forbidden-top violations, and required-term coverage.
- Human output: per-scenario PASS/FAIL + aggregate hit@1/hit@k/MRR + threshold
  comparison.
- `--json` output: machine-readable object with `scenarios`, `aggregate`,
  `thresholds`, `threshold_failures`, `pass`.
- Exits 0 on full pass; exits 1 with diagnostic message when any threshold or
  per-case constraint fails.
- No new crate dependencies.
- Added 2 CLI smoke tests (`cli_eval_quality`, `cli_eval_quality_fails_on_bad_thresholds`).

### Changed — RRF Hybrid Fusion

- Replaced `alpha * vector_norm + (1-alpha) * text_norm` with Reciprocal Rank
  Fusion (RRF, k=60) in `SearchMode::Hybrid`. RRF is parameter-free: each
  candidate is ranked independently by cosine similarity and BM25 score; the
  fused score is `1/(60+rank_v) + 1/(60+rank_t)`. Items absent from one
  dimension receive a penalty rank.
- `RecallResult.reason` for hybrid mode now reports
  `hybrid/rrf(vector_rank=N, text_rank=M)`.
- `RecallResult.vector_score` and `text_score` still carry raw cosine and BM25
  values for debugging transparency.
- `hybrid_alpha` in `Config` is retained in the public API for backwards
  compatibility but has no effect on hybrid scoring.
- Removed unused `clamp01` helper.
- Added `RRF_K` as a public constant in `query.rs`.

### Added — Admin CLI v0.1

- Added `list-tenants`, `list-collections`, `list-documents`, `list-memories`,
  `list-records`, and `list-files` commands for full exploration of stored
  objects without launching a server.
- Added `show-document` (includes per-chunk listing), `show-memory`,
  `show-record` (with payload and context projection), and `show-file` commands
  for detailed per-object inspection.
- All new list and show commands accept `--json` for stable, script-friendly
  machine-readable output.
- Updated `inspect` to accept `--json`.
- Added public library APIs: `list_documents`, `list_memories`, `list_records`,
  `list_files`, `all_collections`, `get_document`, `get_memory`, `get_record`,
  `get_file`, `get_document_chunks`.
- Added `serde_json` as a dev-dependency to `hippocore-cli`.

### Added — File ingestion v0.1

- Added first-class `FileObject` metadata for imported text-like local files,
  including path, name, media type, CRC32 checksum, byte size, source, metadata,
  timestamps and version.
- Added `Hippocore::import_file` / `delete_file` and CLI `import-file` /
  `delete-file`.
- Supported `.txt`, `.md`, `.json`, and `.csv` imports without network or
  external parser dependencies.
- Imported file text is projected into a derived document (`file:<id>`) whose
  chunks are indexed and recallable with file metadata.
- Added tests for import, recall, metadata filtering, JSON projection, restart,
  delete, unsupported extensions and CLI import/delete.
- Scoped document chunk replacement by tenant and collection, avoiding
  cross-tenant chunk removal when document ids repeat.

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
- Query ranking stays generic: it relies on normalized text, BM25 and vector
  similarity, without hard-coded Oracle/PostgreSQL entity boosts in the core.
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

- Pure vector search now skips query normalization, keeping the quality layer
  out of the vector-only hot path.

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
