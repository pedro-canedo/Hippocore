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
