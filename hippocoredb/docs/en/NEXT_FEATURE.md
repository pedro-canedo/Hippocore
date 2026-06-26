# Next Feature

## Feature name

**Temporal Truth Layer v0.1** — add `valid_from` / `valid_until` fields to
memories and documents, and support "what was true at time T?" queries.

## Why it matters

AI agents frequently deal with facts that change over time. Without temporal
metadata, Hippocore can only answer "what is currently stored?" — not "what was
known on a given date?" This means outdated facts silently pollute retrieval
results, and there is no way to audit what the agent believed at a specific
moment.

`valid_from` / `valid_until` allow callers to:
- Mark facts as current (open-ended `valid_until = None`) or expired
  (`valid_until = Some(epoch_ms)`).
- Query memories and documents as of a specific point in time via `as_of`.
- Supersede an old fact cleanly by writing a new one with an updated
  `valid_from`, without deleting the old one (so the old state is auditable).

## Expected behavior

### Model changes

`Memory` and `Document` gain two optional fields:
```
valid_from:  Option<i64>   // epoch ms; None means "always valid from creation"
valid_until: Option<i64>   // epoch ms; None means "still valid"
```

### API changes

`RememberRequest` and `StoreDocumentRequest` gain:
```
valid_from:  Option<i64>
valid_until: Option<i64>
```

`RecallRequest` / `SearchRequest` gain:
```
as_of: Option<i64>  // epoch ms; None = now
```

When `as_of` is set, retrieval filters out any entry where:
- `valid_from > as_of` (not yet valid), or
- `valid_until <= as_of` (already expired).

### CLI changes

- `remember` gains `--valid-from <ms>` and `--valid-until <ms>` flags.
- `recall` gains `--as-of <ms>`.
- `put-document` gains `--valid-from <ms>` and `--valid-until <ms>`.

### Persistence

`valid_from` and `valid_until` are stored in the WAL/snapshot as part of the
existing JSON model; legacy entries without these fields are treated as
`valid_from = created_at`, `valid_until = None` (always valid).

## Acceptance criteria

- `valid_from` / `valid_until` stored and recovered through WAL/snapshot.
- `recall --as-of <ms>` filters entries correctly (past-valid, current, and
  future-valid scenarios).
- Expired entries do not appear in default recall (default `as_of = now`).
- Legacy entries without temporal fields recover correctly with always-valid
  semantics.
- CLI flags work (`--valid-from`, `--valid-until`, `--as-of`).
- New unit + integration tests covering: default no-filter, as-of-past,
  as-of-future, expired entries, legacy-entry recovery.
- All 71+ tests still pass.
- `cargo fmt`, `cargo clippy -D warnings` clean.

## Non-goals

- `supersedes` / `contradicts` relations (Phase 7 full spec — deferred).
- Conflict resolution between overlapping temporal facts.
- Index optimization for time-range queries (brute force acceptable at this
  scale).
- Server mode.

## Follow-up

After Temporal Truth Layer v0.1, the next step could be:
- Filling in the remaining Phase 5 item: **benchmark regression guard** to
  prevent silent performance regressions in recall latency.
- Or Phase 6 admin surface items: local Studio for browsing and editing context
  data (deferred until Phase 6 is scheduled).
