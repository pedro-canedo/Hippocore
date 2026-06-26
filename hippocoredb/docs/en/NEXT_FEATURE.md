# Next Feature

## Feature name

**Context Compiler** — `build_context(query, user, max_tokens)`

## Why it matters

After storing memories, documents, and records, the most common AI agent use
case is assembling a prompt-ready context string: take the top-k recall results
across all item types, rank them, trim to a token budget, and format them so an
LLM can reason over them in one pass. Today every application has to wire this
up manually — choosing which items to include, how many tokens each takes, and
in what order they should appear.

A built-in Context Compiler removes that boilerplate and provides:
- A token-budget-aware selector that fills the budget greedily by score.
- A single API call that replaces the recall + format loop in most agents.
- A `ContextBlock` value type that carries the assembled string plus provenance
  metadata (which items were included, how many tokens, which were dropped).

## Expected behavior

### New API

```rust
pub struct BuildContextRequest {
    pub tenant_id: String,
    pub query: String,
    pub user_id: Option<String>,
    pub max_tokens: usize,           // hard ceiling; default 2048
    pub top_k_candidates: usize,     // recall up to this many items; default 20
    pub mode: SearchMode,            // default Hybrid
    pub collection: Option<String>,
    pub metadata_filter: Metadata,
}

pub struct ContextBlock {
    pub text: String,                // assembled, LLM-ready context
    pub token_count: usize,
    pub items_included: Vec<ContextItem>,
    pub items_dropped: usize,
}

pub struct ContextItem {
    pub id: String,
    pub kind: ItemKind,
    pub score: f32,
    pub token_count: usize,
    pub snippet: String,             // first 120 chars of content
}
```

### Algorithm

1. Run recall (hybrid by default) for up to `top_k_candidates` items.
2. Sort by score descending.
3. Greedily add items (highest-score first) until `max_tokens` would be exceeded.
4. Format: each included item becomes `[<kind>:<id>] <text>` separated by `\n\n`.
5. Return `ContextBlock` with assembled text, item provenance, and drop count.

### Token counting

Built-in approximation: 1 token ≈ 4 bytes of UTF-8. Callers that need exact
tokenization can post-process; the built-in approximation is good enough for
budget enforcement without a tokenizer dependency.

### CLI

```
hippocore build-context --tenant <t> --query "..." [--max-tokens 2048]
  [--top-k 20] [--mode hybrid] [--collection c] [--json]
```

`--json` emits `ContextBlock` as JSON. Default output prints the assembled text
followed by a provenance footer.

## Acceptance criteria

- `Hippocore::build_context(req)` returns a `ContextBlock`.
- Token budget is respected (assembled text never exceeds `max_tokens * 4` bytes
  by more than the last included item's length, i.e. the budget is applied
  greedily, not truncated mid-item).
- Items are ranked by recall score (highest first).
- `ContextBlock.items_included` lists every included item with id, kind, score,
  token count, and snippet.
- `ContextBlock.items_dropped` counts items that were fetched but could not fit.
- `build-context` CLI subcommand works; `--json` output is machine-parseable.
- At least 3 integration tests: budget enforcement, multi-item assembly, JSON
  output round-trip.
- All 82+ tests still pass; no new external dependencies.

## Non-goals

- External tokenizer integration (tiktoken, etc.) — use byte approximation.
- Prompt templating or few-shot injection.
- Server/HTTP mode.

## Follow-up

Phase 8 done → Phase 9: **RAG Audit Engine** — provenance tracking, per-item
confidence scores, and a query-time audit log.
