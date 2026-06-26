//! Text processing helpers: a deterministic embedder and a token chunker.
//!
//! The embedder is intentionally local and reproducible — it uses signed
//! feature hashing so the same text always yields the same vector and texts
//! that share tokens have higher cosine similarity. This keeps tests
//! deterministic and removes any network dependency. Callers may always supply
//! their own embeddings instead.

/// A deterministic, network-free embedder based on signed feature hashing.
#[derive(Debug, Clone)]
pub struct Embedder {
    dim: usize,
}

impl Embedder {
    /// Create an embedder producing `dim`-dimensional vectors.
    pub fn new(dim: usize) -> Self {
        Self { dim: dim.max(1) }
    }

    /// Embedding dimensionality.
    pub fn dim(&self) -> usize {
        self.dim
    }

    /// Embed `text` into an L2-normalized vector. Text with no tokens yields an
    /// all-zero vector (which vector search skips).
    pub fn embed(&self, text: &str) -> Vec<f32> {
        let mut v = vec![0.0f32; self.dim];
        for token in tokenize(text) {
            let h = fnv1a(token.as_bytes());
            let idx = (h % self.dim as u64) as usize;
            // Use a separate bit of the hash for the sign to reduce collisions.
            let sign = if (h >> 32) & 1 == 0 { 1.0 } else { -1.0 };
            v[idx] += sign;
        }
        l2_normalize(&mut v);
        v
    }
}

/// Split `text` into chunks of at most `chunk_tokens` whitespace tokens,
/// preserving order. Returns at least one chunk for non-empty input.
pub fn chunk_text(text: &str, chunk_tokens: usize) -> Vec<String> {
    let chunk_tokens = chunk_tokens.max(1);
    let tokens: Vec<&str> = text.split_whitespace().collect();
    if tokens.is_empty() {
        return Vec::new();
    }
    tokens.chunks(chunk_tokens).map(|w| w.join(" ")).collect()
}

/// Lowercase alphanumeric tokenizer shared by the embedder and the text index.
pub fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase())
        .collect()
}

/// 64-bit FNV-1a hash — small, fast and dependency-free.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn l2_normalize(v: &mut [f32]) {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedding_is_deterministic() {
        let e = Embedder::new(32);
        assert_eq!(e.embed("hello world"), e.embed("hello world"));
    }

    #[test]
    fn shared_tokens_increase_similarity() {
        let e = Embedder::new(64);
        let a = e.embed("oracle connection error in homologation");
        let b = e.embed("oracle connection error in production");
        let c = e.embed("the cat slept on a warm windowsill");
        let sim = |x: &[f32], y: &[f32]| x.iter().zip(y).map(|(p, q)| p * q).sum::<f32>();
        assert!(sim(&a, &b) > sim(&a, &c));
    }

    #[test]
    fn empty_text_embeds_to_zero() {
        let e = Embedder::new(8);
        assert!(e.embed("   ").iter().all(|x| *x == 0.0));
    }

    #[test]
    fn chunking_splits_by_token_count() {
        let chunks = chunk_text("one two three four five", 2);
        assert_eq!(chunks, vec!["one two", "three four", "five"]);
    }

    #[test]
    fn chunking_empty_is_empty() {
        assert!(chunk_text("   ", 4).is_empty());
    }
}
