# Contributing to Hippocore DB

Thanks for your interest in Hippocore DB! This project is in its early MVP
stage, so contributions that improve correctness, tests, documentation and the
foundations described in [ROADMAP.md](./ROADMAP.md) are especially welcome.

**Before contributing, read the project's golden rules: [`regras.md`](./regras.md).**
They are binding — they protect the AI-native vision, the high-performance
invariants, persistence safety, and the open-source/privacy bar.

## Getting started

```bash
git clone <your fork>
cd hippocoredb
cargo build --workspace
cargo test --workspace
```

You need a recent stable Rust toolchain (see `rust-version` in the workspace
`Cargo.toml`). Install via [rustup](https://rustup.rs).

## Quality bar (run before every PR)

```bash
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

All three must pass. If clippy flags something, **fix the code** rather than
adding `#[allow(...)]`, unless there is a clearly documented reason.

## Project layout

```
crates/hippocore/        core library
  src/config|errors|model|storage|index|memory|query|cli.rs + lib.rs (engine)
crates/hippocore-cli/    the `hippocore` binary (thin wrapper over cli::run)
examples/                runnable examples
benches/                 criterion benchmarks
docs/                    ARCHITECTURE, MVP_SCOPE, STATUS, NEXT_FEATURE, DECISIONS
```

## Architecture boundaries

- **storage** — operations + JSON-lines WAL + atomic snapshot; no scoring.
- **index** — in-memory vector store + inverted text index; no disk access.
- **memory** — embedder + chunker; pure, never panics.
- **query** — filters, scoring, hybrid fusion; enforces tenant isolation.
- **`Hippocore`** (`lib.rs`) — the only orchestrator of storage + index.

See `docs/ARCHITECTURE.md` for the full picture.

## Scope discipline

This is an MVP. Please do **not** add HNSW/ANN, a real BM25, an HTTP/gRPC server,
clustering, consensus, a SQL/query language, or cloud embedders in a single large
PR — see `docs/MVP_SCOPE.md`. Those belong to later phases; discuss in an issue
first.

Also: keep dependencies lean, avoid `unsafe` (it is `forbid`den in both crates),
and never silently swallow errors.

## Living documentation

Documentation is part of the product. **No change is complete until
`docs/STATUS.md` and `docs/NEXT_FEATURE.md` are updated**, and `CHANGELOG.md`
when the change is user-facing or architectural.

## Tests

- Add or update tests for any behavioural change.
- Prefer integration tests in `crates/hippocore/tests/` for end-to-end behaviour
  and inline `#[cfg(test)]` modules for unit-level logic.

## Commit / PR etiquette

- Keep PRs focused and reviewable.
- Describe the *why*, not just the *what*.
- Make sure CI-equivalent commands (above) pass locally.
