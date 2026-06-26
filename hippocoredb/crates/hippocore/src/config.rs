//! Configuration for opening a Hippocore database.

use std::path::PathBuf;

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
        }
    }
}
