# Hippocore DB — MVP Scope

## In scope (implemented)

- Embedded Rust library + `hippocore` CLI.
- Strongly-typed model: Tenant, Collection, Document, Chunk, Memory, Embedding,
  Metadata, Source, RecallResult, MemoryType, ItemKind.
- Local persistence with **no external database**:
  - data directory initialization,
  - append-only JSON-lines WAL,
  - startup recovery (snapshot + WAL replay),
  - atomic snapshot writes (temp + fsync + rename),
  - compaction (snapshot + WAL truncation),
  - typed errors for invalid/corrupted data; safe recovery of torn trailing WAL.
- Retrieval:
  - exact vector search (cosine similarity),
  - text search via an inverted index (TF-IDF),
  - hybrid recall fusing vector + text,
  - filters: tenant, collection, user_id, memory type, item kind, metadata,
  - deterministic built-in embedder; user-provided embeddings accepted.
- Public API: `open`, `create_tenant`, `create_collection`, `store_document`,
  `remember`, `recall`, `search`, `stats`, `compact`, `close` (all return
  typed `Result`).
- CLI: `init`, `put-document`, `remember`, `recall`, `stats`, `inspect`.
- Deterministic tests, an example, and baseline benchmarks.
- Documentation set (this folder + README/ROADMAP/CHANGELOG).

## Explicitly out of scope (this MVP)

- Distributed clustering, Raft/consensus, multi-node replication.
- Production authentication / authorization.
- Cloud embedding providers; GPU indexing.
- HNSW / ANN approximate indexes.
- Web dashboard.
- A SQL or complex query language.
- External database dependency.
- `unsafe` Rust (forbidden).

## Deliberate simplifications

- The full state lives in memory; data must fit in RAM.
- Retrieval is exact/brute-force (O(n) over a tenant's entries).
- The index is updated incrementally on write and fully rebuilt on open.
- Documents are embedded with the built-in embedder per chunk; user-provided
  embeddings are supported for **memories** and **queries**.
- Hybrid fusion uses min-max normalization across the candidate set.
