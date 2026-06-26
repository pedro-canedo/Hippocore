# Next Feature

## Feature name

**Audit Retention v0.1** — bounded local retention for `audit.log`.

## Why it matters

The RAG Audit Engine records every `build_context` call, and Graph-Aware Context
adds richer audit entries. Without a retention policy, `audit.log` can grow
without bound in long-running embedded applications. The next high-value
increment is to keep auditability local and deterministic while giving users a
simple way to bound disk usage.

Audit Retention v0.1 should add:

- Config options for audit retention by max records and/or max bytes.
- A `compact_audit()` or `rotate_audit()` API that rewrites `audit.log`
  atomically while preserving the newest records.
- Automatic retention enforcement after audit append when configured.
- CLI command `compact-audit --db <path> [--json]`.
- Status/stats visibility for audit log bytes and retained record count.

## Acceptance criteria

- Default behavior remains unchanged: no audit records are removed unless a
  retention policy is configured or `compact-audit` is called.
- Retention keeps the newest audit records and preserves valid JSON-lines.
- Rewriting is atomic: temp file, fsync, rename.
- `query_audit` continues to work after retention.
- CLI `compact-audit --json` emits parseable summary output.
- At least 4 deterministic tests: default unchanged, max-record retention,
  max-byte retention or manual compaction, query after compaction.
- Documentation is updated in English and Portuguese.
- Quality gate passes:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Non-goals

- Remote audit export.
- Compression.
- Encryption.
- Server/HTTP mode.
- Cross-file audit search.

## Follow-up

After Audit Retention v0.1, evaluate graph-aware recall ranking versus broader
admin/studio controls for graph and audit data.
