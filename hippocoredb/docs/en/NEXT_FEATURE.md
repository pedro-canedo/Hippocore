# Next Feature

## Feature name

**eval-quality CLI command** — let operators run retrieval fixture evaluations
against any deployed database and get a quality report (hit@k, MRR).

## Why it matters

Hippocore DB now has a retrieval quality fixture format
(`tests/fixtures/retrieval_quality_v01.json`) that is used internally by the
test suite. But developers and operators have no way to run the same evaluation
against a live database from the CLI — they must either write code or rely on
the internal test suite, which operates on a freshly seeded temporary database.

An `eval-quality` command closes this gap: it reads a fixture file, seeds the
memories from the fixture into a temporary in-memory database (or an existing
one), runs each query, and reports per-scenario and aggregate metrics (hit@1,
hit@k, MRR). It exits non-zero if any configured threshold is not met, making
it useful in CI pipelines.

## Expected behavior

```
hippocore eval-quality --fixture path/to/fixture.json [--json] [--db <path>]
```

- `--fixture <file>`: path to a fixture JSON file (same format as the internal
  fixture).
- `--db <path>` (optional): if given, seeds memories into an existing database
  and evaluates against it. If omitted, seeds into a fresh temp directory and
  cleans up after evaluation.
- Outputs per-scenario: hit@1, hit@k, MRR, and whether forbidden_first_ids are
  absent from top results.
- Outputs aggregate: mean hit@1, mean hit@k, mean MRR across all scenarios.
- `--json` emits a machine-readable JSON report.
- Exits 0 if all thresholds pass; exits 1 with a human-readable error if any
  threshold fails.

## Affected modules / files

- `crates/hippocore/src/cli.rs` — add `EvalQuality` command and handler.
- `crates/hippocore/src/lib.rs` — add `eval_quality(fixture)` API if useful.
- `crates/hippocore-cli/tests/cli.rs` — subprocess smoke test.
- `docs/en/STATUS.md` and `docs/pt-br/STATUS.md`.
- `CHANGELOG.md`.

## Acceptance criteria

- `eval-quality --fixture <file>` runs and reports hit@1, hit@k, MRR per
  scenario and aggregate.
- `--json` emits a parseable JSON report.
- Exits non-zero when a threshold is not met.
- No new external dependencies.
- All 69+ tests pass.
- `cargo fmt`, `cargo clippy -D warnings` clean.

## Non-goals

- Persistent fixture storage in the database.
- A web dashboard or visual report.
- Automated fixture generation.
- Server mode.

## Follow-up

After eval-quality, the next step is Temporal Truth Layer v0.1:
`valid_from`/`valid_until` on memories and documents, allowing "what was true
at time T?" queries (Phase 7 of the roadmap).
