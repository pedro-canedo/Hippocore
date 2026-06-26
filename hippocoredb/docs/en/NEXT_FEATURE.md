# Next Feature

## Feature name

**Recall Min-Score Filter v0.1**

## Why it matters

After confidence-weighted recall, the score range across candidates can vary
widely. An LLM context assembled from a `top_k=10` recall may include several
items with scores near zero — noise that wastes context tokens and can confuse
the model. Allowing the caller to set a minimum score threshold lets
applications trade recall coverage for precision without changing `top_k`.

## Behavior

- Add `min_score: Option<f32>` to `RecallRequest` (default `None`).
- After confidence weighting and final sort, filter out results where
  `score < min_score`. Truncation to `top_k` happens after filtering so
  `top_k` remains the upper bound on result count.
- Items at exactly `min_score` are included (inclusive lower bound).
- Add `--min-score <f32>` flag to the `hippocore recall` CLI command.
- HTTP server admin recall endpoint (`POST /admin/tenants/:tid/recall`) and
  the service recall endpoint (`POST /tenants/:tid/recall`) already accept
  an arbitrary JSON body — add `min_score` to their request types.

## Likely files

- `crates/hippocore/src/lib.rs` (RecallRequest)
- `crates/hippocore/src/query.rs`
- `crates/hippocore/src/cli.rs` (recall subcommand)
- `crates/hippocore-server/src/handlers/recall.rs`
- `crates/hippocore/tests/retrieval_quality.rs`
- `docs/en/STATUS.md`
- `docs/pt-br/STATUS.md`
- `CHANGELOG.md`

## Acceptance criteria

- `RecallRequest` has `min_score: Option<f32>` defaulting to `None`.
- When `min_score = Some(0.5)`, no result with `score < 0.5` is returned.
- When `min_score = None`, existing behaviour is unchanged.
- `hippocore recall --min-score 0.3 ...` passes the threshold to the core.
- HTTP recall body accepts `{"min_score": 0.5}` without breaking existing
  callers that omit the field.
- Tests cover: threshold removes low-score items, `None` returns all, items
  exactly at threshold are included.
- `cargo fmt --all --check`, `cargo test --workspace`, and
  `cargo clippy --workspace --all-targets -- -D warnings` pass.

## Out of scope

- Per-mode thresholds (different min for vector vs text).
- Dynamic threshold calibration.
- Exposing `min_score` in `BuildContextRequest`.
- Any change to storage or index.
