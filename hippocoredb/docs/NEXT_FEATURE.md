# Next Feature

## Feature name

**BM25 text scoring** — replace the current TF-IDF text score with Okapi BM25
(term-frequency saturation + document-length normalization).

## Why it matters

Text relevance is half of hybrid recall. The current score is `tf * idf` summed
over query terms, with no length normalization — so longer chunks accumulate
higher scores just by being long, and repeated terms grow unbounded. BM25 fixes
both with term-frequency saturation (`k1`) and length normalization (`b`),
which materially improves ranking quality (and therefore hybrid quality, since
the text component feeds the fusion).

## Expected behavior

- `index::text_score` computes BM25 instead of TF-IDF:
  - `idf(t) = ln(1 + (N - df + 0.5) / (df + 0.5))`,
  - per term: `idf * (f * (k1 + 1)) / (f + k1 * (1 - b + b * |d| / avgdl))`,
  - where `f` = term frequency in the entry, `|d|` = entry token count, `avgdl`
    = average entry token count across the index.
- The index tracks each entry's token count and the running average document
  length (updated on insert/remove).
- Defaults `k1 = 1.2`, `b = 0.75` (optionally configurable on `Config` later).
- Hybrid fusion is unchanged (scores still min-max normalized before blending),
  so only the raw text signal improves.

## Affected modules / files

- `index.rs` — store `token_count` per `IndexEntry`; maintain `total_tokens` and
  entry count for `avgdl`; rewrite `text_score`; keep `idf` (or fold into BM25).
- `query.rs` — no change to fusion; still calls `text_score`.
- `config.rs` — optional `bm25_k1` / `bm25_b` (can default-only for now).
- `docs/STATUS.md`, `CHANGELOG.md` — update on completion.

## Acceptance criteria

- For two chunks containing a query term, the shorter/more-focused chunk is not
  unfairly out-ranked purely due to length; a targeted test asserts the BM25
  ordering differs from naive TF in the expected case.
- A term repeated many times saturates (does not grow linearly).
- All existing recall/sorting/hybrid tests still pass (ordering may shift, but
  relevance assertions in tests should hold; adjust any that depended on raw TF).
- `cargo fmt`, `cargo test`, `cargo clippy -D warnings` all pass.

## Tests to add

- `bm25_length_normalization`: a short exact-match chunk ranks above a very long
  chunk that mentions the term once among many tokens.
- `bm25_saturates_repeated_terms`: doubling a term's count less-than-doubles its
  contribution.
- Keep an existing TF-style ordering test or update it for BM25 semantics.

## Risks / open questions

- `avgdl` must stay correct across inserts, overwrites, and deletes — update the
  running totals in `Index::insert` / `Index::remove` and test after deletes.
- Recomputing `avgdl` per query is O(1) if totals are maintained; avoid scanning.
- BM25 with `b` and length norm interacts with min-max fusion; verify hybrid
  tests still rank the intuitively-correct result first.
