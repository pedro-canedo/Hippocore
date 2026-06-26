# Hippocore DB — Status

_Last updated: 2026-06-26._

## What was implemented last

**Phase 4 — Pluggable `VectorIndex` trait + HNSW**, **Phase 7 — Confidence-aware resolution**, and **Phase 2 — Group-commit / batch writes**:

### Phase 7: Source and confidence-aware resolution

- `confidence: Option<f32>` field added to `Memory`, `RecallResult`, `ContextItem`, and `RememberRequest`. All are `#[serde(default)]` — existing WAL entries recover as `None` (unrated).
- `Memory::validate()` enforces `confidence ∈ [0.0, 1.0]`.
- New `Hippocore::rate_memory(tenant, collection, id, confidence)` — human-in-the-loop rating API; writes durably via WAL `PutMemory`.
- `IndexEntry` carries `confidence` and propagates it through `build_result` to `RecallResult`.
- `build_context` applies confidence-aware re-ranking when contradictions exist among candidates: `effective_score = recall_score × 0.7 + confidence × 0.3`. Unrated items use `confidence = 0.5` (neutral), so they are not penalised.
- `ContextItem` gains `confidence: Option<f32>` for caller introspection.
- CLI: `remember --confidence <f32>` and new `rate-memory --tenant --collection --id --confidence` subcommand.
- 6 new integration tests: store+recall confidence, out-of-range rejection, `rate_memory` persistence, invalid rate, not-found error, conflict-ordering in `build_context`.

### Phase 4: Pluggable VectorIndex trait + HNSW from scratch

- `VectorIndex` trait in `index.rs`: `insert(key, embedding)`, `remove(key)`, `knn(query, k, ef) -> Vec<EntryId>`.
- `BruteForceVectorIndex`: exact O(n) cosine scan; the default backend for correctness.
- `HnswVectorIndex`: wraps `HnswIndex` from the new `hnsw.rs` module; sub-linear ANN.
- `HnswIndex` (pure Rust, no external deps): multi-layer graph, proper two-heap beam search (W = min-heap, C = max-heap), soft deletes, geometric level sampling `level = floor(-ln(U) × mL)`, bidirectional connections with pruning.
- `Index` gains `with_vector_backend(kind: VectorIndexKind)` factory; inserts/removes propagate to the vector backend automatically.
- `Index::knn(query, k, ef)` delegates to the active backend.
- `Config::vector_index: VectorIndexKind` (default `BruteForce`) — callers select the backend at open time.
- `query::execute` uses `index.knn()` with `KNN_OVER_FETCH=8× over-retrieval` for post-filter correctness in hybrid/vector modes.
- `VectorIndexKind` re-exported from the crate root.
- 4 new HNSW unit tests + 3 new integration tests (store+recall, restart, brute-force vs HNSW agreement).
- No new crate dependencies. 104 tests total.

### Phase 2: Group-commit / batch writes

- `Storage::append_many(ops: &[Operation])` — serializes all ops to one buffer, single `write_all`, single conditional `sync_all`. One fsync for N ops.
- `Hippocore::remember_many(reqs: Vec<RememberRequest>)` — validates all requests up-front, builds the ops vec, calls `append_many`, then applies state and index updates in a loop. Auto-compaction check at the end.
- `Hippocore::store_documents(reqs: Vec<StoreDocumentRequest>)` — same group-commit pattern for documents.
- 3 new integration tests: batch memories, batch documents, batch durability across restart.
- No new crate dependencies. 94 tests total.

Previous: **Context Compiler** — `build_context(query, user, max_tokens)`:

- New `BuildContextRequest` type: tenant, query, `max_tokens` (default 2048),
  `top_k_candidates` (default 20), `mode` (default `Hybrid`), optional
  `collection` and `metadata_filter`.
- New `ContextBlock` return type: `text` (LLM-ready string), `token_count`
  (1 token ≈ 4 bytes), `items_included: Vec<ContextItem>`, `items_dropped`.
- `ContextItem` carries `id`, `kind`, `score`, `token_count`, and a 120-char
  `snippet`.
- `Hippocore::build_context(req)` recalls up to `top_k_candidates` items,
  sorts by score descending, then greedily fills the token budget — items that
  would exceed `max_tokens` are counted in `items_dropped`, not silently lost.
- Each included item formatted as `[<kind>:<id>]\n<text>`, blocks separated
  by `\n\n`.
- CLI: `build-context --tenant <t> --query "..." [--max-tokens 2048]
  [--top-k 20] [--mode hybrid] [--collection c] [--json]`.
- `--json` output: `{text, token_count, items_included, items_dropped}`.
- 3 new integration tests: budget enforcement, score-ordering + provenance,
  `token_count` self-consistency.
- No new crate dependencies. 85 tests total.

Previous: **Temporal Truth Layer full spec** — supersedes / contradicts relations:

- `Memory` gains `supersedes: Vec<String>`, `contradicts: Vec<String>`, and
  `superseded_by: Option<String>` (all `#[serde(default)]` for WAL compatibility).
- `RecallResult` gains `contradictions: Vec<String>` (advisory ids surfaced in results).
- `RememberRequest` gains `supersedes` and `contradicts` lists; `RecallRequest`
  gains `include_superseded: bool` (default `false`).
- `remember` validates that every supersedes/contradicts id exists in the same
  tenant and marks each superseded memory with `superseded_by = Some(new_id)` in
  the WAL immediately (durable).
- `Filter::matches` skips superseded memories unless `include_superseded = true`.
- `IndexEntry` carries `superseded: bool` and `contradicts: Vec<String>` for
  fast filtering without loading the full memory.
- CLI: `remember --supersedes <id>...`, `remember --contradicts <id>...`,
  `recall --include-superseded`.
- 5 new integration tests: supersedure exclusion, include-superseded flag,
  contradictions surface in result, unknown-id validation error, durability
  across restart.
- No new crate dependencies. 82 tests total.

Previous: **Benchmark regression guard**:

- Added `examples/bench_regression.rs`: a standalone latency guard for hybrid,
  vector, and text recall over 500 memories (50 measured iterations after 5
  warmup iterations).
- Committed `benches/baseline.json` with current mean latencies (~430–470 µs
  on the development machine).
- `HIPPO_BENCH_UPDATE=1 cargo run --release --example bench_regression` rewrites
  the baseline; without that env var the command reads the baseline and exits
  non-zero if any benchmark regresses by more than `HIPPO_BENCH_THRESHOLD`
  (default 20%).
- Satisfies Phase 5's final item: benchmark regression guard.
- No new crate dependencies (uses existing `serde`, `serde_json`, `tempfile`).

Previous: **Temporal Truth Layer v0.1**:

- Added `valid_from: Option<i64>` and `valid_until: Option<i64>` to `Memory`
  and `Document` (epoch milliseconds; `None` = no constraint). Fields are
  `#[serde(default)]` so all existing serialized records recover as always-valid.
- `RememberRequest` and `StoreDocumentRequest` gain matching fields.
- `RecallRequest` gains `as_of: Option<i64>`. Default (`None`) resolves to the
  current time at query time, so expired entries are excluded by default.
- `recall --as-of <ms>` CLI flag for historical point-in-time queries.
- `remember --valid-from / --valid-until` and `put-document --valid-from /
  --valid-until` CLI flags.
- Temporal filtering lives in `Filter::matches`; it applies after all other
  filters and respects the per-entry `valid_from`/`valid_until` from `IndexEntry`.
- Document chunks inherit their parent document's temporal validity window.
- No schema migration needed: legacy WAL entries decode with `None/None`
  (always valid) via serde default.
- 5 new integration tests: future-valid filtering, expired-memory exclusion,
  as-of past-state query, legacy-entry backwards compat, chunk validity inheritance.
- No new crate dependencies.
- 76 tests total.

Previous: **eval-quality CLI**:

- Added `eval-quality` subcommand: reads a retrieval fixture JSON file, seeds
  memories into a temp (or specified) database, runs each scenario, and reports
  hit@1, hit@k, MRR per scenario and aggregate.
- `--json` emits a machine-readable report with `scenarios`, `aggregate`,
  `thresholds`, `threshold_failures`, and `pass` fields.
- Exits non-zero when any threshold is not met; human output marks failing
  scenarios with `FAIL` and prints threshold violations.
- `--db <path>` allows evaluation against an existing database.
- `--top-k` configures K (default 5).
- Uses the same fixture JSON format as the internal `retrieval_quality` test.
- No new crate dependencies.
- 2 new CLI smoke tests (pass + deliberate fail path).
- 71 tests total.

Previous increment: **RRF Hybrid Fusion**:

- Replaced `alpha * vector_norm + (1-alpha) * text_norm` with Reciprocal Rank
  Fusion (RRF, k=60) in `SearchMode::Hybrid`.
- RRF is parameter-free: `score = 1/(60+rank_v) + 1/(60+rank_t)`, where each
  rank is 1-based within the sorted vector or text candidate list. Items with no
  signal in one dimension receive a penalty rank beyond the candidate set.
- `RecallResult.reason` for hybrid mode now reports
  `hybrid/rrf(vector_rank=N, text_rank=M)` for transparency.
- `RecallResult.vector_score` and `text_score` still carry raw cosine and BM25
  scores (not the ranks) for debugging.
- `hybrid_alpha` in `Config` is kept in the API for backwards compatibility but
  has no effect on hybrid scoring.
- `SearchMode::Vector` and `SearchMode::Text` are unchanged (still min-max
  normalized cosine and BM25 respectively).
- Added `RRF_K` as a public constant in `query.rs`.
- Removed dead `clamp01` helper.
- Added 2 new unit tests for the RRF constant and score-vs-rank ordering.
- All 69 tests pass; retrieval quality fixture thresholds are met.

Previous increment: **Admin CLI v0.1**:

- Added `list-tenants`, `list-collections`, `list-documents`, `list-memories`,
  `list-records`, and `list-files` commands for full database exploration.
- Added `show-document` (with chunk listing), `show-memory`, `show-record`, and
  `show-file` commands for detailed object inspection.
- All list and show commands support `--json` for stable machine-readable output
  ready for scripts and future Studio integration.
- Updated `inspect` to support `--json`.
- Added corresponding public library APIs: `list_documents`, `list_memories`,
  `list_records`, `list_files`, `all_collections`, `get_document`,
  `get_memory`, `get_record`, `get_file`, `get_document_chunks`.
- Added `serde_json` as a dev-dependency to `hippocore-cli` (needed for CLI
  integration tests parsing JSON output).
- Added 3 new CLI smoke tests covering all admin commands and `--json` output.

Previous increment: **File ingestion v0.1**:

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
- The query layer normalizes common PostgreSQL typos/aliases and connection
  variants before lexical scoring and built-in query embedding.
- Query ranking stays generic: it relies on normalized text, BM25 and vector
  similarity, without hard-coded Oracle/PostgreSQL entity boosts in the core.
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
  `vector_score`, `text_score`, and a `reason` that explains the scoring path.
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

**All green.** `cargo test --workspace` passes 104 tests:
- 22 unit (embedder/chunker, cosine/index/BM25, query normalization + RRF, CRC32 + WAL
  line decode, 4 new HNSW unit tests),
- 64 library integration (store/recall, chunking, retrieval quality layer,
  structured records, imported files, user-supplied document embeddings +
  validation, modes, sorting, metadata & type/user filters, tenant isolation,
  restart, compaction, auto-compaction bounding the WAL, checksum-mismatch
  recovery, delete/forget + restart + compaction, user embeddings, empty DB,
  error paths, dimension-mismatch, torn-WAL recovery, temporal filtering,
  supersedure/contradictions, context compiler, batch writes, confidence-aware
  resolution, HNSW backend store+recall+restart+brute-force comparison),
- 1 retrieval-quality fixture test (hit@1/hit@5/MRR thresholds),
- 12 CLI subprocess smoke tests (incl. `compact`, `forget`, `put-record`,
  `import-file`, admin `list-*` and `show-*` commands with `--json`, and
  `eval-quality` pass + fail paths),
- 1 doctest (lib.rs quickstart),
- 1 bench_regression example.

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

**Phase 6 — Interactive TUI with ratatui** — `hippocore studio` subcommand with panels for tenants, collections, memories, search and stats. See NEXT_FEATURE.md.
