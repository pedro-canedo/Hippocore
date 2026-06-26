# Hippocore DB

> **Hippocore DB is not just a vector store. It is a memory database for AI applications.**

Hippocore DB is an open-source, **local-first** AI-native database that acts as
long-term **memory and contextual retrieval** infrastructure for agents,
copilots, RAG applications and multi-tenant AI systems. It runs embedded in your
process, persists to a local directory, and recalls context through **vector**,
**text**, and **hybrid** search — with **tenant isolation** built in.

When you run `hippocore serve`, it also exposes a web admin console at
`/admin` for browsing data, rotating API keys, and generating integration
snippets for local workflows such as Ollama.

The product direction is broader than storing embeddings: documents, memories,
structured records, files and metadata should become manageable database objects
that can be projected into native context for SDKs, agents and RAG systems.

For complete usage patterns, examples, and integration choices, see:

- [docs/en/USAGE_GUIDE.md](docs/en/USAGE_GUIDE.md)
- [docs/pt-br/USAGE_GUIDE.md](docs/pt-br/USAGE_GUIDE.md)
- [docker-compose.yml](docker-compose.yml)

---

## What it is

- An **embedded** Rust library (`hippocore`) plus a **CLI** (`hippocore`).
- A durable store for **documents** (auto-chunked), **memories**, JSON-first
  structured **records**, and imported text-like **files**.
- **Vector** (cosine), **text** (BM25 inverted index), and **hybrid** recall.
- **Multi-tenant**: every recall is scoped to a tenant; data never leaks across.
- **Crash-safe**: a write-ahead log + atomic snapshot; recovery rebuilds state
  on open and a torn trailing WAL line is skipped, not fatal.
- **Offline & deterministic**: a built-in hashing embedder means no network and
  reproducible tests. You can also supply your own embeddings.
- A tiny SQL-like read layer for records: `SELECT * FROM <table> WHERE ...`.
- A foundation for a broader context database where records, files and future
  data types can be managed and projected into searchable context.

## What it is *not* (yet)

No clustering, consensus, production auth, cloud embedders, GPU/ANN/HNSW,
replication, full SQL engine, or external database yet. See
[docs/en/MVP_SCOPE.md](docs/en/MVP_SCOPE.md), [docs/en/SQL.md](docs/en/SQL.md),
and [ROADMAP.md](ROADMAP.md).

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
| `Record`     | A structured JSON object in a logical table namespace.           |
| `FileObject` | Imported text-like file metadata linked to a derived document. |
| `Indexed Entry` | In-memory searchable projection of one `Chunk`, `Memory`, or `Record`. |
| `WAL Entry`  | Durable append-only operation record replayed on database open.  |
| `Embedding`  | `Vec<f32>`; produced by the built-in embedder or supplied by you.|
| `Metadata`   | Exact-match `String→String` filter keys.                         |
| `Source`     | Provenance/citation (label, optional URI/title).                 |
| `RecallResult` | A hit with `score`, `vector_score`, `text_score`, and `reason`.|

Documents and memories are intentionally distinct. A `Document` is source text
managed as a whole and split into persisted `Chunk`s; each chunk becomes one
indexed entry. A `Memory` is already an atomic remembered fact/procedure/note and
also becomes one indexed entry directly. A `Record` preserves its original JSON
payload and stores a deterministic text projection for retrieval. A `FileObject`
stores import metadata and links to a derived document whose chunks are
retrievable. The WAL stores write/delete operations, not retrieval hits.

Because memories do not create `Document` records, a memory-only RAG example can
correctly report `documents=0` while `memories>0` and `indexed_entries>0`.

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

# 4. store a structured record (JSON-first, projected into context)
$BIN put-record --db ./data --tenant acme --collection support --table systems \
  --id billing-db --json '{"engine":"postgresql","port":5432,"env":"prod"}'

# 5. import a text-like file (metadata stored, text projected into chunks)
$BIN import-file --db ./data --tenant acme --collection support \
  --id notes --path ./notes.md --meta source=local

# 6. query structured records with the MVP SQL-like layer
$BIN query --db ./data --tenant acme \
  --sql "select * from systems where engine = 'postgresql' limit 5" --json

# 7. recall context (hybrid by default; --mode vector|text|hybrid)
$BIN recall --db ./data --tenant acme --query "oracle ORA-12514 in staging" --top-k 5

# 8. inspect & stats
$BIN inspect --db ./data
$BIN stats --db ./data
```

## Web admin and Docker Compose

The server also serves a browser admin console at `http://localhost:8080/admin`
when run with `hippocore serve`.

For the local-first container setup:

```bash
docker compose up --build
```

Then open `http://localhost:8080/admin`. The default admin user is `admin`. If
`HIPPOCORE_ADMIN_PASSWORD` is not set, the container generates one password for
that data volume, prints it in the first startup logs, and saves it at
`./hippocore-data/admin-password`.

```bash
docker compose logs --no-color hippocore
cat ./hippocore-data/admin-password
```

`HIPPOCORE_API_KEY` remains the service credential for CLI/API automation and
can be rotated from the UI. The compose file also includes an optional `ollama`
profile and local LLM provider registry support for Ollama, OpenRouter, and
compatible endpoints.

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
`remember`, `put_record`, `import_file`, `recall`, `search`, `forget`,
`delete_document`, `delete_record`, `delete_file`, `stats`, `compact`, `close`.
Every fallible call returns
`hippocore::Result<T>` — no panics on normal errors.

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

See [docs/en/ARCHITECTURE.md](docs/en/ARCHITECTURE.md). Modules: `config`, `errors`,
`model`, `storage`, `index`, `memory` (embedder + chunker), `query`, `cli`, with
the `Hippocore` engine in `lib.rs`.

## Status, roadmap, decisions

- Current status: [docs/en/STATUS.md](docs/en/STATUS.md) /
  [docs/pt-br/STATUS.md](docs/pt-br/STATUS.md)
- Next feature: [docs/en/NEXT_FEATURE.md](docs/en/NEXT_FEATURE.md) /
  [docs/pt-br/NEXT_FEATURE.md](docs/pt-br/NEXT_FEATURE.md)
- Usage guide: [docs/en/USAGE_GUIDE.md](docs/en/USAGE_GUIDE.md) /
  [docs/pt-br/USAGE_GUIDE.md](docs/pt-br/USAGE_GUIDE.md)
- Data model direction: [docs/en/DATA_MODEL.md](docs/en/DATA_MODEL.md) /
  [docs/pt-br/DATA_MODEL.md](docs/pt-br/DATA_MODEL.md)
- Admin interface direction: [docs/en/ADMIN_INTERFACE.md](docs/en/ADMIN_INTERFACE.md) /
  [docs/pt-br/ADMIN_INTERFACE.md](docs/pt-br/ADMIN_INTERFACE.md)
- Decisions: [docs/en/DECISIONS.md](docs/en/DECISIONS.md) /
  [docs/pt-br/DECISIONS.md](docs/pt-br/DECISIONS.md)
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
