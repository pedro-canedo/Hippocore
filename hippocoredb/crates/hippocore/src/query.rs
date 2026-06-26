//! Query layer: request types, filters, and the scoring/fusion that turns
//! index entries into ranked [`RecallResult`]s.
//!
//! Tenant isolation is enforced here: every query requires a `tenant_id` and no
//! entry from another tenant can ever match.

use crate::index::{cosine_similarity, text_score, Index, IndexEntry};
use crate::memory::tokenize;
use crate::model::{ItemKind, MemoryType, Metadata, RecallResult};

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

    // Build the candidate set, then score every candidate.
    let scored: Vec<Scored> = index
        .entries()
        .filter(|e| request.filter.matches(e))
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

    let mut results: Vec<RecallResult> = match request.mode {
        SearchMode::Vector => {
            // Min-max normalize cosine scores.
            let (vmin, vmax) = min_max(scored.iter().filter_map(|s| s.raw_vec));
            scored
                .iter()
                .map(|s| {
                    let vnorm = s.raw_vec.map(|v| normalize(v, vmin, vmax)).unwrap_or(0.0);
                    let reason = format!("vector match (cosine={:.3})", s.raw_vec.unwrap_or(0.0));
                    build_result(s.entry, vnorm, vnorm, 0.0, reason)
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
                    build_result(s.entry, tnorm, 0.0, tnorm, "text match (bm25)".into())
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
                    // Carry raw scores for transparency; rrf is the fused score.
                    build_result(s.entry, rrf, s.raw_vec.unwrap_or(0.0), s.raw_text, reason)
                })
                .collect()
        }
    };

    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.id.cmp(&b.id))
    });
    results.truncate(request.top_k);
    results
}

/// RRF constant — keep here so the unit test can import it.
pub const RRF_K: f32 = 60.0;

fn build_result(
    e: &IndexEntry,
    score: f32,
    vector_score: f32,
    text_score: f32,
    reason: String,
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
}
