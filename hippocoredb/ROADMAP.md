# Hippocore DB Roadmap

Hippocore DB aims to become a **context database for AI agents**. The MVP
(Phase 1) is intentionally small; later phases build toward reliable, auditable,
local-first AI memory. Phases are a direction of travel, not a dated commitment.

## Phase 1 — Local-first memory database ✅ (MVP)

- Multi-tenant model: tenants, collections, documents (auto-chunked), memories.
- Durable JSON-lines WAL + atomic snapshot; startup recovery; compaction.
- Exact vector (cosine), text (TF-IDF inverted index), and **hybrid** recall.
- Filters by tenant, collection, user, memory type, item kind, metadata.
- Deterministic offline embedder; user-supplied embeddings accepted.
- Public API (`open`/`create_*`/`store_document`/`remember`/`recall`/`search`/
  `stats`/`compact`/`close`) and CLI (`init`/`put-document`/`remember`/`recall`/
  `stats`/`inspect`). Tests, example, benchmarks.

## Phase 2 — Robust WAL: checksums, automatic compaction, batch writes

- Per-WAL-line CRC32 checksums to detect mid-line corruption. ✅
- Threshold-based **automatic compaction** to bound disk + recovery time. ✅
- Delete / forget operations (tombstones). ✅
- User-provided embeddings for documents. ✅
- Group-commit / batched writes for throughput.

## Phase 3 — HNSW or pluggable ANN vector index

- Approximate nearest-neighbour index for sub-linear search.
- Pluggable index trait so brute force and ANN coexist.

## Phase 4 — Hybrid search with BM25 + vectors

- Lexical BM25 scoring. ⏳ next — see `docs/NEXT_FEATURE.md`.
- Fusion of lexical and vector scores (e.g. reciprocal rank fusion).

## Phase 5 — Temporal Truth Layer

- `valid_from`, `valid_until`, `supersedes`, `contradicts`.
- Source and confidence-aware resolution of conflicting facts.
- "What was true at time T?" queries.

## Phase 6 — Context Compiler

- `build_context(query, user, max_tokens)`.
- Token-budget-aware assembly of the best context for an LLM.

## Phase 7 — RAG Audit Engine

- Trace question → retrieved documents → context → answer.
- Record tokens, latency and feedback for auditability.

## Phase 8 — Graph Memory

- Entities and relations.
- GraphRAG-style traversal over the knowledge graph.

## Phase 9 — Server mode

- HTTP / gRPC server.
- Authentication, multitenancy and permissions.

## Phase 10 — Multimodal storage

- Text, PDF, image, audio, video and code.
