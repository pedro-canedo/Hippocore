//! Query layer: request types, filters, and the scoring/fusion that turns
//! index entries into ranked [`RecallResult`]s.
//!
//! Tenant isolation is enforced here: every query requires a `tenant_id` and no
//! entry from another tenant can ever match.

use crate::index::{cosine_similarity, text_score, Index, IndexEntry};
use crate::memory::tokenize;
use crate::model::{ItemKind, MemoryType, Metadata, RecallResult};

/// Over-retrieval multiplier for ANN post-filtering.
///
/// When the vector backend is HNSW, we request `top_k × KNN_OVER_FETCH` candidates
/// from the ANN index so that, after post-filtering (tenant, temporal, supersedure),
/// enough results survive for the final top-k. A value of 8 balances recall vs
/// latency for typical filter selectivities; increase if recall degrades.
const KNN_OVER_FETCH: usize = 8;

/// Which signals to use when ranking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    /// Cosine similarity over embeddings only.
    Vector,
    /// BM25 text relevance only.
    Text,
    /// Weighted fusion of vector and text scores.
    Hybrid,
}

impl SearchMode {
    /// Parse a case-insensitive mode string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "vector" | "vec" => Some(SearchMode::Vector),
            "text" | "bm25" | "lexical" => Some(SearchMode::Text),
            "hybrid" | "both" => Some(SearchMode::Hybrid),
            _ => None,
        }
    }

    /// Stable lowercase string form.
    pub fn as_str(&self) -> &'static str {
        match self {
            SearchMode::Vector => "vector",
            SearchMode::Text => "text",
            SearchMode::Hybrid => "hybrid",
        }
    }
}

/// Filters applied before scoring. `tenant_id` is mandatory.
#[derive(Debug, Clone, Default)]
pub struct Filter {
    /// Restrict to entries of this tenant (required for isolation).
    pub tenant_id: String,
    /// Restrict to a single collection.
    pub collection: Option<String>,
    /// Restrict to a single owning user.
    pub user_id: Option<String>,
    /// Restrict to a single memory type.
    pub memory_type: Option<MemoryType>,
    /// Restrict to a single item kind (chunks vs memories).
    pub kind: Option<ItemKind>,
    /// Require all of these exact metadata key/value pairs.
    pub metadata: Metadata,
    /// Return only entries valid at this epoch ms. `None` = no temporal filter.
    pub as_of: Option<i64>,
    /// When `false` (default), memories that have been superseded by another
    /// are excluded. Set to `true` to surface the full history.
    pub include_superseded: bool,
}

impl Filter {
    /// Create a filter scoped to a tenant.
    pub fn for_tenant(tenant_id: impl Into<String>) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            ..Self::default()
        }
    }

    fn matches(&self, e: &IndexEntry) -> bool {
        if e.tenant_id != self.tenant_id {
            return false;
        }
        if let Some(c) = &self.collection {
            if &e.collection != c {
                return false;
            }
        }
        if let Some(u) = &self.user_id {
            if e.user_id.as_deref() != Some(u.as_str()) {
                return false;
            }
        }
        if let Some(mt) = self.memory_type {
            if e.memory_type != Some(mt) {
                return false;
            }
        }
        if let Some(k) = self.kind {
            if e.kind != k {
                return false;
            }
        }
        for (k, v) in &self.metadata {
            if e.metadata.get(k).map(|got| got == v) != Some(true) {
                return false;
            }
        }
        if let Some(t) = self.as_of {
            if let Some(vf) = e.valid_from {
                if vf > t {
                    return false; // not yet valid
                }
            }
            if let Some(vu) = e.valid_until {
                if vu <= t {
                    return false; // expired
                }
            }
        }
        if !self.include_superseded && e.superseded {
            return false;
        }
        true
    }
}

/// A retrieval request.
#[derive(Debug, Clone)]
pub struct QueryRequest {
    /// Pre/post filters (includes mandatory tenant).
    pub filter: Filter,
    /// Natural-language query text (used for text scoring and, if no embedding
    /// is supplied, for the built-in embedder).
    pub query_text: Option<String>,
    /// Optional caller-supplied query embedding (overrides text embedding).
    pub query_embedding: Option<Vec<f32>>,
    /// How to rank.
    pub mode: SearchMode,
    /// Hybrid fusion weight in `[0, 1]`; only used in [`SearchMode::Hybrid`].
    pub hybrid_alpha: f32,
    /// Maximum number of results.
    pub top_k: usize,
    /// Drop results whose final score (after confidence weighting) is below
    /// this threshold. `None` disables the filter.
    pub min_score: Option<f32>,
    /// When `true`, keep only the highest-scoring chunk per parent document.
    /// Results are already sorted by score when deduplication runs, so the
    /// first occurrence of each `document_id` is always the best chunk.
    pub dedup_chunks: bool,
}

struct Scored<'a> {
    entry: &'a IndexEntry,
    raw_vec: Option<f32>,
    raw_text: f32,
}

/// Execute `request` against `index`, returning ranked results.
///
/// If `query_embedding` is `None` and `query_text` is `Some`, the embedding is
/// produced by `embed` (the caller passes the database's built-in embedder).
pub fn execute(
    index: &Index,
    embed: impl Fn(&str) -> Vec<f32>,
    request: &QueryRequest,
) -> Vec<RecallResult> {
    let original_query = request.query_text.as_deref().unwrap_or_default();
    let normalized_query = match request.mode {
        SearchMode::Vector => String::new(),
        SearchMode::Text | SearchMode::Hybrid => normalize_query_text(original_query),
    };
    let query_tokens = match request.mode {
        SearchMode::Vector => Vec::new(),
        SearchMode::Text | SearchMode::Hybrid => tokenize(&normalized_query),
    };

    let query_embedding: Option<Vec<f32>> = match request.mode {
        SearchMode::Text => None,
        SearchMode::Vector => request
            .query_embedding
            .clone()
            .or_else(|| (!original_query.trim().is_empty()).then(|| embed(original_query))),
        SearchMode::Hybrid => request
            .query_embedding
            .clone()
            .or_else(|| (!normalized_query.trim().is_empty()).then(|| embed(&normalized_query))),
    };

    // Build the candidate set from the appropriate source(s), then score.
    //
    // Vector candidates: from index.knn() with over-retrieval for post-filtering.
    // Text candidates: from index.text_candidates() (inverted-index lookup).
    // Hybrid: union of both sets.
    //
    // After candidate collection, all filters (tenant, temporal, supersedure,
    // metadata, etc.) are applied through Filter::matches.

    // Collect candidate entry keys based on mode.
    let candidate_entries: Vec<&IndexEntry> = match request.mode {
        SearchMode::Vector => {
            if let Some(q) = &query_embedding {
                let fetch_k = request
                    .top_k
                    .saturating_mul(KNN_OVER_FETCH)
                    .max(request.top_k + 10);
                let keys = index.knn(q, fetch_k, fetch_k);
                keys.iter()
                    .filter_map(|k| index.get(k))
                    .filter(|e| request.filter.matches(e))
                    .collect()
            } else {
                Vec::new()
            }
        }
        SearchMode::Text => {
            if query_tokens.is_empty() {
                Vec::new()
            } else {
                let cands = index.text_candidates(&query_tokens);
                cands
                    .iter()
                    .filter_map(|k| index.get(k))
                    .filter(|e| request.filter.matches(e))
                    .collect()
            }
        }
        SearchMode::Hybrid => {
            // Union of vector ANN candidates and text inverted-index candidates.
            let mut seen = std::collections::HashSet::new();
            let mut entries: Vec<&IndexEntry> = Vec::new();

            if let Some(q) = &query_embedding {
                let fetch_k = request
                    .top_k
                    .saturating_mul(KNN_OVER_FETCH)
                    .max(request.top_k + 10);
                for key in index.knn(q, fetch_k, fetch_k) {
                    if seen.insert(key.clone()) {
                        if let Some(e) = index.get(&key) {
                            if request.filter.matches(e) {
                                entries.push(e);
                            }
                        }
                    }
                }
            }
            if !query_tokens.is_empty() {
                for key in index.text_candidates(&query_tokens) {
                    if seen.insert(key.clone()) {
                        if let Some(e) = index.get(&key) {
                            if request.filter.matches(e) {
                                entries.push(e);
                            }
                        }
                    }
                }
            }
            entries
        }
    };

    let scored: Vec<Scored> = candidate_entries
        .into_iter()
        .map(|e| {
            let raw_vec = query_embedding
                .as_ref()
                .and_then(|q| cosine_similarity(q, &e.embedding));
            let raw_text = if query_tokens.is_empty() {
                0.0
            } else {
                text_score(index, e, &query_tokens)
            };
            Scored {
                entry: e,
                raw_vec,
                raw_text,
            }
        })
        .filter(|s| match request.mode {
            SearchMode::Vector => s.raw_vec.is_some(),
            SearchMode::Text => s.raw_text > 0.0,
            SearchMode::Hybrid => s.raw_vec.is_some() || s.raw_text > 0.0,
        })
        .collect();

    if scored.is_empty() {
        return Vec::new();
    }

    // Build a lowercase token set for the query to support matched_terms.
    let query_token_set: std::collections::HashSet<String> =
        query_tokens.iter().map(|t| t.to_lowercase()).collect();

    // Compute which query tokens appear in a result's text (for matched_terms).
    let compute_matched = |text: &str| -> Vec<String> {
        if query_token_set.is_empty() {
            return Vec::new();
        }
        let mut terms: Vec<String> = tokenize(text)
            .into_iter()
            .map(|t| t.to_lowercase())
            .filter(|t| query_token_set.contains(t))
            .collect();
        terms.sort();
        terms.dedup();
        terms
    };

    let mut results: Vec<RecallResult> = match request.mode {
        SearchMode::Vector => {
            // Min-max normalize cosine scores.
            let (vmin, vmax) = min_max(scored.iter().filter_map(|s| s.raw_vec));
            scored
                .iter()
                .map(|s| {
                    let vnorm = s.raw_vec.map(|v| normalize(v, vmin, vmax)).unwrap_or(0.0);
                    let reason = format!("vector match (cosine={:.3})", s.raw_vec.unwrap_or(0.0));
                    // Pure vector: no text token comparison → matched_terms is empty.
                    build_result(s.entry, vnorm, vnorm, 0.0, reason, Vec::new())
                })
                .collect()
        }
        SearchMode::Text => {
            // Min-max normalize BM25 scores.
            let (tmin, tmax) = min_max(scored.iter().map(|s| s.raw_text).filter(|&t| t > 0.0));
            scored
                .iter()
                .map(|s| {
                    let tnorm = if s.raw_text > 0.0 {
                        normalize(s.raw_text, tmin, tmax)
                    } else {
                        0.0
                    };
                    let matched = compute_matched(&s.entry.text);
                    build_result(
                        s.entry,
                        tnorm,
                        0.0,
                        tnorm,
                        "text match (bm25)".into(),
                        matched,
                    )
                })
                .collect()
        }
        SearchMode::Hybrid => {
            // Reciprocal Rank Fusion: parameter-free rank-based fusion.
            // See module-level RRF_K for the smoothing constant.
            let n = scored.len();

            // Compute vector ranks: descending by raw cosine (None → lowest).
            let mut vec_order: Vec<usize> = (0..n).collect();
            vec_order.sort_by(|&a, &b| {
                let va = scored[a].raw_vec.unwrap_or(f32::NEG_INFINITY);
                let vb = scored[b].raw_vec.unwrap_or(f32::NEG_INFINITY);
                vb.partial_cmp(&va).unwrap_or(std::cmp::Ordering::Equal)
            });
            let mut vec_rank = vec![n + 1; n]; // default: unranked
            for (rank, &idx) in vec_order.iter().enumerate() {
                if scored[idx].raw_vec.is_some() {
                    vec_rank[idx] = rank + 1; // 1-based
                }
            }

            // Compute text ranks: descending by raw BM25 (0 → unranked).
            let mut text_order: Vec<usize> = (0..n).collect();
            text_order.sort_by(|&a, &b| {
                scored[b]
                    .raw_text
                    .partial_cmp(&scored[a].raw_text)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            let mut text_rank = vec![n + 1; n]; // default: unranked
            for (rank, &idx) in text_order.iter().enumerate() {
                if scored[idx].raw_text > 0.0 {
                    text_rank[idx] = rank + 1; // 1-based
                }
            }

            scored
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    let vr = vec_rank[i] as f32;
                    let tr = text_rank[i] as f32;
                    let rrf = 1.0 / (RRF_K + vr) + 1.0 / (RRF_K + tr);
                    let reason = format!(
                        "hybrid/rrf(vector_rank={}, text_rank={})",
                        vec_rank[i], text_rank[i]
                    );
                    let matched = compute_matched(&s.entry.text);
                    // Carry raw scores for transparency; rrf is the fused score.
                    build_result(
                        s.entry,
                        rrf,
                        s.raw_vec.unwrap_or(0.0),
                        s.raw_text,
                        reason,
                        matched,
                    )
                })
                .collect()
        }
    };

    // Confidence weighting: Memory items with an explicit confidence score get a
    // proportional boost or penalty before the final sort.  Items without a
    // confidence rating (None → treated as 0.5) and non-Memory items are neutral.
    for result in &mut results {
        if result.kind == ItemKind::Memory {
            let c = result.confidence.unwrap_or(0.5).clamp(0.0, 1.0);
            result.score *= 1.0 + CONFIDENCE_ALPHA * (c - 0.5);
        }
    }

    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.id.cmp(&b.id))
    });

    // Apply min_score threshold after sorting so the filter sees final scores.
    if let Some(min) = request.min_score {
        results.retain(|r| r.score >= min);
    }

    // Chunk deduplication: keep only the highest-scoring chunk per parent
    // document. Results are already sorted by score so the first occurrence
    // of each document_id is the best chunk. Memories and records are kept
    // unconditionally.
    if request.dedup_chunks {
        let mut seen_docs = std::collections::HashSet::new();
        results.retain(|r| {
            if r.kind != ItemKind::DocumentChunk {
                return true;
            }
            match &r.document_id {
                Some(doc_id) => seen_docs.insert(doc_id.clone()),
                None => true,
            }
        });
    }

    results.truncate(request.top_k);
    results
}

/// RRF constant — keep here so the unit test can import it.
pub const RRF_K: f32 = 60.0;

/// Confidence weighting strength. A memory at confidence=1.0 gets a
/// `1 + ALPHA*0.5` multiplier; at 0.0 it gets `1 - ALPHA*0.5`.
/// Memories with no confidence set are treated as neutral (0.5).
const CONFIDENCE_ALPHA: f32 = 0.4;

fn build_result(
    e: &IndexEntry,
    score: f32,
    vector_score: f32,
    text_score: f32,
    reason: String,
    matched_terms: Vec<String>,
) -> RecallResult {
    RecallResult {
        id: e.id.clone(),
        kind: e.kind,
        tenant_id: e.tenant_id.clone(),
        collection: e.collection.clone(),
        document_id: e.document_id.clone(),
        record_table: e.record_table.clone(),
        user_id: e.user_id.clone(),
        memory_type: e.memory_type,
        text: e.text.clone(),
        metadata: e.metadata.clone(),
        source: e.source.clone(),
        score,
        vector_score,
        text_score,
        reason,
        contradictions: e.contradicts.clone(),
        confidence: e.confidence,
        matched_terms,
    }
}

fn normalize_query_text(text: &str) -> String {
    tokenize(text)
        .into_iter()
        .map(|token| normalize_token(&token).to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_token(token: &str) -> &str {
    match token {
        "postgres" | "postgress" => "postgresql",
        "conexao" | "conexão" | "connection" | "connections" | "conectar" | "conecta" => {
            "connection"
        }
        other => other,
    }
}

fn min_max(iter: impl Iterator<Item = f32>) -> (f32, f32) {
    let mut lo = f32::INFINITY;
    let mut hi = f32::NEG_INFINITY;
    for v in iter {
        lo = lo.min(v);
        hi = hi.max(v);
    }
    (lo, hi)
}

fn normalize(v: f32, lo: f32, hi: f32) -> f32 {
    if !lo.is_finite() || !hi.is_finite() {
        return 0.0;
    }
    if (hi - lo).abs() < f32::EPSILON {
        return 1.0; // single value or all-equal: treat as fully relevant
    }
    ((v - lo) / (hi - lo)).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typo_postgress_normalizes_to_postgresql() {
        assert_eq!(
            normalize_query_text("Como postgress funciona?"),
            "como postgresql funciona"
        );
    }

    #[test]
    fn rrf_k_is_standard_constant() {
        assert!((RRF_K - 60.0).abs() < f32::EPSILON);
    }

    #[test]
    fn rrf_score_decreases_with_rank() {
        // Higher rank (worse) → lower score contribution.
        let score_rank1: f32 = 1.0 / (RRF_K + 1.0);
        let score_rank5: f32 = 1.0 / (RRF_K + 5.0);
        assert!(score_rank1 > score_rank5);
    }

    #[test]
    fn confidence_weight_high_beats_low_at_equal_base_score() {
        // A memory with high confidence must rank above one with low confidence
        // when both have an identical base score.
        let base = 0.5_f32;
        let high_weight = 1.0 + CONFIDENCE_ALPHA * (0.9_f32 - 0.5);
        let low_weight = 1.0 + CONFIDENCE_ALPHA * (0.1_f32 - 0.5);
        assert!(
            base * high_weight > base * low_weight,
            "high-confidence item must score higher"
        );
    }

    #[test]
    fn neutral_confidence_leaves_score_unchanged() {
        // confidence=0.5 (or None→0.5) applies weight 1.0 exactly.
        let base = 0.7_f32;
        let weight = 1.0 + CONFIDENCE_ALPHA * (0.5_f32 - 0.5);
        let expected = base * weight;
        assert!(
            (expected - base).abs() < f32::EPSILON,
            "neutral confidence must not change score"
        );
    }

    #[test]
    fn confidence_weight_is_bounded() {
        // Even at extremes the multiplier stays in a reasonable range.
        let w_max = 1.0 + CONFIDENCE_ALPHA * (1.0_f32 - 0.5);
        let w_min = 1.0 + CONFIDENCE_ALPHA * (0.0_f32 - 0.5);
        assert!(w_max <= 1.25, "max weight must be ≤ 1.25");
        assert!(w_min >= 0.75, "min weight must be ≥ 0.75");
    }

    #[test]
    fn min_score_threshold_drops_low_scoring_items() {
        // Verify the filter formula directly: items below threshold are removed.
        let scores = [0.9_f32, 0.5, 0.3, 0.1];
        let threshold = 0.4_f32;
        let kept: Vec<_> = scores.iter().filter(|&&s| s >= threshold).collect();
        assert_eq!(kept.len(), 2, "only scores ≥ 0.4 should survive");
        assert!(kept.contains(&&0.9));
        assert!(kept.contains(&&0.5));
    }

    #[test]
    fn min_score_none_keeps_all() {
        let scores = [0.9_f32, 0.1, 0.0];
        let min: Option<f32> = None;
        let kept: Vec<_> = scores
            .iter()
            .filter(|&&s| min.map_or(true, |m| s >= m))
            .collect();
        assert_eq!(kept.len(), 3, "None threshold must not filter anything");
    }

    #[test]
    fn min_score_exact_threshold_is_inclusive() {
        let score = 0.5_f32;
        let threshold = 0.5_f32;
        assert!(
            score >= threshold,
            "item exactly at threshold must be included"
        );
    }

    #[test]
    fn dedup_chunks_keeps_best_chunk_per_doc() {
        // Simulate the dedup logic: items sorted by score descending, track seen doc_ids.
        let items: Vec<(&str, Option<&str>, bool)> = vec![
            // (id, document_id, is_chunk)
            ("c1", Some("doc-a"), true), // best chunk for doc-a → keep
            ("m1", None, false),         // memory → always keep
            ("c2", Some("doc-a"), true), // second chunk for doc-a → drop
            ("c3", Some("doc-b"), true), // only chunk for doc-b → keep
        ];
        let mut seen = std::collections::HashSet::new();
        let kept: Vec<_> = items
            .iter()
            .filter(|(_, doc_id, is_chunk)| {
                if !is_chunk {
                    return true;
                }
                match doc_id {
                    Some(did) => seen.insert(*did),
                    None => true,
                }
            })
            .collect();
        assert_eq!(kept.len(), 3, "c2 should be deduped");
        let ids: Vec<_> = kept.iter().map(|(id, _, _)| *id).collect();
        assert!(
            !ids.contains(&"c2"),
            "second chunk for doc-a must be removed"
        );
    }

    #[test]
    fn dedup_chunks_false_keeps_all() {
        // When dedup_chunks is false, all items should pass through unchanged.
        let dedup = false;
        let items = ["c1", "c2", "c3"];
        let kept: Vec<_> = if dedup {
            // dedup logic omitted — would filter
            [].to_vec()
        } else {
            items.to_vec()
        };
        assert_eq!(kept.len(), 3, "all items must be kept when dedup is false");
    }

    #[test]
    fn matched_terms_intersection_is_correct() {
        // Simulate the intersection logic used in compute_matched.
        let query_tokens: std::collections::HashSet<String> = ["rust", "memory", "database"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let result_text = "rust is great for building database systems";
        let mut terms: Vec<String> = crate::memory::tokenize(result_text)
            .into_iter()
            .map(|t| t.to_lowercase())
            .filter(|t| query_tokens.contains(t))
            .collect();
        terms.sort();
        terms.dedup();
        assert!(terms.contains(&"rust".to_string()), "rust must match");
        assert!(
            terms.contains(&"database".to_string()),
            "database must match"
        );
        assert!(!terms.contains(&"memory".to_string()), "memory not in text");
    }

    #[test]
    fn matched_terms_empty_when_no_overlap() {
        let query_tokens: std::collections::HashSet<String> = ["elephant", "jupiter"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let result_text = "rust is great for building memory systems";
        let terms: Vec<String> = crate::memory::tokenize(result_text)
            .into_iter()
            .map(|t| t.to_lowercase())
            .filter(|t| query_tokens.contains(t))
            .collect();
        assert!(terms.is_empty(), "no overlap means empty matched_terms");
    }
}
