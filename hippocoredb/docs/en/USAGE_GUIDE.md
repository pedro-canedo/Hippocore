# Hippocore DB — Complete Usage Guide

This guide covers every way to use Hippocore DB, when to pick each one, and
concrete examples for each path.

---

## The four integration modes

| Mode | When to use | Binary / crate |
|---|---|---|
| **Embedded Rust library** | Your application is written in Rust | `crates/hippocore` |
| **CLI** | Scripting, local development, testing, one-off ingestion | `hippocore` binary |
| **HTTP server** | Any language/runtime, microservices, Docker deployments | `hippocore serve` |
| **Interactive TUI** | Exploring data, debugging recalls, manual review | `hippocore studio` |

---

## Mode 1 — Embedded Rust library

Add to your `Cargo.toml`:

```toml
[dependencies]
hippocore = { path = "path/to/hippocore" }   # or version from crates.io
```

### Open and initialise

```rust
use hippocore::{Config, Hippocore};

let mut db = Hippocore::open(Config::new("./data"))?;

// Each tenant is an isolation boundary. Collections group documents/memories.
db.create_tenant("acme", "Acme Corp")?;
db.create_collection("acme", "support", "support knowledge base")?;
```

`create_collection` is **idempotent** — calling it twice for the same
(tenant, name) pair returns the existing collection with no error.

### Store a memory (atomic, already embedded)

```rust
use hippocore::model::MemoryType;
use hippocore::RememberRequest;

let mem = db.remember(RememberRequest::new(
    "acme", "support", MemoryType::Semantic,
    "Oracle ORA-12514 means the listener does not know the service.",
))?;
println!("stored: {}", mem.id);
```

Memory types:
- `Semantic` — a fact ("the DB is Oracle 19c")
- `Episodic` — an event ("user reported login failure at 14:32 UTC")
- `Procedural` — a how-to ("to reset a password: …")
- `Note` — freeform note

### Store a document (auto-chunked + embedded)

```rust
use hippocore::StoreDocumentRequest;

let doc = db.store_document(StoreDocumentRequest::new(
    "acme", "support",
    "Chapter 1: The ORA-12514 error occurs when the TNS listener \
     cannot find the service name. Check tnsnames.ora and listener.log.",
))?;
println!("stored: {} ({} chars)", doc.id, doc.text.len());
```

Documents are split into overlapping token-window chunks automatically.
Each chunk gets its own embedding and is independently searchable.

### Store a PDF

```rust
let bytes = std::fs::read("manual.pdf")?;
let doc = db.store_document(StoreDocumentRequest::from_pdf(
    "acme", "support", bytes,
))?;
```

### Store source code

```rust
let code = std::fs::read_to_string("src/auth.rs")?;
let doc = db.store_document(StoreDocumentRequest::from_code(
    "acme", "codebase", code, "rust",
))?;
```

Supported languages: `rust`, `python`, `javascript`, `typescript`, `go`,
`java`, `kotlin`. Unrecognised languages fall back to the token chunker.

### Store a structured record (JSON, projected into searchable text)

```rust
use hippocore::PutRecordRequest;
use serde_json::json;

let rec = db.put_record(PutRecordRequest::new(
    "acme", "support", "systems",
    json!({"engine": "postgresql", "port": 5432, "env": "prod"}),
))?;
println!("record version: {}", rec.version);
```

### Recall (hybrid by default)

```rust
use hippocore::RecallRequest;

let hits = db.recall(RecallRequest::new("acme", "oracle connection error"))?;
for h in &hits {
    println!("[{:.3}] {} — {}", h.score, h.id, &h.text[..80.min(h.text.len())]);
}
```

`recall` blends **vector similarity** (cosine) and **text relevance** (BM25)
for every hit. To use one mode only:

```rust
use hippocore::query::SearchMode;

let mut req = RecallRequest::new("acme", "oracle");
req.mode = SearchMode::Vector;   // or SearchMode::Text
req.top_k = 5;
req.collection = Some("support".into());
let hits = db.recall(req)?;
```

### Build LLM context (token-budget assembly)

```rust
use hippocore::BuildContextRequest;

let ctx = db.build_context(BuildContextRequest::new(
    "acme", "oracle ORA-12514 in staging", 2048,  // max tokens
))?;

// ctx.text is ready to be prepended to your LLM prompt.
println!("context ({} tokens, {} items):\n{}", ctx.token_count, ctx.items_included.len(), ctx.text);
```

With graph neighbours included:

```rust
let mut req = BuildContextRequest::new("acme", "oracle", 2048);
req.include_related = true;   // follow 1-hop graph edges
let ctx = db.build_context(req)?;
```

### Graph edges

```rust
use hippocore::{AddGraphEdgeRequest, ItemKind};

db.add_graph_edge(AddGraphEdgeRequest {
    tenant_id: "acme".into(),
    from_kind: ItemKind::Memory,
    from_id: mem.id.clone(),
    to_kind: ItemKind::DocumentChunk,
    to_id: "doc-1#0".into(),
    relation: "supports".into(),
    weight: 1.0,
})?;
```

Multi-hop traversal:

```rust
use hippocore::{TraverseGraphRequest, ItemKind};

let nodes = db.traverse_graph(TraverseGraphRequest::new(
    "acme",
    [(ItemKind::Memory, mem.id.clone())],
));
for n in &nodes {
    println!("hop {} — {} {:?} via {}", n.hop, n.id, n.kind, n.via_edge_id);
}
```

### Maintenance

```rust
// Statistics
let s = db.stats()?;
println!("{} tenants, {} memories, {} chunks, {} wal entries",
    s.tenants, s.memories, s.chunks, s.wal_entries);

// Compact WAL into snapshot (happens automatically; also callable manually)
db.compact()?;

// Graceful close (flushes and closes WAL file)
db.close()?;
```

### Temporal validity

```rust
use hippocore::RememberRequest;

let epoch_ms = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64;

let mut req = RememberRequest::new("acme", "support", MemoryType::Semantic,
    "This fact is valid for the next 30 days.");
req.valid_from = Some(epoch_ms);
req.valid_until = Some(epoch_ms + 30 * 24 * 3600 * 1000);
db.remember(req)?;
```

Expired memories are excluded from `recall` automatically (based on wall-clock
time). Use `req.as_of = Some(timestamp)` to query historical state.

---

## Mode 2 — CLI

### Installation

```bash
git clone <repo> && cd hippocoredb
cargo build --release
# Binary is at: target/release/hippocore
# Add to PATH or alias:
alias hc=./target/release/hippocore
```

### Every subcommand

```bash
# ─── Database lifecycle ────────────────────────────────────────────────────────

hc init --db ./data                        # create/validate data directory

hc stats --db ./data                       # summary: tenants, docs, memories, WAL
hc inspect --db ./data                     # list tenants and collection counts

hc compact --db ./data                     # fold WAL into snapshot


# ─── Tenants & collections ────────────────────────────────────────────────────

hc list-tenants --db ./data
hc list-collections --db ./data --tenant acme


# ─── Documents ────────────────────────────────────────────────────────────────

hc put-document --db ./data \
    --tenant acme --collection support \
    --text "Oracle ORA-12514 — check tnsnames.ora and listener.log." \
    --meta source=wiki --meta priority=high

hc put-document --db ./data \
    --tenant acme --collection support \
    --text "..." --id custom-doc-id          # explicit id (idempotent upsert)

hc list-documents --db ./data --tenant acme
hc show-document  --db ./data --tenant acme --collection support --id <id>
hc delete-document --db ./data --tenant acme --collection support --id <id>


# ─── Memories ─────────────────────────────────────────────────────────────────

hc remember --db ./data \
    --tenant acme --collection support \
    --type semantic \                         # episodic | semantic | procedural | note
    --text "A customer runs Oracle 19c on staging." \
    --meta env=staging \
    --confidence 0.9

hc remember --db ./data --tenant acme --collection support \
    --type episodic \
    --text "Login failure spike at 14:32 UTC 2026-06-26." \
    --valid-from 1751000000000 \
    --valid-until 1759000000000

hc list-memories  --db ./data --tenant acme
hc show-memory    --db ./data --tenant acme --collection support --id <id>
hc forget         --db ./data --tenant acme --collection support --id <id>
hc rate-memory    --db ./data --tenant acme --collection support --id <id> \
    --confidence 0.95                         # human-in-the-loop rating


# ─── Records (structured JSON) ────────────────────────────────────────────────

hc put-record --db ./data \
    --tenant acme --collection support --table systems --id billing-db \
    --json '{"engine":"postgresql","port":5432,"env":"prod"}'

hc list-records  --db ./data --tenant acme --table systems
hc show-record   --db ./data --tenant acme --collection support \
    --table systems --id billing-db
hc delete-record --db ./data --tenant acme --collection support \
    --table systems --id billing-db


# ─── File import (text-like: .txt, .md, .json, .csv) ─────────────────────────

hc import-file --db ./data \
    --tenant acme --collection support \
    --id runbook --path ./runbook.md \
    --meta source=confluence

hc list-files   --db ./data --tenant acme
hc show-file    --db ./data --tenant acme --collection support --id runbook
hc delete-file  --db ./data --tenant acme --collection support --id runbook


# ─── Recall ───────────────────────────────────────────────────────────────────

hc recall --db ./data --tenant acme \
    --query "oracle connection refused" --top-k 5

hc recall --db ./data --tenant acme --query "oracle" \
    --mode vector                              # vector | text | hybrid (default)

hc recall --db ./data --tenant acme --query "oracle" \
    --collection support                       # scope to one collection

hc recall --db ./data --tenant acme --query "oracle" \
    --json                                     # machine-readable output

# With external embeddings (e.g. from Ollama / OpenAI):
hc recall --db ./data --tenant acme --query "oracle" \
    --embedding "0.12,-0.34,0.56,..."


# ─── Build LLM context ────────────────────────────────────────────────────────

hc build-context --db ./data --tenant acme \
    --query "oracle ORA-12514 in staging" \
    --max-tokens 2048

hc build-context --db ./data --tenant acme \
    --query "oracle" --max-tokens 1024 --json   # returns ContextBlock JSON


# ─── Graph edges ──────────────────────────────────────────────────────────────

hc add-edge --db ./data --tenant acme \
    --from-kind Memory --from-id <mem-id> \
    --to-kind DocumentChunk --to-id <doc-id>#0 \
    --relation supports

hc list-edges  --db ./data --tenant acme
hc delete-edge --db ./data --tenant acme --id <edge-id>


# ─── Audit ────────────────────────────────────────────────────────────────────

hc audit --db ./data --tenant acme --from 0 --to 9223372036854775807
                                             # replay all audit records
hc audit --db ./data --tenant acme \
    --from 1750000000000 --to 1760000000000   # time-windowed audit

hc compact-audit --db ./data --max-records 500
hc compact-audit --db ./data --max-bytes 1048576


# ─── Retrieval quality ────────────────────────────────────────────────────────

hc eval-quality --db ./data --tenant acme     # run retrieval fixtures, print metrics
```

### Machine-readable output

All read commands accept `--json` to emit structured JSON to stdout, suitable
for piping to `jq` or other tools:

```bash
hc recall --db ./data --tenant acme --query "oracle" --top-k 3 --json | \
    jq '.[].text'
```

---

## Mode 3 — HTTP server

### Start the server

```bash
# From env var (recommended):
HIPPOCORE_API_KEY=changeme hippocore serve --port 8080 --db ./data

# From flag:
hippocore serve --port 8080 --db ./data --api-key changeme
```

Server logs its address to stderr. No auth on `/health`; all other endpoints
require the `X-Api-Key: changeme` header.

### Health check

```bash
curl http://localhost:8080/health
# 200 OK
```

### Full endpoint reference

```bash
BASE=http://localhost:8080
KEY=changeme
AUTH="-H 'X-Api-Key: $KEY'"

# ─── Stats ────────────────────────────────────────────────────────────────────
curl $AUTH $BASE/stats

# ─── Tenant ───────────────────────────────────────────────────────────────────
curl -X POST $AUTH -H 'Content-Type: application/json' \
    -d '{"id":"acme","name":"Acme Corp"}' \
    $BASE/tenants

# ─── Collection ───────────────────────────────────────────────────────────────
curl -X POST $AUTH -H 'Content-Type: application/json' \
    -d '{"name":"support","description":"Support KB"}' \
    $BASE/tenants/acme/collections

# ─── Memory ───────────────────────────────────────────────────────────────────
curl -X POST $AUTH -H 'Content-Type: application/json' \
    -d '{"collection":"support","text":"Oracle ORA-12514 error","memory_type":"semantic"}' \
    $BASE/tenants/acme/memories

curl -X DELETE $AUTH $BASE/tenants/acme/memories/<id>

# ─── Document ─────────────────────────────────────────────────────────────────
curl -X POST $AUTH -H 'Content-Type: application/json' \
    -d '{"collection":"support","text":"Full document text here..."}' \
    $BASE/tenants/acme/documents

# ─── Recall ───────────────────────────────────────────────────────────────────
curl -X POST $AUTH -H 'Content-Type: application/json' \
    -d '{"query":"oracle connection","top_k":5}' \
    $BASE/tenants/acme/recall

# Response: [{"id":"...","score":0.87,"text":"...","kind":"Memory","document_id":null}]

# ─── Build context ────────────────────────────────────────────────────────────
curl -X POST $AUTH -H 'Content-Type: application/json' \
    -d '{"query":"oracle ORA-12514","max_tokens":2048,"include_related":false}' \
    $BASE/tenants/acme/context

# Response: {"text":"[Memory:mem-123]\nOracle ORA...","token_count":142,"items_included":3,"items_dropped":0}

# ─── Graph traversal ─────────────────────────────────────────────────────────
curl -X POST $AUTH -H 'Content-Type: application/json' \
    -d '{"seeds":[{"kind":"Memory","id":"mem-123"}],"max_hops":2,"max_nodes":20}' \
    $BASE/tenants/acme/graph/traverse
```

### Using from Python

```python
import requests

BASE = "http://localhost:8080"
HEADERS = {"X-Api-Key": "changeme", "Content-Type": "application/json"}

# Setup
requests.post(f"{BASE}/tenants", json={"id": "acme", "name": "Acme"}, headers=HEADERS)
requests.post(f"{BASE}/tenants/acme/collections",
              json={"name": "support"}, headers=HEADERS)

# Store
requests.post(f"{BASE}/tenants/acme/memories",
              json={"collection": "support", "text": "Oracle error ORA-12514"},
              headers=HEADERS)

# Recall
hits = requests.post(f"{BASE}/tenants/acme/recall",
                     json={"query": "oracle connection", "top_k": 5},
                     headers=HEADERS).json()
for h in hits:
    print(f"[{h['score']:.3f}] {h['text'][:80]}")

# Build context for LLM
ctx = requests.post(f"{BASE}/tenants/acme/context",
                    json={"query": "oracle ORA-12514", "max_tokens": 2048},
                    headers=HEADERS).json()
prompt = f"Context:\n{ctx['text']}\n\nQuestion: What is ORA-12514?"
```

### Using from TypeScript / Node.js

```typescript
const BASE = "http://localhost:8080";
const headers = { "X-Api-Key": "changeme", "Content-Type": "application/json" };

// Store a memory
await fetch(`${BASE}/tenants/acme/memories`, {
  method: "POST",
  headers,
  body: JSON.stringify({ collection: "support", text: "Oracle ORA-12514 error" }),
});

// Recall
const hits = await fetch(`${BASE}/tenants/acme/recall`, {
  method: "POST",
  headers,
  body: JSON.stringify({ query: "oracle connection error", top_k: 5 }),
}).then(r => r.json());

// Build LLM context
const ctx = await fetch(`${BASE}/tenants/acme/context`, {
  method: "POST",
  headers,
  body: JSON.stringify({ query: "oracle ORA-12514", max_tokens: 2048 }),
}).then(r => r.json());

const prompt = `Context:\n${ctx.text}\n\nQuestion: What is ORA-12514?`;
```

### Docker

```dockerfile
FROM rust:1.96-slim AS builder
WORKDIR /build
COPY . .
RUN cargo build --release --package hippocore-cli

FROM debian:bookworm-slim
COPY --from=builder /build/target/release/hippocore /usr/local/bin/hippocore
VOLUME /data
ENV HIPPOCORE_API_KEY=changeme
EXPOSE 8080
CMD ["hippocore", "serve", "--port", "8080", "--db", "/data"]
```

```bash
docker build -t hippocore .
docker run -p 8080:8080 -v $(pwd)/data:/data \
    -e HIPPOCORE_API_KEY=supersecret hippocore
```

---

## Mode 4 — Interactive TUI (Studio)

```bash
hippocore studio --db ./data
```

### Navigation

| Key | Action |
|---|---|
| `Tab` / `Shift-Tab` | Switch between tabs |
| `↑` / `↓` | Navigate list items |
| `q` | Quit |

### Tabs

- **Tenants** — list all tenants and their collection counts
- **Memories** — browse memories with text preview; filterable by tenant
- **Documents** — browse documents and their chunk counts
- **Stats** — live database statistics (WAL entries, disk bytes, indexed entries)

The TUI reads data on open; it is read-only (no writes from the UI).
Use it alongside the CLI or library for inspection and debugging.

---

## Choosing the right mode

```
Your app is in Rust?
  └── Embedded library  ← zero network overhead, transactions in-process

You're scripting / running one-off ingestion?
  └── CLI  ← no server needed, pipe-friendly --json output

You need any-language access, or want to run behind a reverse proxy?
  └── HTTP server  ← start once, call from Python/JS/Go/…

You want to visually browse data or debug a recall?
  └── TUI studio  ← no coding required
```

---

## RAG pattern (end-to-end)

```
┌──────────────┐   ingest   ┌──────────────────┐
│ documents /  │──────────► │  Hippocore DB     │
│ memories /   │            │  (store_document  │
│ PDF / code   │            │   remember)       │
└──────────────┘            └────────┬─────────┘
                                     │ recall / build_context
                     ┌───────────────▼──────────────────┐
                     │ LLM (Ollama / OpenAI / Anthropic) │
                     │  prompt = ctx.text + question     │
                     └───────────────────────────────────┘
```

1. **Ingest**: `store_document` or `remember` (CLI, library, or HTTP)
2. **Recall**: `recall(query)` → ranked hits (hybrid vector + BM25)
3. **Assemble context**: `build_context(query, max_tokens)` → ready-to-use string
4. **Generate**: prepend `ctx.text` to your LLM prompt; cite `ctx.items_included`

The built-in embedder is deterministic and offline. For production quality,
supply your own embeddings from a model (OpenAI, Ollama, etc.) via
`RememberRequest.embedding` or `StoreDocumentRequest.chunks`.

---

## Persistence and crash safety

```
<data_dir>/
  wal.log        write-ahead log; one op per line: <crc32-hex>\t<json>\n
  snapshot.json  full state, written atomically (tmp + fsync + rename)
  meta.json      format version
```

- Every write appends to `wal.log` with a CRC32 checksum.
- On `open`, the snapshot is loaded, then the WAL is replayed on top.
- A torn trailing WAL line (process crash mid-write) is skipped, never fatal.
- CRC32 mismatch stops replay at that line — no corrupted state ever loads.
- Automatic compaction fires when WAL exceeds `auto_compact_after_ops` (default
  1000) or `auto_compact_after_bytes`. `compact()` can also be called manually.

---

## Multi-tenant isolation

Every API method that reads or writes data requires an explicit `tenant_id`.
The isolation is enforced in the query layer: vector search, text search, graph
traversal, and context assembly all filter by `tenant_id` before scoring.
There is no shared state between tenants even inside the same data directory.

---

## Configuration

```rust
use hippocore::Config;

let cfg = Config::new("./data")
    .with_embedding_dim(256)           // default 128
    .with_chunk_tokens(64)             // default 48; tokens per chunk window
    .with_auto_compact_after_ops(500)  // default 1000
    .with_auto_compact_after_bytes(5 * 1024 * 1024);  // default 10 MiB
```

All settings are per-process; there is no on-disk config file.
