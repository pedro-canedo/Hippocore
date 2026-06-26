# WAL Recovery Tests v0.1

## Overview

A dedicated test suite verifying that `Storage::replay_wal` handles torn
writes, truncated records, and checksum-mismatched entries without panicking
or silently loading corrupt data. No production code was changed; this feature
is purely additive tests.

## WAL format

Each line in `wal.log` is: `<crc32-hex>\t<json>\n`

On database open, replay stops at the first line that is:
- Truncated (incomplete / no newline) — `BufReader::lines` returns an `Err`
- Unparseable JSON (returns `Ok(None)`)
- Checksum mismatch (returns `Err(Corruption(...))`)

All operations applied before the bad line are kept.

## Test file

`crates/hippocore/tests/wal_recovery.rs`

## What is tested

| Test | Scenario |
|---|---|
| `clean_wal_opens_successfully` | Normal cold reopen — state persists |
| `truncated_wal_trailing_line_skipped_on_open` | Half-written entry (no newline) appended; open succeeds; prior data intact |
| `checksum_mismatch_stops_replay_does_not_panic` | Line with wrong CRC; open does not panic; bad entry not applied |
| `state_before_torn_entry_is_intact` | Memory written before tear survives in both `get_memory` and `recall` |

## Non-goals

- WAL compaction changes.
- Server mode.
- Production code changes.
