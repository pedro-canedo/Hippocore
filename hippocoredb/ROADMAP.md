# Hippocore DB Roadmap

Hippocore DB aims to become a **context database for AI agents**. It is also a
database in the practical sense: users should be able to insert, manage,
inspect, delete and import information, and the supported data types should be
projected into native context for SDKs, agents and RAG systems.

The MVP is intentionally small; later phases build toward reliable, auditable,
local-first AI memory and context management. Phases are a direction of travel,
not a dated commitment.

## Phase 1 — Local-first memory database ✅ (MVP)

- Multi-tenant model: tenants, collections, documents (auto-chunked), memories.
- Durable JSON-lines WAL + atomic snapshot; startup recovery; compaction.
- Exact vector (cosine), text (BM25 inverted index), and **hybrid** recall.
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
- Group-commit / batched writes for throughput. ✅

## Phase 3 — Data model expansion and context projection

- Define `File`, `Table`, `Record`, and `ContextItem` concepts.
- JSON-first structured record storage. ✅
- Text-like file ingestion (`.txt`, `.md`, `.json`, `.csv`). ✅
- Native context projection for documents, memories, records and files. ✅
- CLI `put-record` / `delete-record`. ✅
- CLI `import-file` / `delete-file`. ✅
- CLI inspection/list/import/export workflows for broader database management. ✅

## Phase 4 — HNSW or pluggable ANN vector index

- Approximate nearest-neighbour index for sub-linear search. ✅
- Pluggable index trait so brute force and ANN coexist. ✅

## Phase 5 — Hybrid search refinement

- Lexical BM25 scoring. ✅
- Retrieval quality fixtures. ✅
- Fusion of lexical and vector scores (e.g. reciprocal rank fusion). ✅
- Benchmark regression guard. ✅

## Phase 6 — Admin interface and local Studio

- Expand CLI administration commands. ✅
- Retrieval quality CLI (`eval-quality`). ✅
- Local management UI for browsing, inserting, editing and deleting context
  data.
- Retrieval debugging, stats, compaction and quality-evaluation views.
- Keep this local-first; do not require server/cloud mode.

## Phase 7 — Temporal Truth Layer

- `valid_from`, `valid_until`, `supersedes`, `contradicts`.
- `valid_from`/`valid_until` on memories and documents with `as_of` queries. ✅
- `supersedes`/`contradicts` relations with durable supersedure marking. ✅
- Superseded memories excluded from default recall; `include_superseded` override. ✅
- Contradiction ids surfaced in `RecallResult.contradictions`. ✅
- Source and confidence-aware resolution of conflicting facts. ✅
- "What was true at time T?" queries. ✅ (basic `as_of` filter)

## Phase 8 — Context Compiler

- `build_context(query, user, max_tokens)`. ✅
- Token-budget-aware assembly of the best context for an LLM. ✅
- `ContextBlock` with assembled text, token count, item provenance, and drop count. ✅
- CLI `build-context` subcommand with `--json` output. ✅

## Phase 9 — RAG Audit Engine

- Trace question → retrieved documents → context → answer.
- Record tokens, latency and feedback for auditability.

## Phase 10 — Graph Memory

- Entities and relations.
- GraphRAG-style traversal over the knowledge graph.

## Phase 11 — Server mode

- HTTP / gRPC server.
- Authentication, multitenancy and permissions.

## Phase 12 — Multimodal storage

- Text, PDF, image, audio, video and code.
