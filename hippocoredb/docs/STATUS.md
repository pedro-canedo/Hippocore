# Hippocore DB — Status

_Last updated: 2026-06-26._

## What was implemented last

**RAG Quality Layer v0.1**:

- Query normalization maps common PostgreSQL typos/aliases (`postgress`,
  `postgres`) to `postgresql` and connection variants (`conexão`, `conexao`,
  `connection`) to a shared lexical signal.
- The query layer detects simple technology/entity tags (`oracle`,
  `postgresql`, `python`, `listener`, `vacuum`, `connection`) on queries and
  indexed entries.
- Hybrid/text/vector final scores receive small explainable boosts/penalties
  when a query clearly targets Oracle or PostgreSQL, preventing Oracle listener
  memories from outranking Python/PostgreSQL connection memories unless Oracle
  is explicitly mentioned.
- The TypeScript Ollama RAG example now seeds PostgreSQL service, port, `pg_ctl`,
  Python `psycopg`/`psycopg2`, and "not Oracle listener/lsnrctl" memories; it
  also prints retrieval debug details and uses a stricter grounded answer prompt.

Previous increment: user-provided document embeddings via
`StoreDocumentRequest.chunks: Option<Vec<ChunkInput>>`.

Earlier increments (still current): multi-tenant model; JSON-lines WAL with
per-line CRC32 + atomic snapshot; automatic + manual compaction; delete/forget
(`forget`, `delete_document`); deterministic embedder + chunker;
vector/text/hybrid recall; CLI (`init`/`put-document`/`remember`/`recall`/
`forget`/`delete-document`/`stats`/`inspect`/`compact`, with
`recall --embedding/--json`); and the `examples/ts-ollama-rag/` demo (local
Ollama embeddings + generation, idempotent ingestion).

## What is working

- Store/recall for both documents (chunked) and memories.
- Vector, BM25 text, and hybrid modes; every result carries `score`,
  `vector_score`, `text_score`, and a `reason` that includes entity tags and
  boost/penalty explanations when applied.
- Tenant isolation (verified by test).
- Durable restart: snapshot + WAL replay; torn trailing line AND checksum
  mismatch both recover safely (no panic, no silently-loaded corruption).
- Manual `compact()` and automatic threshold-based compaction; both survive
  reopen.
- User-supplied embeddings for memories and queries; built-in embedder otherwise.
- Typed errors for unknown tenant/collection, validation, empty/mismatched
  embeddings, and corruption.

## What is partial

- Retrieval is exact brute force (O(n) per tenant). Correct, not yet scalable.
- The in-memory index is rebuilt fully on open (incremental during runtime).
- Per-chunk external embeddings are a library API; the CLI `put-document` still
  auto-embeds (CLI ergonomics for many chunk vectors are deferred).

## What is broken or missing

- No ANN/HNSW, no server, no auth — by design (see docs/MVP_SCOPE.md).
- Retrieval quality is still heuristic. It is not a learned ranker and it does
  not yet have a reusable evaluation harness beyond deterministic tests and the
  manual RAG example checks.

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

**All green.** `cargo test --workspace` passes 53 tests:
- 16 unit (embedder/chunker, cosine/index/BM25, query normalization, CRC32 + WAL
  line decode),
- 31 library integration (store/recall, chunking, retrieval quality layer,
  user-supplied document
  embeddings + validation, modes, sorting, metadata & type/user filters, tenant
  isolation, restart, compaction, auto-compaction bounding the WAL,
  checksum-mismatch recovery, delete/forget + restart + compaction, user
  embeddings, empty DB, error paths, dimension-mismatch, torn-WAL recovery),
- 5 CLI subprocess smoke tests (incl. `compact`, `forget`),
- 1 doctest.

`cargo clippy … -D warnings` passes with zero warnings; `cargo fmt --all --check`
is clean.

`cargo bench -p hippocore` completed. In the final run, `remember` showed no
statistically significant change, while `recall_hybrid` and `search_vector`
improved against the local Criterion baseline after avoiding tag work in pure
vector search and reusing indexed tokens for entity detection.

## Current architectural decisions

See docs/DECISIONS.md. Highlights: JSON-lines WAL with per-line CRC32 + atomic
snapshot; threshold-based auto-compaction; unified `IndexEntry` over chunks and
memories; deterministic feature-hashing embedder; min-max hybrid fusion; tenant
isolation enforced in the query layer.

## Next recommended feature

**Retrieval evaluation harness v0.1** — see docs/NEXT_FEATURE.md.
