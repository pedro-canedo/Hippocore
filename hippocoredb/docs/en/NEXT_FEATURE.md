# Next Feature

## Feature name

**RAG Audit Engine** — per-item confidence scoring and a query-time audit log.

## Why it matters

Context Compiler (Phase 8) assembles LLM-ready context, but once an answer is
generated there is no built-in way to trace it back to the source items, score
their reliability, or record what was retrieved and when. In production RAG
systems this is a critical gap: support engineers need to know *why* the agent
said X, and reliability teams need to detect drift or low-quality retrieval
before it causes customer-facing incidents.

The RAG Audit Engine adds:
- A `confidence: Option<f32>` field on `Memory` and `RecallResult` so facts can
  carry an explicit reliability signal (e.g. from user feedback or review).
- An append-only audit log (`<data_dir>/audit.log`) where every `build_context`
  call is recorded: timestamp, query, tenant, items retrieved, scores, tokens.
- A `query_audit(from_ms, to_ms, tenant_id)` API to replay what was retrieved
  in a time window.
- A `rate_memory(id, confidence)` API for human-in-the-loop feedback.
- CLI: `audit --from <ms> --to <ms> --tenant <t> [--json]` and
  `rate-memory --id <id> --confidence <0.0-1.0>`.

## Acceptance criteria

- `Memory` and `RecallResult` gain `confidence: Option<f32>` with backward-
  compatible serde default (`None`).
- Every `build_context` call writes a JSON-lines audit record atomically.
- `query_audit(from_ms, to_ms, tenant_id)` returns `Vec<AuditRecord>`.
- `rate_memory` validates confidence is in `[0.0, 1.0]`, stores it durably.
- CLI `audit` and `rate-memory` commands work; `--json` output is parseable.
- At least 4 integration tests: audit record written on build_context, replay
  by time range, confidence rating round-trip, invalid confidence rejected.
- All 85+ tests pass; no new external dependencies.

## Non-goals

- Automatic confidence calibration (human/agent sets it; engine records it).
- Full provenance graph (Phase 10 — Graph Memory).
- Server/HTTP mode.

## Follow-up

Phase 9 done → Phase 10: **Graph Memory** — inter-item relationship edges and
graph-aware recall.
