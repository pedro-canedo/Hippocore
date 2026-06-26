# Graph-Aware Ranking v0.1

## Overview

Graph-Aware Ranking is an optional re-ranking pass in `build_context` that boosts
the effective score of recalled candidates whose direct graph neighbours also
appear in the same candidate set. It rewards context items that are densely
connected to other relevant memories, lifting genuinely related context above
coincidental lexical matches.

The feature is disabled by default (`graph_rank_weight = 0.0`) and adds no
overhead when disabled.

## How it works

After the hybrid recall pass produces a ranked candidate list (including any
graph-expanded items), and after confidence-aware re-ranking (when contradictions
are present), the graph-aware pass runs:

1. Build a set of all candidate IDs.
2. For each candidate `c`, count how many IDs in its `related_item_ids` appear
   in the candidate set. Call this `neighbours_in_set`.
3. Normalise: `connectivity_score = neighbours_in_set / (total_candidates - 1)`,
   clamped to `[0.0, 1.0]`.
4. Blend into the effective score:

   ```
   effective_score = recall_score × (1 − w) + connectivity_score × w
   ```

   where `w = config.graph_rank_weight`.

5. Re-sort by `effective_score` descending.

Items with no graph edges produce `connectivity_score = 0`, so their effective
score is `recall_score × (1 − w)`. When `w = 0` all items scale identically
and ordering is unchanged.

## Configuration

```rust
use hippocore::Config;

let mut cfg = Config::new("./mydb");
// Enable graph-aware ranking with a light boost:
cfg.graph_rank_weight = 0.1;

let db = Hippocore::open(cfg)?;
```

| Field | Type | Default | Description |
|---|---|---|---|
| `graph_rank_weight` | `f32` | `0.0` | Blend weight in `[0.0, 1.0]` for connectivity bonus |

Passing a value outside `[0.0, 1.0]` causes `build_context` to return a
`HippocoreError::Validation` error before any I/O is performed.

## Invariants

- `w = 0.0` — identical ordering to plain hybrid recall.
- Items with zero neighbours in the candidate set receive no boost.
- Normalisation is relative to the current candidate set size; absolute edge
  count is not compared across queries.
- The pass runs only when the candidate set has at least 2 items (a single item
  has no peers to be connected to).
- No graph traversal — only direct (1-hop) neighbours are considered.

## Non-goals

- Multi-hop or transitive connectivity.
- Graph clustering or community detection.
- Learning-to-rank or ML-based ranking.
- Server mode.
