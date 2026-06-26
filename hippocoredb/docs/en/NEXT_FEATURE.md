# Next Feature

## Feature name

**Multi-tenant Query Isolation Audit v0.1** — deterministic test suite proving
that no recall, search, or `build_context` result ever crosses tenant boundaries.

## Why it matters

Tenant isolation is a core correctness invariant (enforced in the query layer),
but there is no dedicated regression test suite that explicitly verifies it. As
new query paths, blend passes, and graph expansion logic are added, a missing
test could silently allow a cross-tenant data leak. A comprehensive isolation
audit closes that gap.

## Behaviour

No new production code is required — this feature is purely additive tests:

1. Create at least 2 tenants, each with matching collection names.
2. Store semantically identical content in both tenants.
3. For each combination of:
   - `recall()` / `search()` / `build_context()`
   - `SearchMode::Vector`, `Text`, `Hybrid`
   - `include_related = true/false` (for `build_context`)
4. Assert: every result item's `tenant_id` matches the queried tenant.
5. Assert: no item from the other tenant appears in any result.
6. Assert: `add_graph_edge` and `list_graph_edges` are also tenant-isolated.

## Files

- `crates/hippocore/tests/tenant_isolation.rs` — new dedicated test file.
- `docs/en/TENANT_ISOLATION_AUDIT.md` and `docs/pt-br/TENANT_ISOLATION_AUDIT.md`.
- `docs/en/STATUS.md` and `docs/pt-br/STATUS.md`.

## Acceptance criteria

- At least 6 targeted isolation assertions (one per query path × mode).
- No production code changes required.
- All quality gates pass:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Out of scope

- New production-code tenant enforcement.
- Server mode or multi-node isolation.
- Permission / RBAC model.
