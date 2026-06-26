# Hippocore DB

> **Hippocore DB is not just a vector store. It is a memory database for AI applications.**

Hippocore DB is an open-source, **local-first** AI-native database that acts as
long-term **memory and contextual retrieval** infrastructure for agents,
copilots, RAG applications and multi-tenant AI systems. It runs embedded in your
process, persists to a local directory, and recalls context through **vector**,
**text**, and **hybrid** search — with **tenant isolation** built in.

---

## What it is

- An **embedded** Rust library (`hippocore`) plus a **CLI** (`hippocore`).
- A durable store for **documents** (auto-chunked) and **memories**.
- **Vector** (cosine), **text** (TF-IDF inverted index), and **hybrid** recall.
- **Multi-tenant**: every recall is scoped to a tenant; data never leaks across.
- **Crash-safe**: a write-ahead log + atomic snapshot; recovery rebuilds state
  on open and a torn trailing WAL line is skipped, not fatal.
- **Offline & deterministic**: a built-in hashing embedder means no network and
  reproducible tests. You can also supply your own embeddings.

## What it is *not* (yet)

No clustering, consensus, production auth, cloud embedders, GPU/ANN/HNSW, web
dashboard, replication, SQL/query language, or external database. See
[docs/MVP_SCOPE.md](docs/MVP_SCOPE.md) and [ROADMAP.md](ROADMAP.md).

## Why a memory database (not just a vector store)

Vector similarity alone is not reliable context:

- Agents **forget** between sessions — they need durable, queryable memory.
- RAG retrieves chunks that are *similar* but **outdated or contradictory**.
- Teams need **tenant isolation** and provenance (`source`) for every result.
- Pure vector recall misses exact-keyword hits; pure keyword misses paraphrases.
  **Hybrid** recall fuses both, and every result explains **why** it matched.

## Core concepts (data model)

| Entity       | Purpose                                                            |
|--------------|-------------------------------------------------------------------|
| `Tenant`     | Top-level isolation boundary.                                     |
| `Collection` | Named group of documents/memories within a tenant.               |
| `Document`   | A stored text, auto-split into `Chunk`s for retrieval.           |
| `Chunk`      | A searchable slice of a document, with its own embedding.        |
| `Memory`     | A long-term memory item (episodic/semantic/procedural/note).     |
| `Embedding`  | `Vec<f32>`; produced by the built-in embedder or supplied by you.|
| `Metadata`   | Exact-match `String→String` filter keys.                         |
| `Source`     | Provenance/citation (label, optional URI/title).                 |
| `RecallResult` | A hit with `score`, `vector_score`, `text_score`, and `reason`.|

## Quickstart (CLI)

```bash
cargo build --workspace
BIN=target/debug/hippocore

# 1. initialize a data directory
$BIN init --db ./data

# 2. store a memory (tenant + collection auto-created for convenience)
$BIN remember --db ./data --tenant acme --collection support \
  --type semantic --text "A customer runs Oracle in the staging environment" \
  --meta environment=staging

# 3. store a document (auto-chunked + embedded)
$BIN put-document --db ./data --tenant acme --collection support \
  --text "Oracle ORA-12514 means the listener does not know the service. Check tnsnames.ora."

# 4. recall context (hybrid by default; --mode vector|text|hybrid)
$BIN recall --db ./data --tenant acme --query "oracle ORA-12514 in staging" --top-k 5

# 5. inspect & stats
$BIN inspect --db ./data
$BIN stats --db ./data
```

## Quickstart (Rust)

```rust
use hippocore::model::MemoryType;
use hippocore::{Config, Hippocore, RememberRequest, RecallRequest};

let mut db = Hippocore::open(Config::new("./data"))?;
db.create_tenant("acme", "Acme Corp")?;
db.create_collection("acme", "support", "support knowledge")?;

db.remember(RememberRequest::new(
    "acme", "support", MemoryType::Semantic,
    "A customer runs Oracle in the staging environment",
))?;

let hits = db.recall(RecallRequest::new("acme", "oracle HML environment"))?;
for h in &hits {
    println!("{} (score {:.3}) — {}", h.id, h.score, h.reason);
}
db.close()?;
# Ok::<(), hippocore::HippocoreError>(())
```

Run the full example: `cargo run -p hippocore --example basic_usage`.

## Integrating with an LLM (RAG)

Hippocore stores the **memory/context**; your model provides embeddings and
generation. A runnable local example using **Ollama** (embeddings +
generation) over Hippocore lives in
[`examples/ts-ollama-rag/`](examples/ts-ollama-rag/) — TypeScript drives the
`hippocore` CLI (`recall --json --embedding ...`) for a full ingest → recall →
answer loop with citations.

## Public API

`Hippocore::open`, `create_tenant`, `create_collection`, `store_document`,
`remember`, `recall`, `search`, `forget`, `delete_document`, `stats`, `compact`,
`close`. Every fallible call returns `hippocore::Result<T>` — no panics on normal
errors.

**Embeddings.** `remember`, `recall` and `store_document` all accept
caller-supplied embeddings from any model. For documents, set
`StoreDocumentRequest.chunks` with pre-embedded `ChunkInput`s to bypass the
built-in embedder:

```rust
use hippocore::{ChunkInput, StoreDocumentRequest};

let mut req = StoreDocumentRequest::new("acme", "support", "full document body");
req.chunks = Some(vec![
    ChunkInput::new("first passage", embed("first passage")),   // your model
    ChunkInput::new("second passage", embed("second passage")),
]);
db.store_document(req)?;
# Ok::<(), hippocore::HippocoreError>(())
```

When `chunks` is `None`, the document is auto-chunked and embedded with the
built-in deterministic embedder.

## Persistence model

```
<data_dir>/
  wal.log        append-only log; one op per line as <crc32>\t<json>
  snapshot.json  compacted state, written atomically (temp + fsync + rename)
  meta.json      format version
```

On `open`, Hippocore loads the snapshot then replays the WAL on top, rebuilding
the in-memory vector and text indexes. Each WAL line carries a **CRC32**, so
recovery detects corruption and stops safely (a torn trailing line or a checksum
mismatch never panics and never loads bad data). The WAL is bounded by
**automatic compaction** (configurable via `Config.auto_compact_after_ops` /
`auto_compact_after_bytes`); `compact()` (and `hippocore compact`) also fold the
WAL into a fresh snapshot on demand.

## Architecture

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md). Modules: `config`, `errors`,
`model`, `storage`, `index`, `memory` (embedder + chunker), `query`, `cli`, with
the `Hippocore` engine in `lib.rs`.

## Status, roadmap, decisions

- Current status: [docs/STATUS.md](docs/STATUS.md)
- Next feature: [docs/NEXT_FEATURE.md](docs/NEXT_FEATURE.md)
- Decisions: [docs/DECISIONS.md](docs/DECISIONS.md)
- Changelog: [CHANGELOG.md](CHANGELOG.md)
- Roadmap: [ROADMAP.md](ROADMAP.md)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Before a PR run:

```bash
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## License

[MIT](LICENSE).
