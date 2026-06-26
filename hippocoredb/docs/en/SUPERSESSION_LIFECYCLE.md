# Supersession Lifecycle Tests v0.1

## Overview

A dedicated test suite covering the full lifecycle of memory supersession: a
new memory replaces an older one via the `supersedes` field, marking the old
memory as superseded so it is hidden from default recall but preserved for
history.

No production code was changed; this feature is purely additive tests.

## Test file

`crates/hippocore/tests/supersession.rs`

## What is tested

| Test | Scenario |
|---|---|
| `superseded_memory_hidden_from_default_recall` | Memory A superseded by B; A absent from recall, B present |
| `superseded_memory_visible_with_include_superseded` | `include_superseded = true` returns both A and B |
| `supersession_linkage_is_bidirectional` | `A.superseded_by = id_b` and `id_a ∈ B.supersedes` |
| `supersession_survives_wal_compaction` | WAL compaction + cold reopen; all invariants preserved |
| `forgetting_superseding_memory_does_not_restore_superseded` | Deleting B leaves A's `superseded_by` intact (durable tombstone) |

## Design invariants confirmed

- `superseded_by` is a **durable tombstone**: once set, it persists even if the
  superseding memory is later deleted. This preserves the history of the
  supersession event and avoids silently re-activating stale memories.
- The `superseded` flag in the retrieval index is derived from
  `superseded_by.is_some()`, so the filter applies correctly even after WAL
  compaction and recovery.
