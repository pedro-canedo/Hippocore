# Hippocore DB — Architecture

Hippocore is an embedded, single-process memory database. It is organized into
small modules with clear boundaries.

## Module map (crate `hippocore`)

| Module        | Responsibility                                                       |
|---------------|----------------------------------------------------------------------|
| `config`      | `Config` (data dir, embedding dim, chunk size, hybrid alpha, sync).  |
| `errors`      | `HippocoreError` + `Result`; all normal failures are typed.          |
| `model`       | Typed entities: `Tenant`, `Collection`, `Document`, `Chunk`, `Memory`, `Source`, `MemoryType`, `ItemKind`, `RecallResult`, plus validation. |
| `storage`     | `Operation`, `State`, the JSON-lines WAL, atomic snapshot, recovery, compaction. |
| `memory`      | Deterministic `Embedder` (signed feature hashing) + text `chunk_text` + `tokenize`. |
| `index`       | In-memory `Index`: exact vector store + inverted text index; cosine + BM25 scoring. |
| `query`       | `Filter`, `SearchMode`, `QueryRequest`, query normalization, entity tags, score normalization + hybrid fusion → `RecallResult`. |
| `cli`         | clap parser + command handlers (`init`/`put-document`/`remember`/`recall`/`stats`/`inspect`). |
| `lib.rs`      | The `Hippocore` engine tying everything together; public API + request types. |

## Data flow

### Write (`store_document` / `remember`)

```
caller → Hippocore.store_document/remember
       → validate model
       → derive chunks + embeddings (memory::Embedder)        [documents]
       → storage.append(Operation)   (WAL: write + flush + fsync)
       → state.apply(Operation)      (in-memory materialized State)
       → index.insert(IndexEntry…)   (vector + inverted index updated)
```

The WAL append happens **before** in-memory mutation, so a crash can only lose
an un-acknowledged write, never corrupt committed state.

### Read (`recall` / `search`)

```
caller → Hippocore.run_query
       → build Filter (tenant_id mandatory)
       → query::execute(index, embedder, request)
           → normalize query text (for typo/entity handling)
           → filter entries (tenant/collection/user/type/kind/metadata)
           → score: cosine (vector) and/or BM25 (text)
           → min-max normalize each signal across candidates → [0,1]
           → fuse: hybrid = alpha*vec + (1-alpha)*text
           → apply small entity boost/penalty when query clearly targets
             Oracle or PostgreSQL
           → sort desc, truncate top_k, build RecallResult (+ reason)
```

### Open / recovery

```
Storage::open → create data dir → load snapshot.json (base State)
             → replay wal.log line-by-line on top (skip torn trailing line)
Hippocore.open → rebuild Index from State (chunks + memories)
```

### Compaction

`compact()` serializes the current `State` to `snapshot.json` atomically
(temp file → fsync → rename) then truncates `wal.log` to zero length.

## The unified index entry

Both document chunks and memories are projected into a single `IndexEntry`
(`kind` distinguishes them). This keeps retrieval uniform — one vector scan and
one inverted index serve both — while the persisted `Document`/`Chunk`/`Memory`
models stay distinct and strongly typed.

## Documents, chunks, memories, indexed entries, WAL entries

- `Document`: persisted source text managed as one logical object.
- `Chunk`: persisted slice of a document, with its own embedding.
- `Memory`: persisted atomic remembered item, independent of documents.
- `Indexed Entry`: in-memory search projection of either one chunk or one
  memory. This is what recall scores.
- `WAL Entry`: durable append-only operation (`remember`, `put document`,
  delete, etc.) used for crash recovery and replay. It is not searchable.

A database may legitimately show `documents=0` and `memories>0` when an app uses
`remember` directly, as the TypeScript RAG example does. In that case each
memory still contributes one indexed entry, so recall works without documents.

## Tenant isolation

`Filter.tenant_id` is mandatory and checked first for every candidate. No code
path returns an entry from a different tenant.

## Boundaries / invariants

- `storage` knows operations and bytes, not scoring.
- `index`/`query` never touch disk.
- `memory::Embedder` and `index::cosine_similarity` never panic on bad input.
- `unsafe` is `forbid`den crate-wide.
