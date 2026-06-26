# Valid-Window Recall Tests v0.1

## Overview

A dedicated test suite verifying that temporal validity filtering works
correctly for memories with `valid_from` / `valid_until` timestamps. Expired or
not-yet-valid memories must be excluded from recall at the current time; backdated
`as_of` queries must surface memories that were valid at that past moment.

No production code was changed; this feature is purely additive tests.

## Test file

`crates/hippocore/tests/valid_window.rs`

## What is tested

| Test | Scenario |
|---|---|
| `expired_memory_excluded_from_recall` | `valid_until` in the past → excluded |
| `future_valid_memory_included_in_recall` | `valid_until` in the future → included |
| `not_yet_valid_memory_excluded_from_recall` | `valid_from` in the future → excluded |
| `as_of_backdating_retrieves_past_valid_memory` | `as_of` inside validity window → included; at current time → excluded |
| `memory_with_no_validity_window_always_included` | No `valid_from`/`valid_until` → always included |

## Timestamps used

- `FAR_PAST = 1_000_000_000_000` ms (2001-09-08) — safely expired.
- `FAR_FUTURE = 9_000_000_000_000` ms (~year 2255) — safely not expired.

Both constants avoid relying on `std::time::SystemTime::now()` so the tests
remain deterministic regardless of when they are run.

## Non-goals

- Automatic expiry / eviction of expired memories.
- Production code changes.
- Server mode.
