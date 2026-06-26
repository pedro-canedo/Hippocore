# Next Feature

## Feature name

**Supersession Lifecycle Tests v0.1** — deterministic tests covering the full
lifecycle of memory supersession.

## Why it matters

The `supersedes` / `superseded_by` linkage allows a new memory to replace an
older one. This is the primary mechanism for updating stale facts without
deleting history. While the production code handles it, there is no focused
regression suite that verifies:

- Superseded memories are excluded from default recall.
- Superseded memories are visible when `include_superseded = true`.
- The linkage is bidirectional and symmetric.
- The relationship survives WAL compaction.

## Behaviour

No new production code. The test suite will cover:

1. Store memory A; store memory B that supersedes A.
2. Assert: recall without `include_superseded` returns B but not A.
3. Assert: recall with `include_superseded = true` returns both A and B.
4. Assert: A's `superseded_by` is B's id; B's `supersedes` contains A's id.
5. Compact the WAL; re-open the database; repeat assertions 2–4.
6. Delete B; assert: A is no longer superseded (since the superseding memory
   is gone) — or that the delete is rejected if referential integrity is enforced.

## Files

- `crates/hippocore/tests/supersession.rs` — new dedicated test file.
- `docs/en/SUPERSESSION_LIFECYCLE.md` and `docs/pt-br/SUPERSESSION_LIFECYCLE.md`.
- `docs/en/STATUS.md` and `docs/pt-br/STATUS.md`.

## Acceptance criteria

- At least 5 deterministic integration tests.
- All quality gates pass:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Out of scope

- Chains of supersession (A → B → C).
- Server mode.
- Production code changes to the supersession model.
