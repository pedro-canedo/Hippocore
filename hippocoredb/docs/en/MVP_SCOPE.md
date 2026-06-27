# Hippocore DB — MVP Scope

## In scope (implemented)

- Embedded Rust library, `hippocore` CLI, optional HTTP server, and embedded
  local Control Plane.
- Typed tenants, collections, documents, chunks, memories, records, files,
  graph edges, audit records, sources, metadata, and recall/context results.
- Local persistence with no external database: checksummed append-only WAL,
  startup replay, atomic snapshots, compaction, and typed recovery errors.
- Exact cosine and optional HNSW vector indexes, BM25 text search, hybrid
  recall, tenant-scoped filters, deterministic embeddings, and caller-provided
  embeddings.
- Context assembly, temporal validity, supersession, contradiction advisory,
  confidence weighting, graph expansion, and retrieval audit records.
- JSON-first records, chunked documents, memories, and PDF/CSV/JSON/text file
  ingestion.
- A deliberately small read-only SQL-like subset for structured records.
- Admin flows for tenants, collections, tables, data ingestion, SQL, recall,
  context, providers, service keys, prompts, chat, and observability.
- Deterministic tests, examples, documentation, and baseline benchmarks.

## Explicitly out of scope (this MVP)

- Distributed clustering, Raft/consensus, or multi-node replication.
- Production-grade authorization and hosted identity management.
- Mandatory cloud embedding services or GPU-only indexing.
- A full SQL engine, joins, arbitrary expressions, or PostgreSQL wire protocol.
- An external database dependency.
- `unsafe` Rust (forbidden).

## Deliberate simplifications

- The live state fits in memory; indexes are rebuilt on open.
- Exact retrieval remains the offline default; HNSW is an optional backend.
- Snapshot storage remains JSON-based.
- Records are JSON-first and do not yet have rich schemas or per-field indexes.
- The Control Plane is migrating to a React + Vite + TypeScript app served at
  `/console`; the legacy zero-build vanilla-JS app stays at `/admin` until the
  port completes. The TypeScript build is committed and embedded with
  `include_str!`, so `cargo build` and the Docker image still need no Node
  toolchain. See ADR-016.
