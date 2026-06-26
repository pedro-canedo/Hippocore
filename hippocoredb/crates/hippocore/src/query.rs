//! Query layer: request types, filters, and the scoring/fusion that turns
//! index entries into ranked [`RecallResult`]s.
//!
//! Tenant isolation is enforced here: every query requires a `tenant_id` and no
//! entry from another tenant can ever match.

use std::collections::HashSet;

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
    tags: Option<EntityTags>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct EntityTags {
    oracle: bool,
    postgresql: bool,
    python: bool,
    listener: bool,
    vacuum: bool,
    connection: bool,
}

impl EntityTags {
    fn has_exclusive_postgresql(&self) -> bool {
        self.postgresql && !self.oracle
    }

    fn has_exclusive_oracle(&self) -> bool {
        self.oracle && !self.postgresql
    }

    fn names(&self) -> Vec<&'static str> {
        let mut out = Vec::new();
        if self.oracle {
            out.push("oracle");
        }
        if self.postgresql {
            out.push("postgresql");
        }
        if self.python {
            out.push("python");
        }
        if self.listener {
            out.push("listener");
        }
        if self.vacuum {
            out.push("vacuum");
        }
        if self.connection {
            out.push("connection");
        }
        out
    }
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
    let normalized_query = request
        .query_text
        .as_deref()
        .map(normalize_query_text)
        .unwrap_or_default();
    let query_tokens = tokenize(&normalized_query);
    let query_tags = detect_entity_tags(&normalized_query);
    let should_adjust_scores =
        request.mode != SearchMode::Vector && (query_tags.oracle || query_tags.postgresql);

    let query_embedding: Option<Vec<f32>> = match request.mode {
        SearchMode::Text => None,
        _ => request
            .query_embedding
            .clone()
            .or_else(|| (!normalized_query.trim().is_empty()).then(|| embed(&normalized_query))),
    };

    // Build the candidate set, then score every candidate.
    let mut scored: Vec<Scored> = index
        .entries()
        .filter(|e| request.filter.matches(e))
        .map(|e| {
            let tags = should_adjust_scores.then(|| detect_entry_tags(e));
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
                tags,
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

    // Min-max normalize each signal across the candidate set into [0, 1].
    let (vmin, vmax) = min_max(scored.iter().filter_map(|s| s.raw_vec));
    let (tmin, tmax) = min_max(scored.iter().map(|s| s.raw_text).filter(|&t| t > 0.0));

    let alpha = clamp01(request.hybrid_alpha);
    let mut results: Vec<RecallResult> = scored
        .drain(..)
        .map(|s| {
            let vnorm = s.raw_vec.map(|v| normalize(v, vmin, vmax)).unwrap_or(0.0);
            let tnorm = if s.raw_text > 0.0 {
                normalize(s.raw_text, tmin, tmax)
            } else {
                0.0
            };
            let (score, reason) = match request.mode {
                SearchMode::Vector => (
                    vnorm,
                    format!("vector match (cosine={:.3})", s.raw_vec.unwrap_or(0.0)),
                ),
                SearchMode::Text => (tnorm, "text match (bm25)".to_string()),
                SearchMode::Hybrid => (
                    alpha * vnorm + (1.0 - alpha) * tnorm,
                    format!("hybrid(vector={vnorm:.3}, text={tnorm:.3})"),
                ),
            };
            let adjustment = s
                .tags
                .as_ref()
                .map(|tags| score_adjustment(&query_tags, tags))
                .unwrap_or_default();
            let final_score = (score + adjustment.delta).clamp(0.0, 1.0);
            let reason = explain_score(reason, s.tags.as_ref(), &adjustment);
            build_result(s.entry, final_score, vnorm, tnorm, reason)
        })
        .collect();

    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.id.cmp(&b.id))
    });
    results.truncate(request.top_k);
    results
}

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

fn detect_entry_tags(entry: &IndexEntry) -> EntityTags {
    let mut tags = EntityTags::default();
    for token in entry.tf.keys() {
        mark_tag(&mut tags, normalize_token(token));
    }
    for (key, value) in &entry.metadata {
        for token in tokenize(key) {
            mark_tag(&mut tags, normalize_token(&token));
        }
        for token in tokenize(value) {
            mark_tag(&mut tags, normalize_token(&token));
        }
    }
    tags
}

fn detect_entity_tags(text: &str) -> EntityTags {
    let tokens: HashSet<String> = tokenize(text)
        .into_iter()
        .map(|token| normalize_token(&token).to_string())
        .collect();

    let mut tags = EntityTags::default();
    for token in tokens {
        mark_tag(&mut tags, &token);
    }
    tags
}

fn mark_tag(tags: &mut EntityTags, token: &str) {
    match token {
        "oracle" => tags.oracle = true,
        "postgresql" => tags.postgresql = true,
        "python" => tags.python = true,
        "listener" => tags.listener = true,
        "vacuum" | "autovacuum" => tags.vacuum = true,
        "connection" => tags.connection = true,
        "lsnrctl" => {
            tags.oracle = true;
            tags.listener = true;
        }
        "psycopg" | "psycopg2" => {
            tags.postgresql = true;
            tags.python = true;
            tags.connection = true;
        }
        _ => {}
    }
}

#[derive(Default)]
struct ScoreAdjustment {
    delta: f32,
    explanation: Option<&'static str>,
}

fn score_adjustment(query: &EntityTags, entry: &EntityTags) -> ScoreAdjustment {
    if query.has_exclusive_postgresql() {
        if entry.postgresql {
            return ScoreAdjustment {
                delta: 0.20,
                explanation: Some("boost: query targets postgresql"),
            };
        }
        if entry.oracle {
            let explanation = if query.python && query.connection {
                "penalty: python+postgresql connection query without oracle"
            } else {
                "penalty: query targets postgresql, entry is oracle"
            };
            return ScoreAdjustment {
                delta: -0.45,
                explanation: Some(explanation),
            };
        }
    }

    if query.has_exclusive_oracle() {
        if entry.oracle {
            return ScoreAdjustment {
                delta: 0.20,
                explanation: Some("boost: query targets oracle"),
            };
        }
        if entry.postgresql {
            return ScoreAdjustment {
                delta: -0.45,
                explanation: Some("penalty: query targets oracle, entry is postgresql"),
            };
        }
    }

    ScoreAdjustment {
        delta: 0.0,
        explanation: None,
    }
}

fn explain_score(base: String, tags: Option<&EntityTags>, adjustment: &ScoreAdjustment) -> String {
    let mut parts = vec![base];
    if let Some(tags) = tags {
        let tag_names = tags.names();
        if !tag_names.is_empty() {
            parts.push(format!("tags={}", tag_names.join(",")));
        }
    }
    if let Some(explanation) = adjustment.explanation {
        parts.push(format!("{explanation} ({:+.2})", adjustment.delta));
    }
    parts.join("; ")
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

fn clamp01(x: f32) -> f32 {
    x.clamp(0.0, 1.0)
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
        let tags = detect_entity_tags(&normalize_query_text("conexão python no postgress"));
        assert!(tags.postgresql);
        assert!(tags.python);
        assert!(tags.connection);
    }
}
