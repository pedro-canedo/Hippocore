//! Configuration for opening a Hippocore database.

use std::path::PathBuf;

use crate::index::VectorIndexKind;

/// Default embedding dimensionality used by the built-in deterministic embedder.
pub const DEFAULT_EMBEDDING_DIM: usize = 64;

/// Default chunk size, in whitespace-separated tokens.
pub const DEFAULT_CHUNK_TOKENS: usize = 48;

/// Default hybrid fusion weight: how much vector score counts vs. text score.
pub const DEFAULT_HYBRID_ALPHA: f32 = 0.5;

/// Default auto-compaction threshold in WAL operations (0 disables).
pub const DEFAULT_AUTO_COMPACT_OPS: usize = 1000;

/// Default auto-compaction threshold in WAL bytes (0 disables).
pub const DEFAULT_AUTO_COMPACT_BYTES: u64 = 8 * 1024 * 1024;

/// Default audit log max records kept (0 = unlimited, no auto-retention).
pub const DEFAULT_AUDIT_MAX_RECORDS: usize = 0;

/// Default audit log max bytes kept (0 = unlimited, no auto-retention).
pub const DEFAULT_AUDIT_MAX_BYTES: u64 = 0;

/// Options controlling how a database is opened and persists data.
#[derive(Debug, Clone)]
pub struct Config {
    /// Directory holding the data files. Created on [`crate::Hippocore::open`].
    pub data_dir: PathBuf,
    /// Dimensionality of embeddings produced by the built-in embedder.
    pub embedding_dim: usize,
    /// Chunk size in tokens used when splitting document text.
    pub chunk_tokens: usize,
    /// Hybrid fusion weight in `[0.0, 1.0]`; `1.0` = pure vector, `0.0` = text.
    pub hybrid_alpha: f32,
    /// When `true` (default) every WAL append is `fsync`ed before returning.
    pub sync_writes: bool,
    /// Auto-compact once the WAL holds this many operations (`0` disables).
    pub auto_compact_after_ops: usize,
    /// Auto-compact once the WAL reaches this many bytes (`0` disables).
    pub auto_compact_after_bytes: u64,
    /// Vector index backend. Default is brute-force (exact, O(n) per query).
    /// Set to `VectorIndexKind::Hnsw` for approximate sub-linear search over
    /// large collections (trades a small recall penalty for speed).
    pub vector_index: VectorIndexKind,
    /// Keep at most this many records in `audit.log` (`0` = unlimited).
    /// Enforced automatically after every `build_context` append when non-zero.
    pub audit_max_records: usize,
    /// Keep at most this many bytes in `audit.log` (`0` = unlimited).
    /// Enforced automatically after every `build_context` append when non-zero.
    pub audit_max_bytes: u64,
    /// Weight in `[0.0, 1.0]` for graph-connectivity bonus in `build_context`.
    ///
    /// When non-zero, candidates whose direct graph neighbours also appear in the
    /// recalled candidate set receive a small score boost:
    /// `effective = recall_score * (1 - w) + connectivity_score * w`.
    /// `0.0` (default) disables the bonus; ordering is identical to plain recall.
    pub graph_rank_weight: f32,
}

impl Config {
    /// Build a config for `data_dir` with all other fields defaulted.
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
            ..Self::default()
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("./hippocore-data"),
            embedding_dim: DEFAULT_EMBEDDING_DIM,
            chunk_tokens: DEFAULT_CHUNK_TOKENS,
            hybrid_alpha: DEFAULT_HYBRID_ALPHA,
            sync_writes: true,
            auto_compact_after_ops: DEFAULT_AUTO_COMPACT_OPS,
            auto_compact_after_bytes: DEFAULT_AUTO_COMPACT_BYTES,
            vector_index: VectorIndexKind::BruteForce,
            audit_max_records: DEFAULT_AUDIT_MAX_RECORDS,
            audit_max_bytes: DEFAULT_AUDIT_MAX_BYTES,
            graph_rank_weight: 0.0,
        }
    }
}
