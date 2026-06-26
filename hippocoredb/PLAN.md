# Hippocore DB — Implementation Plan

## Goal

Build a working, local-first **AI-native memory database** MVP that proves the
core concept: Hippocore is not just a vector store, it is a **memory database**
for AI applications. It must persist data safely, survive restarts, isolate data
by tenant, and recall context via **vector**, **text**, and **hybrid** search.

## Starting point

v0.1 already provided a document store with an append-only binary log, in-memory
index, cosine search and a CLI. This plan **expands** that into the full memory
database. We preserve the cosine-similarity logic and the typed-error style, and
replace the document-only binary log with a tenant-aware **WAL + snapshot**
engine, because the data model now spans tenants, collections, documents,
chunks, and memories.

## Module layout (crate `hippocore`)

```
src/
  lib.rs       Hippocore engine (open/create_collection/store_document/
               remember/recall/search/stats) + re-exports
  config.rs    Config (data_dir, embedding_dim, chunk size, hybrid alpha, sync)
  errors.rs    HippocoreError + Result
  model.rs     Tenant, Collection, Document, Chunk, Memory, Embedding,
               Metadata, Source, MemoryType, RecallResult, ids, validation
  storage.rs   Operation log (WAL, JSON-lines), Snapshot (atomic), State,
               recovery, compaction
  index.rs     In-memory vector store + inverted text index; cosine + TF-IDF
               scoring; filtered candidate retrieval
  memory.rs    Deterministic hashing embedder + text chunker
  query.rs     Request types, SearchMode, hybrid score fusion, RecallResult build
  cli.rs       clap parser + command handlers (init/put-document/remember/
               recall/stats/inspect)
crates/hippocore-cli/src/main.rs  thin wrapper calling hippocore::cli::run()
```

## Key MVP decisions (see docs/DECISIONS.md)

1. **WAL = JSON lines**, one `Operation` per line; trailing partial line is
   skipped on recovery. Snapshot = single JSON `State` written atomically
   (temp + fsync + rename). Simplest correct durable design; no external DB.
2. **Unified indexed unit**: documents are split into chunks; both chunks and
   memories become `IndexEntry` records the index scores over. Keeps retrieval
   uniform while preserving distinct `Document`/`Chunk`/`Memory` models.
3. **Deterministic embedder**: signed feature-hashing of tokens into a fixed-dim
   L2-normalized vector. Same text → same vector; shared tokens → higher cosine.
   No network, fully reproducible. Users may also supply their own embeddings.
4. **Hybrid score** = `alpha * vector_norm + (1 - alpha) * text_norm`, scores
   min-max normalized across candidates; default `alpha = 0.5`.
5. **Tenant isolation** enforced in the query layer: every recall/search filters
   by `tenant_id` first; no entry from another tenant can ever be returned.

## Steps

1. PLAN.md (this file) + restructure modules. ✅ in progress
2. `errors.rs`, `model.rs`, `config.rs`.
3. `storage.rs` (WAL + snapshot + recovery + compaction).
4. `memory.rs` (embedder + chunker), `index.rs`, `query.rs`.
5. `lib.rs` engine + `cli.rs`; wire `hippocore-cli`.
6. Tests (scoring, chunking, validation, filters, store/recall, restart, tenant
   isolation, errors); update example + bench.
7. Docs: README, ARCHITECTURE, MVP_SCOPE, STATUS, NEXT_FEATURE, DECISIONS,
   CHANGELOG. Run fmt/clippy/test; fix until green.

## Out of scope (this MVP)

Clustering, Raft/consensus, production auth, cloud embedders, GPU/ANN/HNSW, web
dashboard, replication, SQL/query language, external DB, `unsafe`.
