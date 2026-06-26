# Changelog

All notable changes to Hippocore DB are documented here. This project adheres to
[Semantic Versioning](https://semver.org/) (pre-1.0: minor versions may break).

## [Unreleased]

### Added — Compact + Reopen Invariant Tests v0.1

- New test file `crates/hippocore/tests/compact_reopen.rs` with 5 tests: WAL
  length zero after compact, collections / memories / documents survive compact
  + cold reopen, multiple successive compact cycles do not lose any data. No
  production code changes. 174 tests total.

### Added — WAL Recovery Tests v0.1

- New test file `crates/hippocore/tests/wal_recovery.rs` with 4 tests locking
  the torn-write safety contract: clean WAL reopens correctly; half-written
  entry (no newline) is skipped on open without error; checksum-mismatch line
  stops replay and is not applied; all state before a torn entry survives in
  both `get_memory` and `recall`. No production code changes. 169 tests total.

### Added — Collection CRUD Tests v0.1

- New test file `crates/hippocore/tests/collection_crud.rs` with 6 tests
  covering: creation appears in list, idempotency (duplicate create returns
  existing, not error), two tenants sharing a name, tenant-isolated listing,
  recall scoped to nonexistent collection returns empty, description round-trip.
  Key invariant confirmed: `create_collection` is an idempotent upsert. No
  production code changes. 165 tests total.

### Added — Valid-Window Recall Tests v0.1

- New test file `crates/hippocore/tests/valid_window.rs` with 5 deterministic
  tests using fixed epoch-ms constants: expired excluded, future-until included,
  not-yet-valid excluded, `as_of` backdating (inside window → included, current
  time → excluded), no validity window always included. No production code
  changes. 159 tests total.

### Added — Contradiction Advisory Tests v0.1

- New test file `crates/hippocore/tests/contradiction.rs` with 4 tests: both
  contradicting memories appear in recall (advisory only, not a filter), the
  `RecallResult.contradictions` field is populated, `build_context` fires
  confidence-aware re-ranking when contradictions are present (higher confidence
  wins), and when no contradictions exist no re-ranking fires. No production
  code changes. 154 tests total.

### Added — Supersession Lifecycle Tests v0.1

- New test file `crates/hippocore/tests/supersession.rs` with 5 tests: default
  recall hides superseded memories, `include_superseded=true` surfaces them,
  linkage is bidirectional, invariants survive WAL compaction + cold reopen, and
  deleting the superseding memory leaves `superseded_by` as a durable tombstone.
  No production code changes. 150 tests total.

### Added — Metadata Filter Regression Suite v0.1

- New test file `crates/hippocore/tests/metadata_filter.rs` with 7 tests
  covering: exact-match on memories, multi-key AND semantics (env=prod AND
  tier=api), empty result without error, collection-scoped filter, record
  filtering, cross-tenant non-interference, and `build_context`
  `metadata_filter` propagation. No production code changes. 145 tests total.

### Added — Multi-tenant Query Isolation Audit v0.1

- New test file `crates/hippocore/tests/tenant_isolation.rs` with 6 targeted
  isolation assertions: `recall()` (Vector/Text/Hybrid), `build_context()`
  (with and without `include_related`), and `list_graph_edges()`. Tests use two
  tenants with identical collection names and identical content to prove the
  query layer enforces hard tenant isolation on every public read path.
- No production code changes.
- 6 new tests. 138 tests total.

### Added — Score Normalization v0.1

- `build_context` now normalises raw hybrid recall scores to `[0.0, 1.0]`
  relative to the current candidate set before temporal decay and graph-aware
  ranking blend formulas are evaluated. The highest-scoring candidate receives
  normalised score `1.0`; relative ordering is unchanged.
- Normalisation is skipped entirely when both blend weights are `0.0` (the
  default), so there is zero overhead and no behavior change for unblended
  queries.
- 2 new integration tests. 132 tests total.

### Added — Temporal Decay v0.1

- Added `temporal_weight: f32` (default `0.0`) and `temporal_decay_days: f32`
  (default `30.0`) to `Config`.
- `build_context` now applies an optional exponential recency bias before
  graph-aware ranking: `decay = exp(-age_days / decay_days)` blended as
  `recall × (1−w) + decay × w`. Memory items use `created_at`; chunks use
  parent document `updated_at`; records use `updated_at`. Items with
  unresolvable timestamps receive a neutral decay of `0.5`.
- Passing `temporal_weight` outside `[0.0, 1.0]` or `temporal_decay_days ≤ 0`
  with non-zero weight returns a typed `HippocoreError::Validation` error.
- 4 new integration tests. 130 tests total.

### Added — Graph-Aware Ranking v0.1

- Added `graph_rank_weight: f32` to `Config` (default `0.0`; zero means
  disabled — no behavior change for existing databases).
- `build_context` now applies an optional graph-connectivity bonus after
  hybrid recall: for each candidate, the fraction of its direct graph
  neighbours that also appear in the candidate set is blended into the
  effective score (`recall × (1−w) + connectivity × w`). Candidates are
  re-sorted by effective score before the token-budget allocation pass.
- Passing `graph_rank_weight` outside `[0.0, 1.0]` returns a typed
  `HippocoreError::Validation` error before any I/O.
- 3 new integration tests. 126 tests total.

### Added — Audit Retention v0.1

- Added `audit_max_records: usize` and `audit_max_bytes: u64` to `Config`
  (`0` = unlimited by default; no behavior change when unset).
- Added public `AuditRetentionSummary` struct with `records_kept`,
  `records_removed`, `bytes_before`, and `bytes_after` fields.
- Added `Hippocore::compact_audit(max_records, max_bytes)` — rewrites
  `audit.log` atomically (temp file → fsync → rename), keeping only the newest
  records that satisfy both constraints. Caller values override config;
  `Some(0)` disables that constraint for the call.
- `append_audit_record` auto-enforces retention after every `build_context`
  append when `audit_max_records > 0` or `audit_max_bytes > 0` (best-effort;
  append still succeeds even if compaction fails).
- Added `audit_log_bytes: u64` and `audit_records: usize` to `DatabaseStats`.
- `DatabaseStats::Display` now prints audit log bytes and audit record count.
- CLI: new `compact-audit --db <path> [--max-records <n>] [--max-bytes <n>]
  [--json]` subcommand.
- Added 5 core integration tests (default unchanged, max-records, max-bytes,
  query-after-compaction, auto-retention from config) and 1 new CLI smoke test.

### Added — Graph-Aware Context v0.1

- Added `include_related: bool` and `related_limit: usize` to
  `BuildContextRequest` (`false` and `8` by default).
- `build_context` can now optionally append direct graph neighbours of recalled
  items when they fit the token budget.
- Added `ContextItemSource` with `recalled`, `graph_expanded`, and
  `not_included` labels.
- `ContextItem` now exposes `inclusion_source`; `AuditItem` exposes
  `inclusion_source` and uses `not_included` for considered items dropped by
  the budget.
- Graph-expanded items are tenant-isolated, limited by `related_limit`, and
  deduplicated when already present in recall results.
- CLI `build-context` gains `--include-related` and `--related-limit <n>`.
- Added 5 core integration tests and extended the CLI graph edge test to cover
  graph-aware context JSON output.

### Added — Phase 10: Graph Memory v0.1

- Added public `GraphEdge` model for durable direct relationships between
  context items (`Memory`, `DocumentChunk`, `Record`).
- Added `AddGraphEdgeRequest` and public APIs:
  `add_graph_edge`, `list_graph_edges`, `delete_graph_edge`, and
  `graph_neighbors`.
- Persist graph edges through the existing WAL/snapshot model via
  `PutGraphEdge` and `DeleteGraphEdge` operations.
- Endpoint validation is tenant-scoped and rejects unknown or ambiguous
  `(tenant, kind, id)` endpoints.
- Deleting memories, records, documents or imported files removes graph edges
  touching deleted endpoints deterministically.
- Context compiler and RAG audit records now expose `related_item_ids` for
  directly related neighbours.
- Added CLI commands: `add-edge`, `list-edges`, and `delete-edge`; `add-edge`
  and `list-edges` support `--json`.
- Added 6 core integration tests and 1 CLI smoke test covering happy path,
  validation, tenant isolation, restart durability, dangling-edge cleanup and
  context/audit provenance.

### Added — Phase 9: RAG Audit Engine

- Added public `AuditRecord` and `AuditItem` types for query-time retrieval
  auditability.
- `Hippocore::build_context(req)` now appends one JSON-lines record to
  `<data_dir>/audit.log` for every context compilation, including timestamp,
  tenant, query, mode, token budget, final token count, compilation latency,
  dropped count, retrieved item ids, kinds, scores, token counts, confidence and
  inclusion status.
- Added `Hippocore::query_audit(from_ms, to_ms, tenant_id)` to replay audit
  records for a tenant in an inclusive epoch-ms window.
- Added CLI `audit --from <ms> --to <ms> --tenant <t> [--json]`; JSON output is
  the stable serialized `Vec<AuditRecord>`.
- Audit logging is append-only and separate from the mutation WAL, so query
  traces survive reopen without changing database state recovery.
- Added 3 core integration tests for audit write/replay/reopen plus 1 CLI JSON
  smoke test. Existing Phase 7 confidence rating tests cover the Phase 9
  `rate_memory` acceptance criteria.

### Added — Phase 6: Interactive TUI (`hippocore studio`)

- New `hippocore studio [--db <path>]` subcommand launching a full-terminal ratatui
  interface for browsing and querying any Hippocore database without writing code.
- Four navigable tabs via `1`–`4` or `Tab`/`Shift-Tab`:
  - **Tenants** — lists all tenants with their collections and descriptions.
  - **Memories** — live fuzzy search with `/` across all tenants (calls `recall()`).
  - **Documents** — lists all stored documents with tenant/collection provenance.
  - **Stats** — shows tenants, collections, memories, documents, chunks, records,
    files, index entries, WAL entries, and disk usage.
- Scroll with `↑↓` or `jk`; refresh with `r`; quit with `q` or `Ctrl-C`.
- The binary's `main.rs` intercepts `studio` before delegating to `hippocore::cli`,
  so all existing subcommands are unaffected.
- `ratatui 0.29` and `crossterm 0.28` added to workspace and `hippocore-cli` deps.
- 1 new CLI smoke test (`cli_studio_exits_without_panic`). 105 tests total.

### Added — Phase 4: Pluggable VectorIndex trait + HNSW from scratch

- `VectorIndex` trait: `insert(key, embedding)`, `remove(key)`, `knn(query, k, ef)`.
- `BruteForceVectorIndex`: exact cosine scan, O(n) per query.
- `HnswVectorIndex`: wraps `HnswIndex` for sub-linear ANN search.
- `HnswIndex` (pure Rust, zero external deps): multi-layer graph, two-heap beam search
  (W = min-heap, C = max-heap), soft deletes, geometric level sampling, bidirectional
  connections with neighbour pruning.
- `Index::with_vector_backend(kind)` factory; insert/remove propagate to the backend.
- `Index::knn(query, k, ef)` delegate to the active backend.
- `Config::vector_index: VectorIndexKind` (default `BruteForce`).
- `query::execute` uses `index.knn()` with `KNN_OVER_FETCH=8` over-retrieval for
  post-filter correctness in vector/hybrid modes.
- `VectorIndexKind` re-exported from the crate root.
- 4 new HNSW unit tests; 3 new integration tests (store+recall, restart, BF vs HNSW).

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
