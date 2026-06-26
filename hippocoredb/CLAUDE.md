# CLAUDE.md

Guidance for Claude Code (and other AI assistants) working in this repository.

> **Read [`regras.md`](regras.md) first.** It is the project's golden rules
> (vision, high-performance invariants, persistence safety, quality gate,
> open-source/privacy). They are binding; this file expands on the "how".

## Project vision

Hippocore DB is an open-source, local-first, embedded **AI-native memory
database** written in Rust. The tagline: *"not just a vector store — a memory
database for AI applications."*

The long-term goal is a **context database** for AI agents and RAG: documents,
memories, embeddings, metadata, temporal facts, source confidence and audit
information. See `ROADMAP.md`. The current state and the next planned feature
live in `docs/en/STATUS.md` and `docs/en/NEXT_FEATURE.md` — **read them first** and
**update them when you finish work**.

**Keep it small, correct, well-tested and extensible.** Do not pull future
phases forward.

## Build & run commands

```bash
cargo build --workspace                      # build everything
cargo run -p hippocore --example basic_usage # run the example
cargo run -p hippocore-cli -- stats --db ./data   # run the CLI
cargo bench -p hippocore                     # baseline benchmarks
```

## Test commands

```bash
cargo test --workspace
```

## Quality gate — run before completing ANY work

```bash
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

All three must be green. Prefer fixing code over silencing clippy. **Rule: never
finish a task with failing tests, formatting diffs, or clippy warnings.**

## Coding conventions

- Edition 2021. `unsafe` is `forbid`den in both crates — do not introduce it.
- Never silently ignore errors; return a typed `HippocoreError` variant.
- Pure functions (e.g. cosine similarity) must not panic on bad input — return
  `Option` or skip.
- Document public items with `///` doc comments.
- Keep dependencies minimal; discuss before adding new ones.
- Timestamps are epoch milliseconds (`i64`); the database manages `created_at`,
  `updated_at` and `version` — callers' values for these are overwritten.

## Architecture boundaries

- `config` / `errors` / `model` — config, typed errors, typed entities.
- `storage` — `Operation`/`State`, JSON-lines WAL + atomic snapshot, recovery,
  compaction. Knows operations and bytes, not scoring.
- `memory` — deterministic embedder + chunker/tokenizer; never panics.
- `index` — in-memory vector store + inverted text index; cosine + TF-IDF; no
  disk access; `cosine_similarity` returns `Option`, never panics.
- `query` — `Filter`, `SearchMode`, scoring/normalization, hybrid fusion. Tenant
  isolation is enforced here.
- `lib.rs` (`Hippocore`) — the only orchestrator of storage + index; public API.
- `cli` — clap parser + handlers; the `hippocore-cli` binary just calls
  `hippocore::cli::run()`.

On-disk files in `<data_dir>`: `wal.log` (JSON-lines), `snapshot.json` (atomic),
`meta.json`. Recovery loads the snapshot then replays the WAL; a torn trailing
line is skipped, not fatal. See `docs/en/ARCHITECTURE.md`.

## Do NOT implement in this MVP

Clustering, Raft/consensus, production auth, cloud embedders, GPU indexing,
HNSW/ANN, web dashboard, multi-node replication, SQL/query language, external
database dependency, `unsafe` Rust. See `docs/en/MVP_SCOPE.md` and `ROADMAP.md`.

If a request implies one of these, confirm scope first and prefer an issue over
a large speculative PR. **No task is complete until `docs/en/STATUS.md`,
`docs/pt-br/STATUS.md`, `docs/en/NEXT_FEATURE.md` and
`docs/pt-br/NEXT_FEATURE.md` are updated (and `CHANGELOG.md` when relevant).**
