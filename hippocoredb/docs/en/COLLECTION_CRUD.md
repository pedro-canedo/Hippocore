# Collection CRUD Tests v0.1

## Overview

A dedicated test suite covering the full collection lifecycle: creation,
listing, idempotency, cross-tenant isolation, and recall scoped to
non-existent collections. No production code was changed; this feature is
purely additive tests.

## Test file

`crates/hippocore/tests/collection_crud.rs`

## What is tested

| Test | Scenario |
|---|---|
| `created_collection_appears_in_list` | After `create_collection`, it appears in `collections(tenant)` |
| `duplicate_collection_is_idempotent` | Calling `create_collection` with the same name succeeds and returns the existing collection; only one entry exists |
| `tenants_can_share_collection_names` | Two tenants may have collections with the same name without conflict |
| `collection_list_is_tenant_isolated` | `collections(tenant_id)` never returns collections from another tenant |
| `recall_scoped_to_nonexistent_collection_returns_empty` | Recall with `collection = Some("nonexistent")` returns empty, not an error |
| `collection_description_is_stored` | Description round-trips through creation |

## Design invariants confirmed

- `create_collection` is idempotent: calling it twice with the same `(tenant_id, name)` pair returns the existing collection and does not create a duplicate. Callers can use it as a safe upsert without pre-checking existence.
- The `collections(tenant_id)` listing is hard-filtered by `tenant_id`; collections from other tenants never appear.
