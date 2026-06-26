# Next Feature

## Feature name

**Phase 13 — Metadata Filtering & Faceted Search**

## Why it matters

Phase 12 gave Hippocore the ability to ingest diverse content types. Phase 13
makes the retrieval layer more precise: callers often need to scope queries to
a date range, a specific source, a confidence band, or a custom metadata key.
Without first-class filtering, every recall result must be post-processed by the
caller.

## Minimum viable scope for Phase 13

1. **Metadata filter expressions**: extend `RecallRequest` and `BuildContextRequest`
   to accept richer filter expressions beyond the current exact-match
   `Metadata` map — at minimum `$gte`, `$lte`, `$in`, and `$ne` operators on
   string and numeric metadata values.
2. **Date-range filter**: shorthand helpers on `RecallRequest` for
   `valid_from >= X` and `valid_until <= Y`.
3. **Confidence range filter**: `min_confidence: Option<f32>` on `RecallRequest`
   to exclude memories below a threshold.
4. **Facet counts**: new method `Hippocore::facets(tenant, collection, field)`
   returning `Vec<(String, usize)>` — distinct values and their document counts
   for a metadata field.
5. **Tests**: at least 6 integration tests covering each new filter operator
   and facets.
6. **Docs**: bilingual documentation.

## Out of scope for Phase 13

- Full query language (SQL-like DSL).
- Geospatial filtering.
- Nested metadata objects.
- Write-through index updates for large filter scans.
