# Architecture Decision Records

Meaningful decisions for the Hippocore DB MVP. Newest last.

---

## ADR-001: JSON-lines WAL + atomic JSON snapshot

- **Decision**: Persist mutations as one JSON `Operation` per line in `wal.log`;
  periodically fold them into a single `snapshot.json` written atomically
  (temp file → `fsync` → rename). Recovery loads the snapshot then replays the
  WAL; a torn/partial trailing line is skipped.
- **Context**: The MVP needs durable, restart-safe local storage with no
  external database, and must recover safely from a crash mid-write.
- **Alternatives considered**:
  - The v0.1 hand-rolled binary log with CRC framing (document-only).
  - `bincode`/`sled`/embedded KV stores.
- **Reason**: JSON lines are self-delimiting, trivially appendable, human
  inspectable, and dependency-light; a bad trailing line is detected by a failed
  parse. Atomic rename gives crash-safe snapshots. Simplest correct design.
- **Tradeoffs**: Larger on disk and slower to parse than binary; no per-line
  checksum yet (only truncation is detected, not mid-line bit-rot — addressed in
  NEXT_FEATURE.md).

## ADR-002: Unified `IndexEntry` over chunks and memories

- **Decision**: Project both document chunks and memories into one internal
  `IndexEntry` (with a `kind` tag) that the vector and text indexes operate on.
- **Context**: Documents and memories must be searchable through the same recall
  path, with the same filters and fusion.
- **Alternatives**: Separate indexes per type and merge results.
- **Reason**: One scan and one inverted index serve both; filters and scoring are
  written once. The persisted `Document`/`Chunk`/`Memory` models remain distinct.
- **Tradeoffs**: A thin projection layer; entries duplicate some fields in memory.

## ADR-003: Deterministic feature-hashing embedder

- **Decision**: Ship a built-in embedder using signed FNV-1a feature hashing into
  a fixed-dimension L2-normalized vector. Callers may supply their own embeddings
  for memories and queries.
- **Context**: Tests/examples must be deterministic and offline; the product must
  not depend on a network or cloud embedder.
- **Alternatives**: Bundle a real model (heavy, non-deterministic across
  versions); require users to always provide embeddings (poor DX).
- **Reason**: Reproducible, zero-dependency, and good enough for the concept:
  identical text → identical vector; shared tokens → higher cosine.
- **Tradeoffs**: Not semantically rich (no real language understanding); hash
  collisions add noise. Production users should plug in real embeddings.

## ADR-004: Hybrid fusion via min-max normalization

- **Decision**: Score candidates with cosine (vector) and BM25 (text), min-max
  normalize each signal across the candidate set into `[0,1]`, then fuse with
  `score = alpha*vector + (1-alpha)*text` (default `alpha = 0.5`).
- **Context**: Vector and text scores live on different scales and cannot be
  added directly.
- **Alternatives**: Reciprocal Rank Fusion (RRF); fixed score thresholds.
- **Reason**: Min-max is simple, deterministic, and keeps both signals
  comparable; `alpha` is configurable.
- **Tradeoffs**: The best candidate in a set always normalizes to 1.0 and the
  worst to 0.0, so absolute scores are relative to the result set, not global.
  RRF is a reasonable future upgrade.

## ADR-005: Tenant isolation enforced in the query layer

- **Decision**: Every recall/search requires a `tenant_id`; the `Filter` checks
  it first and no entry from another tenant can be returned.
- **Context**: Multi-tenant AI systems must never leak one tenant's memory to
  another.
- **Alternatives**: Separate physical stores per tenant.
- **Reason**: A single store with a mandatory tenant filter is simple and keeps
  one WAL/snapshot; isolation is a single, testable invariant.
- **Tradeoffs**: Isolation is logical, not physical; a query-layer bug could
  cross tenants — hence it is covered by a dedicated test.

## ADR-007: Per-line CRC32 + safe-stop recovery; threshold auto-compaction

- **Decision**: Frame each WAL line as `<crc32-hex>\t<json>` (format v2). On
  recovery, verify the CRC of every line; on a mismatch (or unparseable
  payload, or truncated tail) stop replay immediately, keeping only what was
  read before. Add a `Config` policy that auto-compacts once the WAL exceeds an
  op-count or byte threshold. CRC32 is implemented inline (no new dependency).
- **Context**: The JSON-lines WAL (ADR-001) detected truncation but not mid-line
  bit-rot, and grew unbounded without a manual `compact()` call.
- **Alternatives considered**: re-add the `crc32fast` crate (extra dependency);
  binary length-framed records with a trailing checksum (less inspectable);
  background/timer-based compaction (more moving parts).
- **Reason**: A checksum per line detects corruption cheaply and keeps the log
  human-inspectable; stopping on the first bad line is the conservative,
  data-integrity-preserving choice for an ordered log. Inline CRC32 keeps the
  dependency set lean. Threshold auto-compaction in `commit` is simple and makes
  the database safe to run continuously. Legacy lines stay readable for a smooth
  upgrade.
- **Tradeoffs**: A corrupted *middle* line discards all later (possibly valid)
  records — acceptable because ordering after a gap is untrustworthy.
  Auto-compacting inside `commit` can cause an occasional latency spike on the
  triggering write; thresholds are tunable (and `0` disables).

## ADR-006: Strict library API, ergonomic CLI

- **Decision**: The library requires tenant/collection to exist before
  `store_document`/`remember` (typed `UnknownTenant`/`UnknownCollection`). The
  CLI auto-creates them for convenience.
- **Context**: We want clear typed errors and testable invariants, but a smooth
  quickstart.
- **Reason**: Library correctness + CLI ergonomics without coupling them.
- **Tradeoffs**: Slightly different behavior between the two surfaces (documented
  in README and CLI help).

## ADR-008: RAG audit log is append-only and separate from the mutation WAL

- **Decision**: Persist query-time audit records in `<data_dir>/audit.log` as
  one JSON object per line, separate from `wal.log` and `snapshot.json`.
- **Context**: Phase 9 needs to trace `build_context` calls without treating a
  query as a durable mutation of the materialized database state.
- **Alternatives considered**:
  - Add audit records to the mutation WAL as a new `Operation`.
  - Store audit records inside `State` and snapshots.
- **Reason**: Audit events are operational history, not live context data. A
  dedicated append-only JSON-lines file keeps query replay simple, inspectable
  and local-first while avoiding index rebuild or snapshot bloat.
- **Tradeoffs**: `audit.log` is not compacted with the state snapshot. That is
  acceptable for Phase 9; retention/rotation can be added later without changing
  recovery semantics.

## ADR-009: Graph Memory v0.1 uses tenant-scoped item ids, not graph queries

- **Decision**: Store `GraphEdge` endpoints as `(tenant_id, ItemKind, id)` and
  reject endpoint ids that are missing or ambiguous within a tenant.
- **Context**: Phase 10 needs durable relationships between memories, document
  chunks and records, but the MVP explicitly avoids a complex query language and
  full GraphRAG traversal.
- **Alternatives considered**:
  - Require collection/table-qualified endpoint references in every CLI command.
  - Introduce a graph query/reference language.
  - Store graph edges only as loose strings without validation.
- **Reason**: Tenant-scoped kind/id endpoints keep the API and CLI small while
  still enforcing correctness. Rejecting ambiguity is safer than guessing which
  item an edge should target.
- **Tradeoffs**: Users with duplicate ids across collections/tables must use
  unique ids before creating graph edges. A richer endpoint reference can be
  added later without changing the basic `GraphEdge` persistence model.

## ADR-010: Graph-aware context expands direct neighbours only

- **Decision**: `build_context` can optionally include direct `GraphEdge`
  neighbours of recalled items through `BuildContextRequest::include_related`
  and `related_limit`. Expanded items are marked as `graph_expanded` in context
  and audit output; recalled items stay marked as `recalled`, and considered
  items that do not fit the token budget are marked as `not_included` in audit.
- **Context**: Graph Memory v0.1 records durable relationships and exposes
  direct neighbour ids as provenance. The next useful increment is to let callers
  opt into those relationships during context assembly without turning the MVP
  into a graph query engine.
- **Alternatives considered**:
  - Automatically expand graph neighbours for every `build_context` call.
  - Boost recall scores based on graph edges.
  - Traverse multiple hops or implement GraphRAG-style planning.
- **Reason**: Opt-in direct expansion keeps default recall behavior unchanged,
  preserves tenant isolation, keeps the token budget as the final arbiter, and
  gives RAG callers useful related context without adding a new ranking system.
- **Tradeoffs**: Expanded items currently use neutral scores because they were
  not selected by recall. Direct expansion can improve completeness, but it does
  not prove relevance beyond the stored edge; graph-aware ranking and multi-hop
  traversal remain future work.

## ADR-011: Control Plane stays zero-build while backend flows stabilize

- **Decision**: The `/admin` Control Plane remains embedded static
  HTML/CSS/JavaScript served by `hippocore-server`, but is organized as an app
  shell with domain pages and reusable UI render helpers.
- **Context**: The admin UI needs to become a clear developer-facing product
  surface, while SQL/import/table-catalog backend flows are still evolving.
- **Alternatives considered**:
  - Migrate immediately to Vite/React/TypeScript.
  - Keep a single-page debug console until all backend flows exist.
- **Reason**: A zero-build shell keeps Docker/local deployment simple and avoids
  adding frontend build complexity before backend contracts settle. Domain pages
  still make the product model clear: logical data, physical/internal objects,
  RAG/context workflows, and API/admin operations are separated.
- **Tradeoffs**: The hand-written UI code is less scalable than a framework app.
  If Control Plane state management grows beyond this foundation, a bundled
  frontend build can replace the static assets later without changing the core
  database model.

## ADR-012: HTTP retrieval responses preserve user-facing evidence

- **Decision**: Server response adapters preserve retrieval evidence required
  to explain a result, beginning with `matched_terms`, instead of reducing core
  results to identifiers, text, and a final score only.
- **Context**: The core already computes deterministic retrieval diagnostics,
  but the recall HTTP handler discarded `matched_terms`, preventing the Control
  Plane and API clients from explaining lexical matches.
- **Reason**: Hippocore's product promise includes grounded, auditable context.
  Exposing already-computed evidence is additive, costs no extra retrieval work,
  and keeps clients from reimplementing query analysis inconsistently.
- **Trade-offs**: Response objects become slightly larger. New evidence fields
  remain additive so existing clients continue to deserialize known fields.

## ADR-013: Control Plane localization uses an embedded catalog

- **Decision**: Keep English and Portuguese copy in the embedded `I18N` catalog
  and make runtime renderers reference catalog keys. Preserve active form values
  with a transient DOM snapshot when switching language; do not persist drafts.
- **Context**: The zero-build Control Plane had partial translations and mixed
  hardcoded English with catalog-based text. A framework migration solely for
  localization would add build complexity without improving backend contracts.
- **Reason**: A matched internal catalog keeps deployment unchanged, makes
  missing translations auditable, and lets every active view rerender
  immediately. A transient snapshot preserves UX without writing passwords or
  unfinished input to local storage.
- **Trade-offs**: Catalog maintenance is manual and static tests cannot replace
  full browser automation. A future bundled frontend may adopt an i18n library.

## ADR-014: Data Explorer composes existing scoped list endpoints

- **Decision**: Load Records, Memories, Documents, and Files concurrently from
  their existing tenant/collection-scoped admin endpoints and derive active
  collection counts in the Control Plane.
- **Context**: The Explorer needs type counts and instant switching, but the
  current data volume is local-first and every required scoped list already
  exists. A new aggregation contract would duplicate behavior before scale is
  measured.
- **Reason**: Reusing stable endpoints preserves tenant isolation, keeps the
  server unchanged, and provides a complete workspace with a small UI-only
  increment.
- **Trade-offs**: Opening or changing a collection makes four parallel local
  requests. If measured object volume makes this expensive, a summarized admin
  endpoint or pagination can replace client-derived counts later.
