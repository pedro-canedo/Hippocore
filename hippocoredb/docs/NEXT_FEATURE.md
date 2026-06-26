# Next Feature

## Feature name

**Retrieval evaluation harness v0.1** — deterministic quality checks for common
RAG query scenarios.

## Why it matters

The RAG Quality Layer v0.1 improves obvious Oracle/PostgreSQL confusion with
normalization, entity tags, and small explainable score adjustments. The next
highest-value step is to keep those improvements from regressing as BM25,
hybrid fusion, seed data, and future ranking heuristics evolve.

## Expected behavior

- A small fixture file defines query scenarios, expected top technologies, and
  disallowed first results.
- A test or example runner ingests the fixture into an isolated temporary
  database and reports top-k ids, scores, tags, and reasons.
- The harness covers at least:
  - Oracle listener questions.
  - PostgreSQL service/start/status questions.
  - Python + PostgreSQL connection questions.
  - Mixed Oracle + PostgreSQL questions where both should be allowed.
- The output is human-readable enough to compare before/after behavior, but the
  assertions stay deterministic and do not require Ollama or network access.

## Affected modules / files

- `crates/hippocore/tests/` — deterministic regression tests or shared fixtures.
- `examples/ts-ollama-rag/` — optional fixture reuse for the demo seed memories.
- `docs/STATUS.md`, `CHANGELOG.md` — update on completion.

## Acceptance criteria

- `cargo test --workspace` includes the retrieval quality fixture checks.
- The fixture catches an Oracle memory ranking first for a Python/PostgreSQL
  connection query unless Oracle is explicitly mentioned.
- Test output or failure messages include enough score/reason detail to debug a
  ranking regression.
- No networked model is required for the harness.

## Risks / open questions

- Keep the harness small enough that it tests ranking behavior without freezing
  every exact score.
- Avoid duplicating the TypeScript demo seed data in ways that drift silently;
  either share concepts clearly or keep the Rust fixture intentionally minimal.
