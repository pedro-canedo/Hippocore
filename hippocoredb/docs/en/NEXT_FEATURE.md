# Next Feature

## Feature name

**Temporal Truth Layer full spec** — `supersedes` and `contradicts` relations
between memories, conflict detection on recall, and confidence-aware resolution.

## Why it matters

Phase 7 v0.1 added `valid_from`/`valid_until` and basic `as_of` filtering. The
full Temporal Truth Layer spec adds the semantic layer: AI agents frequently
store facts that contradict or update each other, and without explicit
`supersedes`/`contradicts` metadata Hippocore cannot surface or resolve those
conflicts. Without this, a recall result set may contain both an old belief and
a newer fact that contradicts it, leaving the agent to sort them out — or worse,
silently using stale information.

Adding explicit supersedure/contradiction relations allows:
- Marking a new memory as superseding an older one (explicit replacement).
- Flagging two memories as contradicting each other (human or agent review
  required).
- Filtering out superseded memories from default recall results.
- Returning contradiction warnings alongside recall results.

## Expected behavior

### Model changes

`Memory` gains two optional fields:
```
supersedes:   Vec<String>  // ids of memories this one replaces
contradicts:  Vec<String>  // ids of memories this one contradicts
```

### Write behavior

`remember` validates that any id listed in `supersedes`/`contradicts` exists
in the same tenant. If a memory is superseded, it is marked with
`superseded_by = <new_id>` (stored on the old memory) and excluded from default
recall (treated like an expired entry).

### Query behavior

- Superseded memories are excluded from default recall (unless `include_superseded
  = true` is set on the request).
- When a recalled memory has a non-empty `contradicts` list, the `RecallResult`
  carries a `contradictions: Vec<String>` advisory field.

### CLI changes

- `remember` gains `--supersedes <id>` (repeatable) and `--contradicts <id>`
  (repeatable).
- `recall` gains `--include-superseded` to surface the full history.

## Acceptance criteria

- `supersedes` and `contradicts` stored and recovered through WAL/snapshot.
- Superseded memories excluded from default recall; surfaced via
  `--include-superseded`.
- Contradiction ids appear in `RecallResult` when present.
- New integration tests covering: supersedure exclusion, contradiction
  surfacing, include-superseded override, validation of unknown supersede ids.
- All 76+ tests still pass.
- `cargo fmt`, `cargo clippy -D warnings` clean.
- No new external dependencies.

## Non-goals

- Automatic conflict resolution (human or agent loop, not engine responsibility).
- Provenance graphs or full knowledge graph traversal (Phase 10 — Graph Memory).
- Server mode.

## Follow-up

After the full Temporal Truth Layer, Phase 7 is complete. Phase 8 is the
**Context Compiler**: `build_context(query, user, max_tokens)` — token-budget-
aware assembly of the best context for an LLM from all stored item types.
