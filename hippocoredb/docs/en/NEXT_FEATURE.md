# Next Feature

## Feature name

**Graph-Aware Context v0.1** — optional direct-neighbour expansion for context
assembly.

## Why it matters

Graph Memory v0.1 stores durable direct relationships and exposes neighbour ids
in context/audit provenance, but it does not yet use those relationships to
improve the assembled context. The next high-value increment is to let callers
opt into including directly related graph neighbours when building context,
without implementing full GraphRAG traversal or changing default recall ranking.

Graph-Aware Context v0.1 should add:

- A `include_related: bool` option on `BuildContextRequest` (default `false`).
- A small `related_limit: usize` option to cap direct-neighbour expansion.
- Context assembly that can include directly related neighbours of recalled
  items when they fit the token budget.
- Provenance that marks whether a context item was included by recall or by
  graph expansion.
- Audit records that capture graph-expanded items distinctly from recalled
  candidates.
- CLI flags on `build-context`: `--include-related` and `--related-limit <n>`.

## Acceptance criteria

- Default `build_context` behavior is unchanged when `include_related = false`.
- When enabled, only direct neighbours of recalled items are considered.
- Expansion remains tenant-isolated and token-budget-bound.
- Expanded items are not duplicated if already recalled.
- Context/audit output clearly distinguishes recalled vs graph-expanded items.
- At least 4 deterministic tests: default unchanged, includes a direct neighbour,
  respects token budget/limit, tenant isolation/no duplication.
- Documentation is updated in English and Portuguese.
- Quality gate passes:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Non-goals

- Multi-hop traversal.
- Graph-aware ranking or score boosting.
- Entity extraction.
- Graph query language.
- Server/HTTP mode.

## Follow-up

After Graph-Aware Context v0.1, evaluate whether graph-aware recall ranking is
worth adding or whether audit retention/rotation is more valuable.
