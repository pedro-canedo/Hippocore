# Compact + Reopen Invariant Tests v0.1

## Overview

A dedicated test suite verifying that `compact()` followed by a cold reopen
produces exactly the same observable state. No production code was changed;
this feature is purely additive tests.

## What compact does

`compact()` folds all WAL entries into an atomic snapshot (`snapshot.json`) and
truncates `wal.log` to zero entries. The next open loads from the snapshot
instead of replaying the full WAL history.

## Test file

`crates/hippocore/tests/compact_reopen.rs`

## What is tested

| Test | Scenario |
|---|---|
| `wal_length_is_zero_after_compact` | `stats().wal_entries == 0` immediately after `compact()` |
| `collections_survive_compact_reopen` | All collections present before compact are present after cold reopen |
| `memories_survive_compact_reopen` | Memory retrievable by id and appears in `recall()` after compact + reopen |
| `documents_survive_compact_reopen` | Document retrievable by id after compact + reopen |
| `multiple_compact_cycles_preserve_state` | Three successive compact cycles do not lose any of the inserted memories |

## Non-goals

- WAL compaction strategy changes.
- Server mode.
- Production code changes.
