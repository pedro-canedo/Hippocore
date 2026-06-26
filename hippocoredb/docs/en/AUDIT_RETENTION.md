# Audit Retention v0.1

## Overview

The RAG Audit Engine records every `build_context` call to `audit.log` as an
append-only JSON-lines file. Without a retention policy the log grows without
bound in long-running embedded applications.

Audit Retention v0.1 provides:

- **Config-level limits** (`audit_max_records`, `audit_max_bytes`).
- **`compact_audit(max_records, max_bytes)`** — atomically rewrites `audit.log`,
  keeping only the newest records.
- **Automatic enforcement** — retention runs after every `build_context` append
  when a limit is configured.
- **CLI command** — `compact-audit`.
- **Stats visibility** — `DatabaseStats` now exposes `audit_log_bytes` and
  `audit_records`.

## Configuration

```rust
use hippocore::Config;

let mut cfg = Config::new("./data");
cfg.audit_max_records = 1000;    // keep at most 1000 audit records
cfg.audit_max_bytes   = 10_000_000; // keep at most ~10 MB
```

Both fields default to `0` (unlimited). When both are non-zero the more
restrictive constraint wins (fewest records kept).

## Library API

```rust
// Manual compaction — uses config values when Some(…) is None, or disables
// a constraint for this call with Some(0).
let summary = db.compact_audit(Some(100), None)?;
println!(
    "kept {} records, removed {}, {} → {} bytes",
    summary.records_kept,
    summary.records_removed,
    summary.bytes_before,
    summary.bytes_after,
);

// Stats include audit log info.
let stats = db.stats()?;
println!("audit records: {}, bytes: {}", stats.audit_records, stats.audit_log_bytes);
```

## CLI

```bash
# Compact keeping at most 200 records (human-readable output).
hippocore compact-audit --db ./data --max-records 200

# Compact keeping at most 5 MB, emit JSON.
hippocore compact-audit --db ./data --max-bytes 5242880 --json
```

JSON output:

```json
{
  "records_kept": 45,
  "records_removed": 155,
  "bytes_before": 204800,
  "bytes_after": 18432
}
```

## Invariants

- Default behavior is unchanged when no config limits are set and
  `compact_audit` is not called.
- Only the newest records are retained; ordering is always preserved.
- Rewrite is atomic: `audit.tmp` is written and fsynced, then renamed over
  `audit.log`.
- `query_audit` continues to work after compaction.
- Auto-compaction on append is best-effort: the `build_context` call succeeds
  even if compaction fails.

## Non-goals (this version)

- Remote audit export.
- Log compression or encryption.
- Cross-file audit search.
- Server/HTTP mode.
