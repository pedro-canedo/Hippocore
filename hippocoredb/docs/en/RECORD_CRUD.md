# Record CRUD Tests v0.1

## Overview

A dedicated test suite covering the full `Record` entity lifecycle: store,
retrieve by id, appear in recall, version increment on update, delete, and
tenant isolation. No production code was changed; this feature is purely
additive tests.

## Test file

`crates/hippocore/tests/record_crud.rs`

## What is tested

| Test | Scenario |
|---|---|
| `stored_record_is_retrievable_by_id` | `put_record` → `get_record` returns the same id |
| `record_appears_in_recall` | Stored record appears in `recall()` for a matching query |
| `update_increments_version` | Re-`put_record` with the same id increments `version` from 0 to 1 |
| `delete_record_returns_none_on_get` | After `delete_record`, `get_record` returns `None` |
| `deleted_record_absent_from_recall` | After deletion, record does not appear in `recall()` |
| `records_are_tenant_isolated` | One tenant cannot retrieve or recall another tenant's records |

## Design invariants confirmed

- Initial `version` is `0`; each update increments it by 1.
- `delete_record` is durable — the record disappears from both point-lookup
  and recall.
- Tenant isolation applies to both `get_record` (id lookup) and `recall`
  (vector/text search).
