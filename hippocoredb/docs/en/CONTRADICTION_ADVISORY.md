# Contradiction Advisory Tests v0.1

## Overview

A dedicated test suite covering the full behavior of the contradiction advisory
system: both contradicting memories appear in recall, the advisory is populated
in `RecallResult.contradictions`, and `build_context` applies confidence-aware
re-ranking when contradictions are present.

No production code was changed; this feature is purely additive tests.

## Test file

`crates/hippocore/tests/contradiction.rs`

## What is tested

| Test | Scenario |
|---|---|
| `contradicting_memories_both_appear_in_recall` | Both A and B are returned; contradiction does NOT suppress either item |
| `contradictions_advisory_populated_in_recall_result` | B's `RecallResult.contradictions` contains A's id |
| `build_context_confidence_reranking_prefers_higher_confidence` | B (confidence 0.95) ranks above A (confidence 0.1) when contradictions are present |
| `build_context_no_contradiction_no_reranking` | No contradictions → no confidence re-ranking; all items appear in context |

## Design invariants confirmed

- The `contradicts` annotation is purely advisory: both items are returned by
  `recall()` regardless of confidence.
- `build_context` fires confidence-aware re-ranking only when at least one
  candidate in the set carries a non-empty `contradictions` list.
- When no contradictions are present, the extra re-ranking pass is skipped and
  recall ordering is preserved.
