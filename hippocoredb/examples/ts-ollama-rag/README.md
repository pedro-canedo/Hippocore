# Hippocore + Ollama — TypeScript RAG demo

A small local RAG loop that proves Hippocore being used as the **memory /
context database** for an AI:

- **Ollama** produces the embeddings (`nomic-embed-text`) and generates the
  final answer (`llama3.2`).
- **Hippocore** stores the embedded passages and does the **hybrid recall**
  (vector + text) that feeds the model's context.

TypeScript talks to Hippocore by spawning the `hippocore` CLI as a subprocess
(the MVP is embedded/local-first — no HTTP server yet). The CLI's `recall
--json --embedding ...` is what makes this clean.

```
ingest:  text  ── Ollama /api/embeddings ──► vector ──► hippocore remember
ask:     query ── Ollama /api/embeddings ──► vector ──► hippocore recall --json
         retrieved context + query ── Ollama /api/chat ──► grounded answer
```

## Prerequisites

1. **Build the Hippocore CLI** (from the repo root `hippocoredb/`):
   ```bash
   cargo build -p hippocore-cli      # produces target/debug/hippocore
   ```
2. **Node.js 18+** (for the global `fetch`).
3. **Ollama** running locally with the models pulled:
   ```bash
   ollama serve            # if not already running
   ollama pull nomic-embed-text
   ollama pull llama3.2
   ```

## Run

```bash
cd examples/ts-ollama-rag
npm install
npm start                                   # default question
npm start -- "Como inicio o listener do Oracle?"
```

## Configuration (env vars)

| Variable        | Default                         | Meaning                          |
|-----------------|---------------------------------|----------------------------------|
| `HIPPOCORE_BIN` | `../../target/debug/hippocore`  | Path to the built CLI binary.    |
| `DB_DIR`        | `./data`                        | Hippocore data directory.        |
| `OLLAMA_URL`    | `http://localhost:11434`        | Ollama base URL.                 |
| `EMBED_MODEL`   | `nomic-embed-text`              | Embedding model.                 |
| `CHAT_MODEL`    | `llama3.2`                      | Generation model.                |

The knowledge base lives in `src/index.ts` (`KNOWLEDGE`). Passages use stable
ids, so re-running overwrites them instead of creating duplicates — Hippocore's
`version`/overwrite semantics in action.

Ingestion is **idempotent**: a marker file (`<DB_DIR>/.ingested.json`) stores a
hash of each passage's text, so re-runs **skip re-embedding** unchanged passages
(no redundant Ollama calls). Change a passage's text and only that one is
re-embedded.

The demo calls **`hippocore compact`** at the end. Without it the append-only
WAL grows on every run (each 768-dim embedding is ~9 KB of JSON); `compact`
folds the WAL into a fresh snapshot and truncates the log, so disk usage stays
bounded across runs.

## What to look at

- `src/hippocore.ts` — typed wrapper over the CLI (`remember`, `recall --json`).
- `src/ollama.ts` — embeddings + chat via the Ollama HTTP API.
- `src/index.ts` — the ingest → recall → generate loop with grounded citations.

> Note: embeddings here are real (e.g. 768-dim for `nomic-embed-text`); the
> stored memories carry those vectors, so recall is genuine semantic search —
> the built-in deterministic embedder is bypassed because we pass embeddings
> explicitly to both `remember` and `recall`.
