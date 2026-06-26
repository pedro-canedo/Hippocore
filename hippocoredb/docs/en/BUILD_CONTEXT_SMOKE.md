# build_context Smoke Tests v0.1

## Overview

A dedicated test suite verifying that `build_context` assembles a usable
LLM-ready context block from recalled items. No production code was changed;
this feature is purely additive tests.

## Test file

`crates/hippocore/tests/build_context_smoke.rs`

## What is tested

| Test | Scenario |
|---|---|
| `build_context_produces_non_empty_text` | Non-empty `text` when relevant memories exist |
| `context_text_contains_memory_snippet` | Context string contains recognisable word from the stored memory |
| `max_tokens_limits_items_included` | Tight budget includes fewer items than a loose budget |
| `include_related_adds_graph_neighbours` | Graph-connected item appears in `items_included` when `include_related = true` |
| `empty_store_returns_empty_context_not_error` | Empty collection → `text == ""`, `items_included.is_empty()`, no error |

## Context format

Each item in `ContextBlock.text` is formatted as: `[<kind>:<id>]\n<text>`,
separated by `\n\n`. Token count: 1 token ≈ 4 bytes.
