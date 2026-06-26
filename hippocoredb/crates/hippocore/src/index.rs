//! In-memory retrieval indexes: a pluggable vector index and an inverted text
//! index, both rebuilt from the persisted [`crate::storage::State`] on open and
//! updated incrementally on each write.
//!
//! The vector backend is selected via [`VectorIndexKind`]: either the exact
//! brute-force implementation or the HNSW approximate index.

use std::collections::{HashMap, HashSet};

use crate::hnsw::{HnswIndex, HNSW_EF_CONSTRUCTION, HNSW_EF_SEARCH, HNSW_M};
use crate::memory::tokenize;
use crate::model::{Chunk, ItemKind, Memory, MemoryType, Metadata, Record, Source};

// ── VectorIndex trait ────────────────────────────────────────────────────────

/// Which ANN backend to use when opening or building an [`Index`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VectorIndexKind {
    /// Exact brute-force cosine scan (O(n)). Default; correct for any workload size.
    #[default]
    BruteForce,
    /// HNSW approximate nearest-neighbour index. Sub-linear query time; slightly
    /// lower recall than brute force at very small n.
    Hnsw,
}

/// Pluggable vector backend used by [`Index`].
///
/// Implementors maintain their own map from [`EntryId`] to embedding vectors and
/// answer approximate (or exact) k-nearest-neighbour queries. The [`Index`]
/// drives insertions, removals and queries through this trait.
pub trait VectorIndex: std::fmt::Debug + Send + Sync {
    /// Insert or update an entry. No-op when `embedding` is empty.
    fn insert(&mut self, key: EntryId, embedding: Vec<f32>);
    /// Remove an entry by key. No-op if the key is absent.
    fn remove(&mut self, key: &EntryId);
    /// Return up to `k` nearest neighbour keys for `query`, using the given
    /// `ef` (beam size). For brute force `ef` is ignored.
    fn knn(&self, query: &[f32], k: usize, ef: usize) -> Vec<EntryId>;
}

// ── Brute-force backend ──────────────────────────────────────────────────────

/// Exact cosine brute-force vector backend. O(n) per query; precise.
#[derive(Debug, Default)]
pub struct BruteForceVectorIndex {
    entries: HashMap<EntryId, Vec<f32>>,
}

impl VectorIndex for BruteForceVectorIndex {
    fn insert(&mut self, key: EntryId, embedding: Vec<f32>) {
        if !embedding.is_empty() {
            self.entries.insert(key, embedding);
        }
    }

    fn remove(&mut self, key: &EntryId) {
        self.entries.remove(key);
    }

    fn knn(&self, query: &[f32], k: usize, _ef: usize) -> Vec<EntryId> {
        if query.is_empty() {
            return Vec::new();
        }
        let mut scored: Vec<(f32, &EntryId)> = self
            .entries
            .iter()
            .filter_map(|(key, emb)| cosine_similarity(query, emb).map(|s| (s, key)))
            .collect();
        // Descending by similarity.
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(k).map(|(_, k)| k.clone()).collect()
    }
}

// ── HNSW backend ─────────────────────────────────────────────────────────────

/// HNSW approximate nearest-neighbour backend. Sub-linear query time.
#[derive(Debug)]
pub struct HnswVectorIndex(HnswIndex);

impl Default for HnswVectorIndex {
    fn default() -> Self {
        Self(HnswIndex::new(HNSW_M, HNSW_EF_CONSTRUCTION))
    }
}

impl VectorIndex for HnswVectorIndex {
    fn insert(&mut self, key: EntryId, embedding: Vec<f32>) {
        self.0.insert(key, embedding);
    }

    fn remove(&mut self, key: &EntryId) {
        self.0.remove(key);
    }

    fn knn(&self, query: &[f32], k: usize, ef: usize) -> Vec<EntryId> {
        self.0.search(query, k, ef.max(HNSW_EF_SEARCH))
    }
}

/// Stable identity of an indexed entry (kind disambiguates id collisions).
pub type EntryId = (ItemKind, String);

/// A unified, searchable entry covering chunks, memories and record projections.
#[derive(Debug, Clone)]
pub struct IndexEntry {
    /// Underlying chunk/memory/record id.
    pub id: String,
    /// Whether this entry is a document chunk, memory or record.
    pub kind: ItemKind,
    /// Owning tenant.
    pub tenant_id: String,
    /// Owning collection.
    pub collection: String,
    /// Parent document id (for chunks).
    pub document_id: Option<String>,
    /// Table name (for structured records).
    pub record_table: Option<String>,
    /// Owning user (for memories).
    pub user_id: Option<String>,
    /// Memory type (for memories).
    pub memory_type: Option<MemoryType>,
    /// Text content.
    pub text: String,
    /// Embedding vector.
    pub embedding: Vec<f32>,
    /// Exact-match metadata.
    pub metadata: Metadata,
    /// Provenance.
    pub source: Option<Source>,
    /// Term frequencies for text scoring.
    pub tf: HashMap<String, u32>,
    /// Total token count of `text` (document length for BM25 normalization).
    pub token_count: u32,
    /// Validity start (epoch ms), inherited from the parent item.
    pub valid_from: Option<i64>,
    /// Validity end (epoch ms), inherited from the parent item.
    pub valid_until: Option<i64>,
    /// True when this memory has been superseded by another. Excluded from
    /// default recall unless `include_superseded` is set on the filter.
    pub superseded: bool,
    /// Ids of memories this entry contradicts (advisory).
    pub contradicts: Vec<String>,
    /// Caller-assigned confidence in `[0.0, 1.0]`, or `None` if unrated.
    pub confidence: Option<f32>,
}

impl IndexEntry {
    fn key(&self) -> EntryId {
        let id = match (self.kind, self.record_table.as_deref()) {
            (ItemKind::Record, Some(table)) => {
                format!(
                    "{}\0{}\0{table}\0{}",
                    self.tenant_id, self.collection, self.id
                )
            }
            _ => format!("{}\0{}\0{}", self.tenant_id, self.collection, self.id),
        };
        (self.kind, id)
    }

    /// Build an entry from a document chunk plus its parent's metadata/source.
    pub fn from_chunk(
        chunk: &Chunk,
        metadata: Metadata,
        source: Option<Source>,
        valid_from: Option<i64>,
        valid_until: Option<i64>,
    ) -> Self {
        let tf = term_frequencies(&chunk.text);
        let token_count = tf.values().sum();
        Self {
            id: chunk.id.clone(),
            kind: ItemKind::DocumentChunk,
            tenant_id: chunk.tenant_id.clone(),
            collection: chunk.collection.clone(),
            document_id: Some(chunk.document_id.clone()),
            record_table: None,
            user_id: None,
            memory_type: None,
            text: chunk.text.clone(),
            embedding: chunk.embedding.clone(),
            metadata,
            source,
            tf,
            token_count,
            valid_from,
            valid_until,
            superseded: false,
            contradicts: Vec::new(),
            confidence: None,
        }
    }

    /// Build an entry from a memory.
    pub fn from_memory(memory: &Memory) -> Self {
        let tf = term_frequencies(&memory.text);
        let token_count = tf.values().sum();
        Self {
            id: memory.id.clone(),
            kind: ItemKind::Memory,
            tenant_id: memory.tenant_id.clone(),
            collection: memory.collection.clone(),
            document_id: None,
            record_table: None,
            user_id: memory.user_id.clone(),
            memory_type: Some(memory.memory_type),
            text: memory.text.clone(),
            embedding: memory.embedding.clone(),
            metadata: memory.metadata.clone(),
            source: memory.source.clone(),
            tf,
            token_count,
            valid_from: memory.valid_from,
            valid_until: memory.valid_until,
            superseded: memory.superseded_by.is_some(),
            contradicts: memory.contradicts.clone(),
            confidence: memory.confidence,
        }
    }

    /// Build an entry from a structured record's context projection.
    pub fn from_record(record: &Record) -> Self {
        let tf = term_frequencies(&record.projection);
        let token_count = tf.values().sum();
        Self {
            id: record.id.clone(),
            kind: ItemKind::Record,
            tenant_id: record.tenant_id.clone(),
            collection: record.collection.clone(),
            document_id: None,
            record_table: Some(record.table.clone()),
            user_id: None,
            memory_type: None,
            text: record.projection.clone(),
            embedding: record.embedding.clone(),
            metadata: record.metadata.clone(),
            source: record.source.clone(),
            tf,
            token_count,
            valid_from: None,
            valid_until: None,
            superseded: false,
            contradicts: Vec::new(),
            confidence: None,
        }
    }
}

/// BM25 term-frequency saturation parameter.
pub const BM25_K1: f32 = 1.2;
/// BM25 document-length normalization parameter.
pub const BM25_B: f32 = 0.75;

/// The in-memory index over all searchable entries.
///
/// The text (BM25 inverted) index is always exact. The vector backend is
/// pluggable via [`VectorIndexKind`] — pass it to [`Index::with_vector_backend`]
/// when opening the database; the default is brute-force.
#[derive(Debug)]
pub struct Index {
    entries: HashMap<EntryId, IndexEntry>,
    /// token -> set of entry ids containing it (inverted index).
    postings: HashMap<String, HashSet<EntryId>>,
    /// Sum of all entries' token counts (for the BM25 average document length).
    total_tokens: u64,
    /// Pluggable vector backend.
    vector: Box<dyn VectorIndex>,
}

impl Default for Index {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
            postings: HashMap::new(),
            total_tokens: 0,
            vector: Box::new(BruteForceVectorIndex::default()),
        }
    }
}

impl Index {
    /// Create an empty index with the brute-force vector backend (default).
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an empty index using the specified vector backend.
    pub fn with_vector_backend(kind: VectorIndexKind) -> Self {
        let vector: Box<dyn VectorIndex> = match kind {
            VectorIndexKind::BruteForce => Box::new(BruteForceVectorIndex::default()),
            VectorIndexKind::Hnsw => Box::new(HnswVectorIndex::default()),
        };
        Self {
            entries: HashMap::new(),
            postings: HashMap::new(),
            total_tokens: 0,
            vector,
        }
    }

    /// Insert or replace an entry, maintaining the inverted index and the
    /// running token total used for BM25 length normalization.
    pub fn insert(&mut self, entry: IndexEntry) {
        let key = entry.key();
        self.remove(&key);
        for token in entry.tf.keys() {
            self.postings
                .entry(token.clone())
                .or_default()
                .insert(key.clone());
        }
        self.total_tokens += entry.token_count as u64;
        self.vector.insert(key.clone(), entry.embedding.clone());
        self.entries.insert(key, entry);
    }

    /// Remove an entry by key, cleaning up its postings, token total, and
    /// vector backend entry.
    pub fn remove(&mut self, key: &EntryId) {
        if let Some(old) = self.entries.remove(key) {
            self.total_tokens = self.total_tokens.saturating_sub(old.token_count as u64);
            for token in old.tf.keys() {
                if let Some(set) = self.postings.get_mut(token) {
                    set.remove(key);
                    if set.is_empty() {
                        self.postings.remove(token);
                    }
                }
            }
        }
        self.vector.remove(key);
    }

    /// Remove every chunk entry belonging to `document_id`.
    pub fn remove_document(&mut self, tenant_id: &str, collection: &str, document_id: &str) {
        let keys: Vec<EntryId> = self
            .entries
            .values()
            .filter(|e| {
                e.tenant_id == tenant_id
                    && e.collection == collection
                    && e.document_id.as_deref() == Some(document_id)
            })
            .map(IndexEntry::key)
            .collect();
        for k in keys {
            self.remove(&k);
        }
    }

    /// Remove one memory entry by identity.
    pub fn remove_memory(&mut self, tenant_id: &str, collection: &str, id: &str) {
        self.remove(&(ItemKind::Memory, format!("{tenant_id}\0{collection}\0{id}")));
    }

    /// Remove one structured record entry by table and record id.
    pub fn remove_record(&mut self, tenant_id: &str, collection: &str, table: &str, id: &str) {
        self.remove(&(
            ItemKind::Record,
            format!("{tenant_id}\0{collection}\0{table}\0{id}"),
        ));
    }

    /// Number of indexed entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the index is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// All entries (used by brute-force vector scan).
    pub fn entries(&self) -> impl Iterator<Item = &IndexEntry> {
        self.entries.values()
    }

    /// Look up a single entry.
    pub fn get(&self, key: &EntryId) -> Option<&IndexEntry> {
        self.entries.get(key)
    }

    /// Candidate entry ids containing at least one of `query_tokens`.
    pub fn text_candidates(&self, query_tokens: &[String]) -> HashSet<EntryId> {
        let mut out = HashSet::new();
        for t in query_tokens {
            if let Some(set) = self.postings.get(t) {
                out.extend(set.iter().cloned());
            }
        }
        out
    }

    /// Return up to `k` nearest-neighbour entry ids for `query`, using the
    /// configured vector backend (brute-force or HNSW).
    ///
    /// The caller is responsible for applying post-filters (tenant, temporal,
    /// supersedure) against the returned ids via [`Index::get`].
    pub fn knn(&self, query: &[f32], k: usize, ef: usize) -> Vec<EntryId> {
        self.vector.knn(query, k, ef)
    }

    /// Document frequency: how many entries contain `token`.
    pub fn df(&self, token: &str) -> usize {
        self.postings.get(token).map(|s| s.len()).unwrap_or(0)
    }

    /// Average document length (in tokens) across the index; `0.0` when empty.
    pub fn avg_doc_len(&self) -> f32 {
        let n = self.entries.len();
        if n == 0 {
            0.0
        } else {
            self.total_tokens as f32 / n as f32
        }
    }

    /// BM25 inverse document frequency of a token.
    fn bm25_idf(&self, token: &str) -> f32 {
        let n = self.entries.len() as f32;
        let df = self.df(token) as f32;
        // Standard BM25 idf with +1 to keep it non-negative.
        (1.0 + (n - df + 0.5) / (df + 0.5)).ln()
    }
}

/// Cosine similarity between two equal-length, non-zero vectors.
///
/// Returns `None` on empty/mismatched/zero-magnitude input so callers can skip
/// rather than panic.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> Option<f32> {
    if a.is_empty() || a.len() != b.len() {
        return None;
    }
    let mut dot = 0.0f32;
    let mut na = 0.0f32;
    let mut nb = 0.0f32;
    for (&x, &y) in a.iter().zip(b.iter()) {
        dot += x * y;
        na += x * x;
        nb += y * y;
    }
    if na == 0.0 || nb == 0.0 {
        return None;
    }
    Some(dot / (na.sqrt() * nb.sqrt()))
}

/// Okapi BM25 text score of an entry for the given query tokens.
///
/// BM25 adds term-frequency saturation (`k1`) and document-length normalization
/// (`b`) over plain TF-IDF, so a long entry that merely mentions a term once
/// among many tokens does not out-rank a short, focused one, and repeating a
/// term yields diminishing returns.
pub fn text_score(index: &Index, entry: &IndexEntry, query_tokens: &[String]) -> f32 {
    let avgdl = index.avg_doc_len();
    let avgdl = if avgdl > 0.0 { avgdl } else { 1.0 };
    let dl = entry.token_count as f32;

    let mut score = 0.0f32;
    let mut seen = HashSet::new();
    for t in query_tokens {
        if !seen.insert(t) {
            continue; // count each distinct query token once
        }
        if let Some(&tf) = entry.tf.get(t) {
            let f = tf as f32;
            let idf = index.bm25_idf(t);
            let denom = f + BM25_K1 * (1.0 - BM25_B + BM25_B * dl / avgdl);
            score += idf * (f * (BM25_K1 + 1.0)) / denom;
        }
    }
    score
}

fn term_frequencies(text: &str) -> HashMap<String, u32> {
    let mut tf = HashMap::new();
    for token in tokenize(text) {
        *tf.entry(token).or_insert(0) += 1;
    }
    tf
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn cosine_identical_and_orthogonal() {
        assert!(approx(
            cosine_similarity(&[1.0, 2.0], &[1.0, 2.0]).unwrap(),
            1.0
        ));
        assert!(approx(
            cosine_similarity(&[1.0, 0.0], &[0.0, 1.0]).unwrap(),
            0.0
        ));
    }

    #[test]
    fn cosine_handles_bad_input() {
        assert!(cosine_similarity(&[], &[]).is_none());
        assert!(cosine_similarity(&[1.0], &[1.0, 2.0]).is_none());
        assert!(cosine_similarity(&[0.0, 0.0], &[1.0, 1.0]).is_none());
    }

    fn mem_entry(id: &str, text: &str) -> IndexEntry {
        let m = Memory {
            id: id.to_string(),
            tenant_id: "t".into(),
            collection: "c".into(),
            user_id: None,
            memory_type: MemoryType::Note,
            text: text.to_string(),
            embedding: Vec::new(),
            metadata: HashMap::new(),
            source: None,
            created_at: 0,
            valid_from: None,
            valid_until: None,
            supersedes: Vec::new(),
            contradicts: Vec::new(),
            superseded_by: None,
            confidence: None,
        };
        IndexEntry::from_memory(&m)
    }

    fn score(idx: &Index, id: &str, query: &[&str]) -> f32 {
        let key = (ItemKind::Memory, format!("t\0c\0{id}"));
        let tokens: Vec<String> = query.iter().map(|s| s.to_string()).collect();
        text_score(idx, idx.get(&key).unwrap(), &tokens)
    }

    #[test]
    fn bm25_length_normalization_favors_focused_entry() {
        let mut idx = Index::new();
        idx.insert(mem_entry("short", "oracle"));
        idx.insert(mem_entry(
            "long",
            "oracle alpha beta gamma delta epsilon zeta eta theta iota",
        ));
        // Both contain "oracle" once, so only length normalization differs.
        let s_short = score(&idx, "short", &["oracle"]);
        let s_long = score(&idx, "long", &["oracle"]);
        assert!(s_short > s_long, "short={s_short} long={s_long}");
    }

    #[test]
    fn bm25_saturates_repeated_terms() {
        let mut idx = Index::new();
        idx.insert(mem_entry("single", "oracle"));
        idx.insert(mem_entry("double", "oracle oracle"));
        let single = score(&idx, "single", &["oracle"]);
        let double = score(&idx, "double", &["oracle"]);
        assert!(double > single, "more occurrences should score higher");
        assert!(
            double < 2.0 * single,
            "tf saturation: doubling tf must less-than-double the score"
        );
    }

    #[test]
    fn avg_doc_len_tracks_inserts_and_removes() {
        let mut idx = Index::new();
        idx.insert(mem_entry("a", "one two")); // 2 tokens
        idx.insert(mem_entry("b", "one two three four")); // 4 tokens
        assert!(approx(idx.avg_doc_len(), 3.0));
        idx.remove_memory("t", "c", "b");
        assert!(approx(idx.avg_doc_len(), 2.0));
    }
}
