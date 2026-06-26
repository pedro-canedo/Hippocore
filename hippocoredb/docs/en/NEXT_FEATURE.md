# Next Feature

## Feature name

**Contradiction Advisory Tests v0.1** — deterministic tests covering the full
behavior of the `contradicts` / `contradictions` advisory system.

## Why it matters

The `contradicts` field on a memory lets callers flag that two memories conflict.
When present, `build_context` applies confidence-aware re-ranking to prefer the
higher-confidence item. There is no focused regression suite that verifies:

- The `contradictions` advisory appears in `RecallResult` for flagged pairs.
- Both items are still returned (advisory does NOT suppress items).
- Confidence-aware re-ranking fires when contradictions are present.
- The higher-confidence item ranks above the lower-confidence one.

## Behaviour

No new production code. The test suite will cover:

1. Store memory A (confidence 0.3) and B (confidence 0.9) where B contradicts A.
2. Recall both; assert A's result carries `contradictions = [id_b]` and vice versa.
3. Neither A nor B is absent from recall results (advisory only, not a filter).
4. Call `build_context`; assert B appears before A (higher confidence wins).
5. Call `build_context` with `confidence = None` on both; assert ordering is
   unchanged from plain recall (confidence re-ranking only fires when needed).

## Files

- `crates/hippocore/tests/contradiction.rs` — new dedicated test file.
- `docs/en/CONTRADICTION_ADVISORY.md` and `docs/pt-br/CONTRADICTION_ADVISORY.md`.
- `docs/en/STATUS.md` and `docs/pt-br/STATUS.md`.

## Acceptance criteria

- At least 4 deterministic integration tests.
- All quality gates pass:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Out of scope

- Automatic contradiction detection (NLI, semantic comparison).
- Production code changes to the contradiction model.
- Server mode.
