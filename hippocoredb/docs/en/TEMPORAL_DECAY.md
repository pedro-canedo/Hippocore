# Temporal Decay v0.1

## Overview

Temporal Decay is an optional re-ranking pass in `build_context` that applies a
recency bias to recall scores. Recently created or updated items receive a higher
decay score than older ones, so a stale-but-semantically-similar memory does not
displace a fresher, slightly less similar one.

The feature is disabled by default (`temporal_weight = 0.0`) and adds no overhead
when disabled.

## How it works

After the hybrid recall pass (and after confidence-aware re-ranking when
contradictions are present), the temporal decay pass runs over the candidate set:

1. For each candidate, look up its effective timestamp:
   - **Memory** → `created_at` (memories are immutable; use creation time).
   - **Document chunk** → parent document's `updated_at`.
   - **Record** → `updated_at`.
2. Compute `age_days = (now_ms - timestamp_ms) / 86_400_000`.
3. Compute an exponential decay score:
   ```
   decay = exp(-age_days / temporal_decay_days)
   ```
   This starts near `1.0` for a brand-new item and falls toward `0.0` for very
   old items, with `temporal_decay_days` controlling the half-life. Items whose
   timestamp cannot be resolved receive a neutral decay of `0.5`.
4. Blend into the effective score:
   ```
   effective_score = recall_score × (1 − w) + decay × w
   ```
   where `w = config.temporal_weight`.
5. Re-sort by effective score descending.

When `w = 0` (default) the formula reduces to `recall_score` and ordering is
unchanged.

## Configuration

```rust
use hippocore::Config;

let mut cfg = Config::new("./mydb");
// Enable temporal decay with a moderate bias (30-day half-life):
cfg.temporal_weight = 0.2;
cfg.temporal_decay_days = 30.0;

let db = Hippocore::open(cfg)?;
```

| Field | Type | Default | Description |
|---|---|---|---|
| `temporal_weight` | `f32` | `0.0` | Blend weight in `[0.0, 1.0]` for recency score |
| `temporal_decay_days` | `f32` | `30.0` | Exponential half-life in days; must be `> 0` when weight `> 0` |

Passing `temporal_weight` outside `[0.0, 1.0]` or `temporal_decay_days ≤ 0`
when `temporal_weight > 0` causes `build_context` to return a
`HippocoreError::Validation` error before any I/O is performed.

## Invariants

- `w = 0.0` — identical ordering to plain hybrid recall.
- Items with the same age receive the same decay score.
- Items whose timestamp cannot be resolved (edge case) receive a neutral decay
  of `0.5`, so they are not silently dropped.
- The pass runs on all item kinds; chunked documents use the parent document's
  timestamp to avoid penalising chunks of recently-updated documents.
- Temporal decay is applied before graph-aware ranking, so both features compose.

## Non-goals

- Periodic background reindexing based on age.
- Automatic eviction of stale memories.
- Multi-hop graph traversal.
- Server mode.
