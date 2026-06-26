//! Hierarchical Navigable Small World (HNSW) approximate nearest-neighbour
//! index, implemented from scratch with no external dependencies.
//!
//! # Algorithm overview
//!
//! HNSW builds a multi-layer graph where:
//! - Layer 0 contains every node (densest graph).
//! - Each higher layer is a geometrically-thinned subset of the layer below.
//! - Insertion assigns a maximum layer `l` ∝ −ln(U[0,1)) / ln(M), so higher
//!   layers are exponentially sparser.
//! - Search descends from the top layer greedily and expands the candidate set
//!   at layer 0 using a beam of `ef_search` candidates.
//!
//! # Parameters
//!
//! - `M` — maximum bi-directional connections per node per layer (default 16).
//!   Higher M → better recall, higher memory.
//! - `M0` — max connections at layer 0 = 2 × M (as in the paper).
//! - `ef_construction` — candidate beam size during insertion (default 200).
//!   Higher → better graph quality, slower insertion.
//! - `ef_search` — candidate beam size during query (default 50).
//!   Higher → better recall, slower query. Can be set per-query.
//!
//! # Limitations (acceptable for MVP)
//!
//! - No deletion: removal marks a node as deleted and skips it in search.
//!   Rebuild on compaction if tombstone fraction exceeds a threshold.
//! - Cosine distance (1 − cosine_similarity). Vectors must be non-zero.
//! - Single-threaded; lock-free parallelism is a future enhancement.

use std::collections::{BinaryHeap, HashMap, HashSet};

use crate::index::{cosine_similarity, EntryId};

/// Default `M` for HNSW (bi-directional connections per layer per node).
pub const HNSW_M: usize = 16;
/// Default beam size during construction.
pub const HNSW_EF_CONSTRUCTION: usize = 200;
/// Default beam size during search.
pub const HNSW_EF_SEARCH: usize = 50;

// A node in the graph.
#[derive(Debug, Clone)]
struct Node {
    key: EntryId,
    embedding: Vec<f32>,
    // Per-layer neighbour lists. `layers[0]` is layer-0 neighbours.
    layers: Vec<Vec<usize>>,
    deleted: bool,
}

/// Scored candidate used in the results `BinaryHeap` (max-heap by distance so
/// `peek` / `pop` surface the farthest item for pruning).
#[derive(Clone, PartialEq)]
struct Candidate {
    dist: f32,
    idx: usize,
}

impl Eq for Candidate {}
impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
// Max-heap: larger dist = higher priority = peeked/popped first.
impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.dist
            .partial_cmp(&other.dist)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

// Min-heap candidate: used in the working set W (closest at top).
// Wrapped in std::cmp::Reverse<CandMin> for BinaryHeap min-heap semantics.
#[derive(Clone, PartialEq)]
struct CandMin {
    dist: f32,
    idx: usize,
}
impl Eq for CandMin {}
impl PartialOrd for CandMin {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for CandMin {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.dist
            .partial_cmp(&other.dist)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

/// An HNSW approximate nearest-neighbour index over fixed-dimension vectors.
#[derive(Debug)]
pub struct HnswIndex {
    nodes: Vec<Node>,
    key_to_idx: HashMap<EntryId, usize>,
    entry_point: Option<usize>, // node index of current entry point (top layer)
    top_layer: usize,
    m: usize,
    m0: usize, // max connections at layer 0
    ef_construction: usize,
    ml: f64, // layer assignment factor = 1 / ln(m)
    deleted_count: usize,
}

impl Default for HnswIndex {
    fn default() -> Self {
        Self::new(HNSW_M, HNSW_EF_CONSTRUCTION)
    }
}

impl HnswIndex {
    /// Create a new HNSW index with the given parameters.
    pub fn new(m: usize, ef_construction: usize) -> Self {
        let m = m.max(2);
        Self {
            nodes: Vec::new(),
            key_to_idx: HashMap::new(),
            entry_point: None,
            top_layer: 0,
            m,
            m0: 2 * m,
            ef_construction,
            ml: 1.0 / (m as f64).ln(),
            deleted_count: 0,
        }
    }

    /// Number of non-deleted nodes.
    pub fn len(&self) -> usize {
        self.nodes.len() - self.deleted_count
    }

    /// Whether the index is empty (no live nodes).
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Insert or update a node. If the key already exists, removes the old
    /// node first (marks deleted, inserts fresh).
    pub fn insert(&mut self, key: EntryId, embedding: Vec<f32>) {
        if embedding.is_empty() {
            return; // un-embedded entries are not vector-indexed
        }
        if self.key_to_idx.contains_key(&key) {
            self.remove(&key);
        }
        let level = self.random_level();
        let new_idx = self.nodes.len();

        // Allocate per-layer neighbour lists.
        let layers = vec![Vec::new(); level + 1];
        self.nodes.push(Node {
            key: key.clone(),
            embedding,
            layers,
            deleted: false,
        });
        self.key_to_idx.insert(key, new_idx);

        let ep = match self.entry_point {
            None => {
                // First node.
                self.entry_point = Some(new_idx);
                self.top_layer = level;
                return;
            }
            Some(ep) => ep,
        };

        // Phase 1: descend to level+1, following greedy closest neighbour.
        let mut cur_ep = ep;
        for lc in (level + 1..=self.top_layer).rev() {
            let candidates =
                self.search_layer(&self.nodes[new_idx].embedding.clone(), cur_ep, 1, lc);
            if let Some(best) = candidates.first() {
                cur_ep = best.idx;
            }
        }

        // Phase 2: at each layer lc ∈ [min(level, top_layer)..=0], find ef_construction
        // nearest neighbours and connect bidirectionally.
        for lc in (0..=level.min(self.top_layer)).rev() {
            let q_emb = self.nodes[new_idx].embedding.clone();
            let candidates = self.search_layer(&q_emb, cur_ep, self.ef_construction, lc);
            let m_max = if lc == 0 { self.m0 } else { self.m };
            let neighbours: Vec<usize> = candidates.iter().take(m_max).map(|c| c.idx).collect();

            // Connect new_idx → neighbours and neighbours → new_idx.
            self.nodes[new_idx].layers[lc] = neighbours.clone();
            for &nb in &neighbours {
                let nb_m_max = if lc == 0 { self.m0 } else { self.m };
                self.nodes[nb].layers[lc].push(new_idx);
                // Prune if over-connected.
                if self.nodes[nb].layers[lc].len() > nb_m_max {
                    let nb_emb = self.nodes[nb].embedding.clone();
                    let pruned = self.select_neighbours(
                        &nb_emb,
                        &self.nodes[nb].layers[lc].clone(),
                        nb_m_max,
                    );
                    self.nodes[nb].layers[lc] = pruned;
                }
            }

            if let Some(best) = candidates.first() {
                cur_ep = best.idx;
            }
        }

        if level > self.top_layer {
            self.top_layer = level;
            self.entry_point = Some(new_idx);
        }
    }

    /// Mark a node as deleted (soft delete; space is not reclaimed).
    pub fn remove(&mut self, key: &EntryId) {
        if let Some(&idx) = self.key_to_idx.get(key) {
            if !self.nodes[idx].deleted {
                self.nodes[idx].deleted = true;
                self.deleted_count += 1;
            }
            self.key_to_idx.remove(key);

            // If we deleted the entry point, find a new one.
            if self.entry_point == Some(idx) {
                self.entry_point = self
                    .nodes
                    .iter()
                    .enumerate()
                    .rev()
                    .find(|(_, n)| !n.deleted)
                    .map(|(i, _)| i);
                self.top_layer = self
                    .entry_point
                    .map(|ep| self.nodes[ep].layers.len() - 1)
                    .unwrap_or(0);
            }
        }
    }

    /// Return up to `k` nearest neighbour keys (excluding deleted nodes) for
    /// `query`. Returns an empty vec when the index is empty or the query is
    /// the zero vector.
    pub fn search(&self, query: &[f32], k: usize, ef: usize) -> Vec<EntryId> {
        if self.is_empty() || query.is_empty() {
            return Vec::new();
        }
        let ep = match self.entry_point {
            Some(ep) => ep,
            None => return Vec::new(),
        };
        let ef = ef.max(k);

        // Greedy descent to layer 1.
        let mut cur_ep = ep;
        for lc in (1..=self.top_layer).rev() {
            let candidates = self.search_layer(query, cur_ep, 1, lc);
            if let Some(best) = candidates.first() {
                cur_ep = best.idx;
            }
        }

        // Full search at layer 0.
        let candidates = self.search_layer(query, cur_ep, ef, 0);
        candidates
            .into_iter()
            .take(k)
            .map(|c| self.nodes[c.idx].key.clone())
            .collect()
    }

    // Search a single layer: standard two-heap HNSW beam search.
    //
    // `W` = min-heap of unprocessed candidates (closest at top).
    // `C` = max-heap of best ef results so far (farthest at top, for pruning).
    //
    // Returns up to `ef` nearest non-deleted nodes sorted by ascending distance.
    fn search_layer(&self, query: &[f32], ep: usize, ef: usize, layer: usize) -> Vec<Candidate> {
        if self.nodes.is_empty() {
            return Vec::new();
        }
        let ep = if self.nodes[ep].deleted {
            match self.nodes.iter().enumerate().find(|(_, n)| !n.deleted) {
                Some((i, _)) => i,
                None => return Vec::new(),
            }
        } else {
            ep
        };

        let ep_dist = self.dist(query, ep);
        let mut visited: HashSet<usize> = HashSet::new();
        visited.insert(ep);

        // W: min-heap (closest candidate first) — use Reverse<> for a min-heap.
        let mut w: BinaryHeap<std::cmp::Reverse<CandMin>> =
            BinaryHeap::from([std::cmp::Reverse(CandMin {
                dist: ep_dist,
                idx: ep,
            })]);
        // C: max-heap (farthest result first) — normal Candidate Ord (max by dist).
        let mut c: BinaryHeap<Candidate> = BinaryHeap::new();
        if !self.nodes[ep].deleted {
            c.push(Candidate {
                dist: ep_dist,
                idx: ep,
            });
        }

        while let Some(std::cmp::Reverse(closest)) = w.pop() {
            // Early termination: if the closest unexplored is farther than the
            // worst result and we have enough results, no improvement is possible.
            if c.len() >= ef {
                if let Some(farthest) = c.peek() {
                    if closest.dist > farthest.dist {
                        break;
                    }
                }
            }

            // Expand neighbours.
            if layer < self.nodes[closest.idx].layers.len() {
                for &nb in self.nodes[closest.idx].layers[layer].iter() {
                    if !visited.insert(nb) {
                        continue;
                    }
                    let nb_dist = self.dist(query, nb);
                    let add = c.len() < ef || c.peek().map(|f| nb_dist < f.dist).unwrap_or(true);
                    if add {
                        w.push(std::cmp::Reverse(CandMin {
                            dist: nb_dist,
                            idx: nb,
                        }));
                        if !self.nodes[nb].deleted {
                            c.push(Candidate {
                                dist: nb_dist,
                                idx: nb,
                            });
                            if c.len() > ef {
                                c.pop();
                            }
                        }
                    }
                }
            }
        }

        let mut out: Vec<Candidate> = c
            .into_iter()
            .filter(|item| !self.nodes[item.idx].deleted)
            .collect();
        out.sort_by(|a, b| {
            a.dist
                .partial_cmp(&b.dist)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        out
    }

    // Select at most `m` neighbours from `candidates` by simple greedy heuristic
    // (closest first). The full SELECT-NEIGHBORS-HEURISTIC from the paper is
    // optional; this simpler version works well in practice.
    fn select_neighbours(&self, _query: &[f32], candidates: &[usize], m: usize) -> Vec<usize> {
        let mut scored: Vec<(f32, usize)> = candidates
            .iter()
            .filter(|&&i| !self.nodes[i].deleted)
            .map(|&i| {
                let d = if !self.nodes[i].embedding.is_empty() {
                    1.0 - cosine_similarity(_query, &self.nodes[i].embedding).unwrap_or(0.0)
                } else {
                    1.0
                };
                (d, i)
            })
            .collect();
        scored.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(m).map(|(_, i)| i).collect()
    }

    // Cosine distance (1 − similarity) between `query` and node `idx`.
    fn dist(&self, query: &[f32], idx: usize) -> f32 {
        if self.nodes[idx].embedding.is_empty() {
            return 1.0;
        }
        1.0 - cosine_similarity(query, &self.nodes[idx].embedding).unwrap_or(0.0)
    }

    // Sample a max layer using the HNSW geometric distribution.
    //
    // level = floor(-ln(U) * ml) where U ~ Uniform(0,1) and ml = 1/ln(M).
    // For M=16 this gives level=0 with probability ≈ 94%, level=1 with ≈ 6%,
    // level=2 with ≈ 0.4%, etc. — as in the original HNSW paper.
    fn random_level(&self) -> usize {
        // Deterministic LCG seeded by the current node count. Two mixing steps
        // improve distribution when the seed is small.
        let seed = self.nodes.len() as u64;
        let mut x = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        x = x
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        // Map to (0, 1) — shift and add 1 to avoid u=0 → ln(0) = -∞.
        let u = ((x >> 11) as f64 + 1.0) / ((1u64 << 53) as f64 + 1.0);
        let level = (-u.ln() * self.ml).floor() as usize;
        level.min(16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ItemKind;

    fn make_key(n: u32) -> EntryId {
        (ItemKind::Memory, format!("t\0c\0m{n}"))
    }

    fn unit(n: u32, dim: usize) -> Vec<f32> {
        let mut v = vec![0.0f32; dim];
        let i = (n as usize) % dim;
        v[i] = 1.0;
        v
    }

    #[test]
    fn hnsw_insert_and_search_basic() {
        let mut idx = HnswIndex::default();
        for i in 0..20u32 {
            idx.insert(make_key(i), unit(i, 32));
        }
        assert!(idx.len() >= 15, "most nodes should survive insert");

        // Query with vector matching key 0.
        let results = idx.search(&unit(0, 32), 5, HNSW_EF_SEARCH);
        assert!(!results.is_empty(), "should return at least one result");
        assert!(
            results.contains(&make_key(0)),
            "exact match must be in results (got {results:?})"
        );
    }

    #[test]
    fn hnsw_remove_marks_deleted() {
        let mut idx = HnswIndex::default();
        for i in 0..10u32 {
            idx.insert(make_key(i), unit(i, 8));
        }
        let before = idx.len();
        idx.remove(&make_key(0));
        assert_eq!(idx.len(), before - 1);

        let results = idx.search(&unit(0, 8), 10, HNSW_EF_SEARCH);
        assert!(
            !results.contains(&make_key(0)),
            "deleted node must not appear in search"
        );
    }

    #[test]
    fn hnsw_empty_vector_skipped() {
        let mut idx = HnswIndex::default();
        idx.insert(make_key(0), vec![]);
        assert_eq!(idx.len(), 0, "empty-embedding nodes must be skipped");
    }

    #[test]
    fn hnsw_search_empty_index_returns_empty() {
        let idx = HnswIndex::default();
        assert!(idx.search(&[1.0, 0.0], 5, HNSW_EF_SEARCH).is_empty());
    }
}
