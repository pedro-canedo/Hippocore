# Hippocore DB — Status

_Last updated: 2026-06-27._

## What was implemented last

**Control Plane TS — Ingestion v0.5**:

- New **Ingestion** page in `/console`, gated on an active tenant and a target
  collection selector (reusing the per-tenant collection memory).
- Three typed forms create data in the active scope: **Memory**
  (text + type semantic/episodic/procedural), **Record** (table + JSON payload
  validated client-side), and **Document** (text), via the existing
  `POST /admin/tenants/{tid}/{memories,records,documents}` endpoints.
- Each success clears the form, shows a toast, and refreshes Dashboard counts;
  typed `ApiError` renders inline and invalid Record JSON is caught before send.
- The TypeScript console now covers the full create → browse → query loop
  without falling back to `/admin`.
- No server changes; assets rebuilt and embedded.

Previous: **Control Plane TS — SQL Editor v0.4**:

- New **SQL Editor** page in `/console`, gated on an active tenant, running the
  existing read-only `POST /admin/sql` (`SELECT` over record payloads).
- A monospace editor with a Run button and `Ctrl/Cmd + Enter`; example snippets
  populate the editor without auto-running.
- Successful results render as a table with deterministic, sorted payload
  columns (id/table stay as system columns), a Raw JSON toggle, and the
  client-observed row count and duration.
- Typed `ApiError` surfaces parse/validation errors inline without crashing.
- Shared `format.ts` helper (record columns, cell, truncate) now backs both the
  Data Explorer and the SQL Editor.
- No server changes; assets rebuilt and embedded.

Previous: **Control Plane TS — Data Explorer (read) v0.3**:

- New **Data Explorer** page in `/console`, gated on an active tenant, with a
  collection selector whose choice is remembered per tenant in local storage.
- Entity-type tabs (Records, Memories, Documents, Files) load in parallel for
  the selected scope and show per-type counts; clicking a row opens a detail
  drawer with the full JSON of that object.
- Record rows render their payload keys as deterministic, sorted columns while
  `id` and `table` stay as stable system columns; the other types show their
  core fields.
- Typed API client extended with `listRecords`, `listMemories`,
  `listDocuments`, `listFiles` over the existing `/admin/*` endpoints; reads are
  tenant-scoped so only the active tenant's data is shown.
- No server changes; assets rebuilt and embedded.

Previous: **Control Plane TS — Navigation Shell + Tenants & Collections v0.2**:

- The `/console` app gains a persistent sidebar shell (Dashboard, Tenants,
  Collections) with client-side view switching — no router library — plus a
  topbar tenant selector that persists the active tenant in local storage,
  sharing the `hippocore.tenant` key with the classic `/admin` console.
- **Tenants** page lists and creates tenants (`GET`/`POST /admin/tenants`);
  **Collections** page lists and creates collections for the active tenant
  (`GET /admin/collections?tenant_id=`, `POST /admin/tenants/{tid}/collections`),
  requiring a tenant to be selected first.
- Forms use the typed API client; typed `ApiError` surfaces server messages
  inline, success refreshes lists and bootstrap counts, and shows a toast.
- Dashboard was refactored to render inside the shell, with stat cards and
  links that navigate into Tenants/Collections.
- No server changes: all endpoints already existed; assets rebuilt and embedded.

Previous: **Control Plane TypeScript Rebuild — Foundation v0.1**:

- New React + Vite + TypeScript admin console under
  `crates/hippocore-server/admin-ui/`, served at **`/console`**. The classic
  zero-build vanilla-JS console stays at **`/admin`** during the migration.
- First migrated slice is a real **Login → Dashboard** flow: authenticates via
  `POST /admin/login`, then renders server status and bootstrap stats (tenant,
  collection, memory, document, record, file, graph-edge and index counts) from
  `GET /admin/bootstrap`, with refresh and sign-out. Typed API client and
  session module establish the pattern for future page ports.
- `vite build` emits flat, non-hashed assets (`index.html`, `index.js`,
  `index.css`) into `crates/hippocore-server/src/console/`, which is committed
  and embedded into the binary with `include_str!`. This keeps `cargo build`
  and the Docker image free of any Node dependency, and the `dist/` name is
  avoided because `.dockerignore` excludes `**/dist`.
- Three new public routes (`/console`, `/console/index.js`, `/console/index.css`)
  serve the embedded assets; no existing route or behavior changed.
- Frontend quality gate adds `pnpm type-check` (`tsc --noEmit`) and `pnpm build`
  alongside the Rust gate. New integration test asserts the console page and
  assets are public with the correct content types.
- See [CONTROL_PLANE_TS.md](CONTROL_PLANE_TS.md) and ADR-016.

Previous: **Guided Workspace Onboarding v0.1**:

- Dashboard now keeps a five-step checklist visible from empty database through
  tenant, collection, first data, SQL, and recall/context completion.
- Bootstrap stats drive the first three steps. SQL and recall success flags are
  browser-local and scoped by `data_dir`; no database state is mutated.
- Only the first incomplete step has a primary action, and the same next action
  appears in the sidebar until setup is complete.
- Operational status, active tenant, storage, API status, data counts, and quick
  actions are separate sections.
- Raw stats JSON was removed from the Dashboard and remains available through
  Observability.
- English and Portuguese catalogs now contain 297 matching, non-duplicate keys.

Previous: **Data Explorer Workspace v0.1**:

- The Explorer now supports tenant -> collection -> data-type navigation for
  Records, Memories, Documents, and Files without leaving the page.
- Collection search filters the active tenant tree, and contextual actions lead
  to collection creation or first-item ingestion.
- The four existing tenant-scoped list endpoints load in parallel for the
  active collection, providing per-type counts and instant type switching.
- Record payload keys become deterministic logical columns; id and table remain
  stable system columns.
- Raw JSON shows complete objects. Physical Info shows kind, lineage, source,
  metadata, timestamps, version, and opens the complete object in the drawer.
- English and Portuguese catalogs now contain 280 matching keys.

Previous: **Control Plane i18n Completeness v0.1**:

- All runtime Control Plane copy now comes from matching English and
  Portuguese catalogs, including navigation, pages, forms, actions, errors,
  toasts, integrations, prompts, and observability.
- Technical identifiers, SQL, endpoint paths, provider names, and stored user
  data retain their precise original forms.
- Switching language rerenders the active page while preserving transient form
  values, tenant/collection selection, results, session, and detail state.
- Operation and toast messages retain translation keys, so status already on
  screen changes language too.
- Integration tests assert bilingual catalog markers and reject known hardcoded
  runtime page copy regressions.

Previous: **Ingestion & Recall UX Hardening v0.1**:

- Collection inputs on the Ingestion & Recall page autocomplete from the
  selected tenant's loaded collections.
- Memory, Document, Record, and File type cards now show only their relevant
  fields and action; File opens the existing upload page.
- Recall results render as cards with kind, score, matched terms, text, and a
  copy-id action, with Raw JSON still available.
- `build_context` renders token count, included/dropped item counts, and the
  formatted context block, with Raw JSON still available.
- The HTTP recall response now preserves the core `matched_terms` field.
- The updated workflow has complete English and Portuguese copy.

Previous: **File Upload with Drag-and-Drop v0.1**:

- New `POST /admin/tenants/{tid}/files` multipart endpoint (axum `multipart` feature).
  Dispatch by extension:
  - `.pdf` → `store_document(from_pdf(...))` — chunked Document for recall
  - `.txt`, `.md` → `store_document(new(...))` — plain text Document
  - `.csv` → inline CSV parser → one `put_record()` per row; table = filename stem
  - `.json` — array-of-objects → Records; any other JSON → Document
  - other extensions → `422 Unprocessable Entity`
  - Response: `{ file_id, kind, count, name }`
- New `handlers/files.rs` module following the existing handler pattern.
- Files page now shows a Liquid Glass drag-and-drop zone with file type badges,
  XHR upload with live progress bar, and a success banner after upload.
- Dragging files onto `#dropZone` or clicking to browse both trigger upload.
- `styles.css`: new `.drop-zone`, `.drag-over`, `.drop-zone-progress`, `.drop-zone-progress-bar`,
  `.file-type-badge`, `.upload-result`, `.radio-card` classes.

Previous: **Control Plane — Liquid Glass UI v0.1**:

- Complete visual redesign of `/admin` into a premium Liquid Glass interface:
  translucent cards with `backdrop-filter: blur`, radial gradient body,
  cyan glow accent, smooth hover transitions, slide-in drawer, toast
  notifications, and loading/empty states with guidance text.
- New `Documentation` page with embedded bilingual docs (9 sections:
  Overview, Quickstart, Concepts, Data Model, SQL Guide, RAG/Context Guide,
  API Authentication, CLI Guide, Architecture) and sticky sidebar navigation.
- `DashboardPage()` now shows a welcome card with 4 onboarding steps when
  no tenants exist, guiding new users through tenant → collection →
  ingest → recall.
- Empty states across all pages (Tenants, Collections, Records, Memories,
  Documents, Files, Data Explorer) now include actionable guidance text
  and contextual next-step buttons.
- New quick-action card grid on Dashboard with branded `.quick-action-card`
  CSS class and arrow indicator.
- Navigation shortcuts (`go-tenants`, `go-collections`, `go-sql`,
  `go-explorer`, `go-ingest`, `go-keys`, `go-docs`) from any empty state
  or onboarding step.
- `index.html` updated to import Inter font family via Google Fonts for
  premium typography.
- All existing functionality preserved; no pages removed; PT/EN i18n maintained.
- Quality gate: `cargo fmt`, `cargo test --workspace`, `cargo clippy -D warnings` all green.

Previous: **MMR (Maximal Marginal Relevance) v0.1**:

- Added `mmr: bool` and `mmr_lambda: f32` to `RecallRequest` (defaults: `false`, `0.5`).
- When enabled, results are reranked by greedily selecting items that maximise
  `lambda * relevance - (1-lambda) * max_sim_to_selected`. Items without
  embeddings are appended last.
- `lambda=1.0` degenerates to pure score order; `lambda=0.0` to pure diversity.
- CLI: `--mmr` and `--mmr-lambda` flags on `hippocore recall`.
- HTTP: `{"mmr": true, "mmr_lambda": 0.5}` in the recall body.
- Two unit tests: diverse pick at `lambda=0.5`; score order preserved at `lambda=1.0`.

Previous: **Matched Terms v0.1**:

- Added `matched_terms: Vec<String>` to `RecallResult` (serde default `[]`).
- For hybrid and text recall modes, the field contains the intersection of
  tokenized query terms and result text tokens (lowercase, deduplicated, sorted).
- Pure vector recall always produces an empty list (no text token comparison).
- Existing serialized results that omit the field deserialize without error.
- Two unit tests: intersection is correct, no-overlap yields empty list.

Previous: **Document Chunk Deduplication v0.1**:

- Added `dedup_chunks: bool` to `RecallRequest` (default `false`).
- When enabled, only the highest-scoring chunk per parent document is returned.
  Memories, records, and other non-chunk items are always kept.
- Results are already sorted by score before dedup runs, so the first
  occurrence of each `document_id` is always the best chunk.
- Exposed as `--dedup-chunks` in the CLI and `{"dedup_chunks": true}` in HTTP.
- Two unit tests verify: dedup keeps best chunk and drops the rest; `false`
  keeps all items unchanged.

Previous: **Recall Min-Score Filter v0.1**:

- Added `min_score: Option<f32>` to `RecallRequest` (default `None`).
- After confidence weighting and sorting, results below the threshold are
  dropped before `top_k` truncation. Inclusive lower bound.
- Added `--min-score` flag to `hippocore recall` CLI command.
- HTTP recall endpoint accepts `{"min_score": 0.5}` in the request body.
- Three unit tests verify: threshold drops items, `None` keeps all, exact
  threshold is inclusive.

Previous: **Confidence-Weighted Recall v0.1**:

- Memory items with an explicit `confidence` score now influence their final
  ranking position in all recall modes (vector, text, hybrid).
- Formula: `final_score = base_score * (1.0 + 0.4 * (confidence - 0.5))`.
  A memory at confidence=1.0 receives a 20% boost; at 0.0, a 20% penalty.
- Items with no confidence set (the common case) and non-Memory items are
  neutral (no change to relative ranking).
- Change is contained in `query.rs`; storage, model, and CLI are unchanged.
- Three unit tests verify the weighting direction, neutral-case invariance,
  and bound guarantees.

Previous: **Admin CLI v0.1**:

- Added `hippocore tenants list` and `hippocore tenants create` nested
  subcommands, backed by explicit `create_tenant` calls (tenants and collections
  were previously only created implicitly as side effects).
- Added `hippocore collections list` and `hippocore collections create`
  subcommands.
- All existing flat commands (`list-tenants`, `list-collections`, `put-record`,
  `recall`, etc.) remain unchanged.
- Five unit tests verify argument parsing and round-trip create/list flows for
  the new subcommands.

Previous: **Admin Data Actions v0.1**:

- Added `POST /admin/tenants/:tid/records` handler backed by `put_record`.
- Ingestion & Recall page now supports Memory, Document, and Record creation.
  Record type accepts a JSON object payload, a collection, and a table name.
- File import via browser remains Planned and is labelled explicitly in the UI.
- Added two integration tests: success path and tenant isolation enforcement.

Previous: **Control Plane Foundation v0.1**:

- `/admin` was restructured from a single debug-style page into a Control Plane
  app shell with fixed sidebar, topbar, global tenant selector, session status,
  refresh action, breadcrumbs, responsive layout, and a modern dark theme.
- New domain pages: Dashboard, Data Explorer, SQL Editor, Collections, Records,
  Memories, Documents, Files, Ingestion & Recall, Graph, API Reference, Tenants,
  Service Keys, Observability, and Settings.
- Dashboard surfaces server status, current tenant, object counts, audit log
  size, data directory, and last operation.
- Data Explorer separates logical view, raw JSON, and physical/internal info,
  with tenant/collection navigation and detail drawer.
- SQL Editor uses the existing `POST /admin/sql` endpoint and keeps tenant
  isolation explicit.
- Physical/internal objects are separated through Records/Memories/Documents/
  Files/Graph pages plus Observability placeholders for chunks, WAL, snapshot,
  audit, logs, and index status.
- Existing provider registry save/validate workflows remain available from the
  Service Keys page.
- Static admin asset tests now assert the Control Plane navigation and reusable
  component foundation.

Previous: **Phase 12 — Multimodal Storage ✅**:


- `StoreDocumentRequest` gains `content_type: Option<String>` and
  `raw: Option<Vec<u8>>` (binary payload for PDF).
- New module `crates/hippocore/src/ingest.rs` with content-type dispatch.
- **PDF** (`application/pdf`): `from_pdf(tenant, col, bytes)` constructor;
  text extracted via `pdf-extract` (pure Rust, no C deps). Extraction errors
  return `HippocoreError::Validation`, never panic.
- **Code files** (`text/x-<lang>`): `from_code(tenant, col, text, language)`;
  heuristic chunker splits at top-level definition boundaries. Supported
  languages: rust, python, javascript, typescript, go, java, kotlin.
  Unrecognised languages fall back to the standard token chunker.
- 8 integration tests + 6 unit tests. **223 tests total.**
- Documentation in `docs/en/MULTIMODAL_INGEST.md` and `docs/pt-br/`.

Previous: **Phase 11 — HTTP Server ✅**:

- New crate `crates/hippocore-server`: axum 0.8 REST/JSON server over the
  full core API. Shared `Arc<Mutex<Hippocore>>` state.
- `GET /health` (no auth), `GET /stats`, `POST /tenants`,
  `POST /tenants/:tid/collections`, `POST /tenants/:tid/memories`,
  `DELETE /tenants/:tid/memories/:id`, `POST /tenants/:tid/recall`,
  `POST /tenants/:tid/context`, `POST /tenants/:tid/documents`,
  `POST /tenants/:tid/graph/traverse`.
- `X-Api-Key` header auth; unauthenticated requests return 401.
- `hippocore serve [--port 8080] [--db ./data] [--api-key KEY]` CLI subcommand.
- 7 integration tests in `crates/hippocore-server/tests/server_integration.rs`.
- **Phase 11 — Server mode is now 100% complete.**
- Documentation in `docs/en/SERVER.md` and `docs/pt-br/SERVER.md`.
- 8 new tests (7 integration + 1 doc-test). **209 tests total.**

Previous: **GraphRAG Multi-hop Traversal — Phase 10 ✅**:

- New public API: `traverse_graph(TraverseGraphRequest) -> Vec<TraversalNode>`
- BFS traversal from seed items up to `max_hops` deep (default 2).
- Fields: `tenant_id`, `seed_ids`, `max_hops`, `max_nodes`, `relation_filter`.
- `TraversalNode` returns `id`, `kind`, `hop`, `via_edge_id`.
- Tenant-isolated: edges from other tenants are never followed.
- 6 new integration tests in `crates/hippocore/tests/graph_traversal.rs`.
- **Phase 10 — Graph Memory is now 100% complete.**
- Documentation in `docs/en/GRAPH_TRAVERSAL.md` and `docs/pt-br/`.
- 6 new tests. 201 tests total.

Previous: **build_context Smoke Tests v0.1**:

- New test file `crates/hippocore/tests/build_context_smoke.rs` with 5 tests.
- Covers: non-empty text output, snippet in context string, `max_tokens`
  budget enforced, `include_related = true` graph expansion, empty store
  returns `text = ""` (not an error).
- No production code changes.
- Documentation in `docs/en/BUILD_CONTEXT_SMOKE.md` and
  `docs/pt-br/BUILD_CONTEXT_SMOKE.md`.
- 5 new tests. 195 tests total.

Previous: **Document Chunking Tests v0.1**:

- New test file `crates/hippocore/tests/document_chunking.rs` with 5 tests.
- Key API finding documented: `RecallResult.id` is the **chunk id**; the
  parent document id is in `RecallResult.document_id`.
- Covers: ≥1 chunk produced, `document_id` in recall result, disjoint chunks
  between two documents, chunks survive compact + reopen, delete removes chunks.
- No production code changes.
- Documentation in `docs/en/DOCUMENT_CHUNKING.md` and
  `docs/pt-br/DOCUMENT_CHUNKING.md`.
- 5 new tests. 190 tests total.

Previous: **Graph Edge Lifecycle Tests v0.1**:

- New test file `crates/hippocore/tests/graph_edge_lifecycle.rs` with 5 tests.
- Covers: added edge appears in list, relation type round-trips, deleted edge
  absent from list, edges survive compact + reopen, tenant isolation.
- Key API note documented: `list_graph_edges` returns `Vec<GraphEdge>` directly
  (not `Result`).
- No production code changes.
- Documentation in `docs/en/GRAPH_EDGE_LIFECYCLE.md` and
  `docs/pt-br/GRAPH_EDGE_LIFECYCLE.md`.
- 5 new tests. 185 tests total.

Previous: **Record CRUD Tests v0.1**:

- New test file `crates/hippocore/tests/record_crud.rs` with 6 tests.
- Covers: store + retrieve by id, appears in recall, version increments on
  update (0 → 1), delete removes from get and recall, tenant isolation on both
  id-lookup and recall.
- Key invariants confirmed: initial version is 0; `delete_record` is durable.
- No production code changes.
- Documentation in `docs/en/RECORD_CRUD.md` and `docs/pt-br/RECORD_CRUD.md`.
- 6 new tests. 180 tests total.

Previous: **Compact + Reopen Invariant Tests v0.1**:

- New test file `crates/hippocore/tests/compact_reopen.rs` with 5 tests.
- Covers: WAL length zero after compact, collections / memories / documents
  survive compact + cold reopen, multiple compact cycles preserve all state.
- No production code changes.
- Documentation in `docs/en/COMPACT_REOPEN.md` and
  `docs/pt-br/COMPACT_REOPEN.md`.
- 5 new tests. 174 tests total.

Previous: **WAL Recovery Tests v0.1**:

- New test file `crates/hippocore/tests/wal_recovery.rs` with 4 tests.
- Covers: clean WAL cold reopen, truncated trailing line skipped (no panic),
  checksum mismatch stops replay (corrupt entry not applied), state before torn
  entry fully intact in both `get_memory` and `recall`.
- WAL format reminder: `<crc32-hex>\t<json>\n`.
- No production code changes.
- Documentation in `docs/en/WAL_RECOVERY.md` and `docs/pt-br/WAL_RECOVERY.md`.
- 4 new tests. 169 tests total.

Previous: **Collection CRUD Tests v0.1**:

- New test file `crates/hippocore/tests/collection_crud.rs` with 6 tests.
- Key finding documented: `create_collection` is **idempotent** (returns the
  existing collection on duplicate, no error) — safe to use as an upsert.
- Covers: creation appears in list, idempotency, two tenants share names,
  collection list tenant-isolated, recall on nonexistent returns empty,
  description round-trip.
- No production code changes.
- Documentation in `docs/en/COLLECTION_CRUD.md` and
  `docs/pt-br/COLLECTION_CRUD.md`.
- 6 new tests. 165 tests total.

Previous: **Valid-Window Recall Tests v0.1**:

- New test file `crates/hippocore/tests/valid_window.rs` with 5 tests using
  deterministic epoch-ms constants (FAR_PAST=2001, FAR_FUTURE=~year 2255).
- Covers: expired excluded, future-until included, not-yet-valid excluded,
  `as_of` backdating inside vs outside window, no validity window always
  included.
- No production code changes.
- Documentation in `docs/en/VALID_WINDOW_RECALL.md` and
  `docs/pt-br/VALID_WINDOW_RECALL.md`.
- 5 new tests. 159 tests total.

Previous: **Contradiction Advisory Tests v0.1**:

- New test file `crates/hippocore/tests/contradiction.rs` with 4 tests.
- Confirms: both contradicting memories appear in recall (advisory, not filter);
  `RecallResult.contradictions` is populated; `build_context` fires confidence-
  aware re-ranking when contradictions present (high-confidence wins); when no
  contradictions, no re-ranking and recall ordering preserved.
- No production code changes.
- Documentation in `docs/en/CONTRADICTION_ADVISORY.md` and
  `docs/pt-br/CONTRADICTION_ADVISORY.md`.
- 4 new tests. 154 tests total.

Previous: **Supersession Lifecycle Tests v0.1**:

- New test file `crates/hippocore/tests/supersession.rs` with 5 tests.
- Covers: default recall hides superseded memories; `include_superseded = true`
  surfaces them; linkage is bidirectional; invariants survive WAL compaction +
  cold reopen; deleting the superseding memory leaves `superseded_by` as a
  durable tombstone (stale memory stays hidden).
- No production code changes.
- Documentation in `docs/en/SUPERSESSION_LIFECYCLE.md` and
  `docs/pt-br/SUPERSESSION_LIFECYCLE.md`.
- 5 new tests. 150 tests total.

Previous: **Metadata Filter Regression Suite v0.1**:

- New test file `crates/hippocore/tests/metadata_filter.rs` with 7 tests.
- Covers: exact-match on memories, multi-key AND semantics, empty result (no
  error), collection-scoped filter, record filtering, cross-tenant
  non-interference, and `build_context` filter propagation.
- No production code changes.
- Documentation in `docs/en/METADATA_FILTER_REGRESSION.md` and
  `docs/pt-br/METADATA_FILTER_REGRESSION.md`.
- 7 new tests. 145 tests total.

Previous: **Multi-tenant Query Isolation Audit v0.1**:

- New dedicated test file `crates/hippocore/tests/tenant_isolation.rs`.
- 6 targeted isolation assertions covering all public read paths:
  `recall()` (Vector, Text, Hybrid), `build_context()` (basic and with
  `include_related = true`), and `list_graph_edges()`.
- Tests use two tenants with identical collection names and semantically
  identical content, ensuring the query layer hard-filters by `tenant_id`.
- No production code changes.
- Documentation in `docs/en/TENANT_ISOLATION_AUDIT.md` and
  `docs/pt-br/TENANT_ISOLATION_AUDIT.md`.
- 6 new tests. 138 tests total.

Previous: **Score Normalization v0.1**:

- `build_context` now normalises raw hybrid recall scores to `[0.0, 1.0]`
  before temporal decay and graph-aware ranking blends when any blend weight
  is non-zero. The highest-scoring candidate receives a normalised score of
  `1.0`; ordering is preserved.
- Normalisation is skipped entirely when both blend weights are `0.0` (default),
  so there is zero overhead and zero behavior change for unblended queries.
- Documentation in `docs/en/SCORE_NORMALIZATION.md` and
  `docs/pt-br/SCORE_NORMALIZATION.md`.
- 2 new integration tests. 132 tests total.

Previous: **Temporal Decay v0.1**:

- `Config` gains `temporal_weight: f32` (default `0.0`) and
  `temporal_decay_days: f32` (default `30.0`).
- `build_context` applies an optional recency bias after hybrid recall:
  `decay = exp(-age_days / decay_days)` blended into the effective score
  (`recall × (1−w) + decay × w`). Memory items use `created_at`; document
  chunks use the parent document's `updated_at`; records use `updated_at`.
  Items with unresolvable timestamps get a neutral decay of `0.5`.
- Temporal decay runs before graph-aware ranking; both features compose.
- `temporal_weight` outside `[0.0, 1.0]` or `temporal_decay_days ≤ 0` with
  non-zero weight returns `HippocoreError::Validation` before any I/O.
- Documentation in `docs/en/TEMPORAL_DECAY.md` and
  `docs/pt-br/TEMPORAL_DECAY.md`.
- 4 new integration tests. 130 tests total.

Previous: **Graph-Aware Ranking v0.1**:

- `Config` gains `graph_rank_weight: f32` (default `0.0` = disabled; no
  behavior change for existing databases).
- `build_context` applies an optional connectivity bonus after hybrid recall:
  for each candidate, counts how many of its direct graph neighbours also
  appear in the candidate set, normalises to `[0.0, 1.0]`, and blends into
  the effective score (`recall × (1−w) + connectivity × w`).
- Candidates are re-sorted by effective score after the pass.
- `w = 0.0` (default) produces identical ordering to plain recall.
- Passing `graph_rank_weight` outside `[0.0, 1.0]` returns a typed
  `HippocoreError::Validation` before any I/O.
- Documentation in `docs/en/GRAPH_AWARE_RANKING.md` and
  `docs/pt-br/GRAPH_AWARE_RANKING.md`.
- 3 new integration tests. 126 tests total.

Previous: **Audit Retention v0.1**:

- `Config` gains `audit_max_records: usize` and `audit_max_bytes: u64` (both
  default to `0` = unlimited; zero-change for existing databases).
- New public `AuditRetentionSummary` struct: `records_kept`, `records_removed`,
  `bytes_before`, `bytes_after`.
- New `Hippocore::compact_audit(max_records, max_bytes)` — reads `audit.log`,
  keeps the newest records satisfying both constraints, writes to `audit.tmp`,
  fsyncs, renames atomically. `Some(0)` disables that constraint for the call;
  `None` falls back to the configured value.
- `append_audit_record` now auto-enforces retention after every `build_context`
  append when `audit_max_records > 0` or `audit_max_bytes > 0` (best-effort).
- `DatabaseStats` gains `audit_log_bytes: u64` and `audit_records: usize`;
  `Display` prints them.
- CLI: new `compact-audit --db <path> [--max-records <n>] [--max-bytes <n>]
  [--json]` subcommand.
- Documentation in `docs/en/AUDIT_RETENTION.md` and
  `docs/pt-br/AUDIT_RETENTION.md`.
- 5 new core integration tests + 1 new CLI smoke test. 123 tests total.

Previous: **Graph-Aware Context v0.1**:

- `BuildContextRequest` gains `include_related: bool` and
  `related_limit: usize`; defaults keep existing behavior unchanged.
- `build_context` can optionally include direct graph neighbours of recalled
  items when they fit the token budget.
- Graph-expanded items are tenant-isolated, capped by `related_limit`, and
  deduplicated if already returned by recall.
- New `ContextItemSource` labels: `recalled`, `graph_expanded`,
  `not_included`.
- `ContextItem` and `AuditItem` now expose `inclusion_source`; audit records mark
  graph-expanded candidates that were considered but did not fit as
  `not_included`.
- CLI `build-context` gains `--include-related` and `--related-limit <n>`.
- Documentation added in `docs/en/GRAPH_AWARE_CONTEXT.md` and
  `docs/pt-br/GRAPH_AWARE_CONTEXT.md`.
- 5 new core integration tests plus CLI coverage through the graph edge flow.
  117 tests total.

Previous: **Phase 10 — Graph Memory v0.1**:

- New public `GraphEdge` model and `AddGraphEdgeRequest`.
- New durable APIs: `add_graph_edge`, `list_graph_edges`,
  `delete_graph_edge`, and `graph_neighbors`.
- New WAL/snapshot operations: `PutGraphEdge` and `DeleteGraphEdge`.
- Edge endpoints are validated by tenant, kind and id; missing or ambiguous
  endpoints are rejected with typed validation errors.
- Deleting memories, records, documents or imported files removes graph edges
  touching deleted endpoints deterministically.
- New CLI commands:
  - `add-edge --tenant <t> --from-kind <kind> --from-id <id> --to-kind <kind>
    --to-id <id> --relation <name> [--json]`
  - `list-edges --tenant <t> [--from-id <id>] [--json]`
  - `delete-edge --tenant <t> --id <edge_id>`
- `ContextItem` and `AuditItem` now expose `related_item_ids` for direct graph
  neighbours at context-build time.
- Documentation added in `docs/en/GRAPH_MEMORY.md` and
  `docs/pt-br/GRAPH_MEMORY.md`.
- 6 new core integration tests and 1 new CLI smoke test. 112 tests total.

Previous: **Phase 9 — RAG Audit Engine**:

- New public audit types: `AuditRecord` and `AuditItem`.
- Every `Hippocore::build_context(req)` appends one JSON-lines audit record to
  `<data_dir>/audit.log`.
- Audit records capture timestamp, tenant, query, mode, optional collection,
  requested token budget, final token count, dropped count, retrieved item ids,
  context compilation latency, item kinds, collections, vector/text/final
  scores, confidence, per-item token counts and whether each item entered the
  final context.
- New `Hippocore::query_audit(from_ms, to_ms, tenant_id)` API replays records
  for one tenant in an inclusive epoch-ms window.
- New CLI command:
  `hippocore audit --db <path> --tenant <t> --from <ms> --to <ms> [--json]`.
- `audit --json` emits parseable `Vec<AuditRecord>` JSON.
- Audit history is append-only and separate from the mutation WAL; it survives
  reopen without changing state recovery.
- Documentation added in `docs/en/RAG_AUDIT_ENGINE.md` and
  `docs/pt-br/RAG_AUDIT_ENGINE.md`.
- 3 new core integration tests and 1 new CLI smoke test. The existing
  confidence/rating tests cover the `rate_memory` acceptance criteria. 105 tests
  total.

Previous: **Phase 6 — Interactive TUI (`hippocore studio`)**:

- New `hippocore studio [--db <path>]` subcommand launching a full-terminal ratatui UI.
- Four navigable tabs: **Tenants** (tenants + their collections), **Memories** (live search across all tenants), **Documents** (all stored documents), **Stats** (database statistics).
- Keybindings: `1`–`4` or `Tab`/`Shift-Tab` to switch tabs; `↑↓` / `jk` to scroll; `/` to enter search in the Memories tab; `Enter` to execute search; `Esc` to cancel; `r` to refresh data; `q` or `Ctrl-C` to quit.
- Memory search calls `recall()` across all tenants and displays `[score] tenant/collection — snippet…`.
- Refresh re-queries the database for all tenant, document, and stats panels.
- `ratatui 0.29` and `crossterm 0.28` added as workspace + CLI crate dependencies.
- TUI implementation in `crates/hippocore-cli/src/tui.rs`; binary entry point at `crates/hippocore-cli/src/main.rs` intercepts `studio` before delegating all other subcommands to `hippocore::cli`.
- 1 new CLI smoke test (`cli_studio_exits_without_panic`) — verifies no panic when invoked in a non-TTY environment. 105 tests total.
- All quality gates pass: `cargo fmt`, `cargo test --workspace`, `cargo clippy -D warnings`.

Previous: **Phase 4 — Pluggable `VectorIndex` trait + HNSW**, **Phase 7 — Confidence-aware resolution**, and **Phase 2 — Group-commit / batch writes**:

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
- Durable graph edges between memories, records and document chunks, with
  direct-neighbour lookup and CLI management.
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

- Retrieval defaults to exact brute force; optional HNSW exists.
- Graph edges can optionally expand context by direct neighbours; graph-aware
  ranking and multi-hop traversal are not implemented yet.
- The in-memory index is rebuilt fully on open (incremental during runtime).
- Per-chunk external embeddings are a library API; the CLI `put-document` still
  auto-embeds (CLI ergonomics for many chunk vectors are deferred).
- Record field-level indexes, schema management, full blob storage, PDF/OCR and
  a web admin dashboard are product-direction documents only; they are not
  implemented yet.

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

**All green.** `cargo test --workspace` passes 123 tests:
- 22 unit (embedder/chunker, cosine/index/BM25, query normalization + RRF, CRC32 + WAL
  line decode, 4 new HNSW unit tests),
- 83 library integration (store/recall, chunking, retrieval quality layer,
  structured records, imported files, user-supplied document embeddings +
  validation, modes, sorting, metadata & type/user filters, tenant isolation,
  restart, compaction, auto-compaction bounding the WAL, checksum-mismatch
  recovery, delete/forget + restart + compaction, user embeddings, empty DB,
  error paths, dimension-mismatch, torn-WAL recovery, temporal filtering,
  supersedure/contradictions, context compiler, batch writes, confidence-aware
  resolution, HNSW backend store+recall+restart+brute-force comparison, RAG audit
  write/replay/reopen, Graph Memory add/list/validation/isolation/restart/delete
  cleanup/context provenance, Graph-Aware Context expansion/limit/budget/no
  duplication/default behavior, Audit Retention default/max-records/max-bytes/
  query-after-compaction/auto-retention-from-config),
- 1 retrieval-quality fixture test (hit@1/hit@5/MRR thresholds),
- 16 CLI subprocess smoke tests (incl. `compact`, `forget`, `put-record`,
  `import-file`, admin `list-*` and `show-*` commands with `--json`,
  `eval-quality` pass + fail paths, `audit --json`, graph edge flow,
  `compact-audit --json`, and `studio` non-TTY smoke test),
- 1 doctest (lib.rs quickstart),
- 1 bench_regression example.

`cargo clippy … -D warnings` passes with zero warnings; `cargo fmt --all --check`
is clean.

`cargo bench -p hippocore` completed. Final run:
- `remember`: 7.9212-8.0148 ms.
- `recall_hybrid`: 556.14-563.91 us.
- `search_vector`: 84.930-85.380 us.

## Current architectural decisions

See DECISIONS.md. Highlights: JSON-lines WAL with per-line CRC32 + atomic
snapshot; threshold-based auto-compaction; unified `IndexEntry` over chunks and
memories; deterministic feature-hashing embedder; RRF hybrid fusion; tenant
isolation enforced in the query layer; append-only RAG audit log separate from
the mutation WAL.

## Next recommended feature

**Phase 11 — HTTP Server** (`crates/hippocore-server`): axum-based REST API
exposing the full core library over HTTP, with API-key authentication, JSON
request/response bodies, and a `hippocore serve` CLI subcommand.
See NEXT_FEATURE.md.
