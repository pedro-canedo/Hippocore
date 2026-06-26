# Next Feature

## Feature name

**build_context Smoke Tests v0.1** — deterministic tests verifying that
`build_context` assembles a usable LLM context block.

## Why it matters

`build_context` is the primary output path for AI consumers — it ranks, trims,
and serialises items into a prompt-ready string. There are no dedicated tests
verifying: (a) the context string is non-empty when relevant items exist,
(b) `max_tokens` is respected, (c) graph-expanded related items appear when
`include_related = true`. A regression in any of these paths silently degrades
AI outputs.

## Behaviour

No new production code. The test suite will cover:

1. Single memory → `build_context` produces a non-empty `text` string.
2. The context string contains a recognisable snippet from the memory.
3. `max_tokens` set low → fewer items included than available.
4. `include_related = true` with a graph edge → related item appears in
   `items_included`.
5. Empty store → `build_context` returns `text = ""` (not an error).

## Files

- `crates/hippocore/tests/build_context_smoke.rs` — new dedicated test file.
- `docs/en/BUILD_CONTEXT_SMOKE.md` and `docs/pt-br/BUILD_CONTEXT_SMOKE.md`.
- `docs/en/STATUS.md` and `docs/pt-br/STATUS.md`.

## Acceptance criteria

- At least 5 deterministic integration tests using `TempDir`.
- All quality gates pass:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Out of scope

- Custom context format changes.
- Server mode.
- Production code changes.
