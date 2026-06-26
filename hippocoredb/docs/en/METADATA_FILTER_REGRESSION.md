# Metadata Filter Regression Suite v0.1

## Overview

A dedicated test suite that exercises exact-match metadata filtering across all
item kinds (memories, document chunks, records), multi-key AND semantics, empty
result handling, collection-scoped filtering, cross-tenant non-interference, and
`build_context` filter propagation.

No production code was changed; this feature is purely additive tests.

## Test file

`crates/hippocore/tests/metadata_filter.rs`

## What is tested

| Test | Scenario |
|---|---|
| `metadata_filter_exact_match_memories` | Single-key filter on memories; non-matching memory excluded |
| `metadata_filter_multi_key_and_semantics` | Two-key filter (env=prod AND tier=api); items matching only one key excluded |
| `metadata_filter_no_match_returns_empty_not_error` | Filter that matches zero items returns empty `Vec`, not an error |
| `metadata_filter_scoped_to_collection` | Filter combined with `collection` scope; items in other collection excluded |
| `metadata_filter_records` | Single-key filter on structured records |
| `metadata_filter_does_not_cross_tenant_boundary` | Same metadata key/value in two tenants; querying one tenant never returns items from the other |
| `build_context_metadata_filter_respected` | `BuildContextRequest.metadata_filter` propagated into the recall pass |

## Non-goals

- New filter operators (prefix, range, regex).
- Server mode.
- Production code changes.
