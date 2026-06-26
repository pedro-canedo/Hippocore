# Multi-tenant Query Isolation Audit v0.1

## Overview

This is a dedicated regression test suite proving that no public query path
(`recall`, `search`, `build_context`, `list_graph_edges`) ever returns data
from a tenant other than the one specified in the request, even when tenants
share identical collection names and store semantically identical content.

No production code was changed; this feature is purely additive tests.

## Test file

`crates/hippocore/tests/tenant_isolation.rs`

## What is tested

| Test | Path | Mode |
|---|---|---|
| `recall_vector_stays_within_tenant` | `recall()` | Vector |
| `recall_text_stays_within_tenant` | `recall()` | Text |
| `recall_hybrid_stays_within_tenant` | `recall()` | Hybrid |
| `build_context_stays_within_tenant` | `build_context()` | — |
| `build_context_with_related_stays_within_tenant` | `build_context()` | with `include_related = true` |
| `list_graph_edges_stays_within_tenant` | `list_graph_edges()` | — |

## Test strategy

All tests use a shared helper `two_tenant_db` that:
- Creates two tenants (`alpha`, `beta`) with identical collection names (`shared`).
- Stores identical content (same text) in both tenants.
- Returns the populated `Hippocore` handle.

Each test then queries one tenant and asserts that every result item's tenant
matches the queried tenant, and no item from the other tenant leaks in.

## Invariant confirmed

`tenant_id` in the query request is a hard isolation boundary for all public
read paths. It is enforced in the `query` module and verified here.
