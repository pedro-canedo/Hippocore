# Next Feature

## Feature name

**Valid-Window Recall Tests v0.1** — deterministic tests for `valid_from` /
`valid_until` temporal validity filtering.

## Why it matters

Memories and documents support `valid_from` / `valid_until` timestamps so that
time-bounded facts (e.g. promotional offers, seasonal configuration) are
automatically excluded from recall when they expire. The filter exists but there
is no focused regression suite. A bug here could surface expired data to LLM
prompts or hide currently valid data.

## Behaviour

No new production code. The test suite will cover:

1. Memory with `valid_until` in the past → excluded from recall.
2. Memory with `valid_until` in the future → included.
3. Memory with `valid_from` in the future → excluded until that time.
4. Backdated query via `as_of` → retrieves state at a past timestamp.
5. Boundary edge case: `valid_until == query_time` → excluded (strictly less than).
6. Memory with no validity window → always included.

## Files

- `crates/hippocore/tests/valid_window.rs` — new dedicated test file.
- `docs/en/VALID_WINDOW_RECALL.md` and `docs/pt-br/VALID_WINDOW_RECALL.md`.
- `docs/en/STATUS.md` and `docs/pt-br/STATUS.md`.

## Acceptance criteria

- At least 5 deterministic integration tests using explicit epoch-ms timestamps.
- All quality gates pass:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Out of scope

- Automatic expiry / eviction.
- Production code changes.
- Server mode.
