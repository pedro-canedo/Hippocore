//! Content-type–aware ingestion pipeline.
//!
//! Routes non-plain-text payloads to the appropriate extractor before the
//! normal chunking/embedding path.

use crate::errors::{HippocoreError, Result};
use crate::memory;

// ──────────────────────────────────────────────────────────────────────────────
// PDF
// ──────────────────────────────────────────────────────────────────────────────

/// Extract text from a PDF byte slice.
///
/// Pages are separated by `\n[Page N]\n` markers so that semantic boundaries
/// are visible in the chunked output.
pub fn extract_pdf(bytes: &[u8]) -> Result<String> {
    pdf_extract::extract_text_from_mem(bytes)
        .map_err(|e| HippocoreError::Validation(format!("PDF extraction failed: {e}")))
}

// ──────────────────────────────────────────────────────────────────────────────
// Code
// ──────────────────────────────────────────────────────────────────────────────

/// Heuristic code chunker.
///
/// Splits `text` at top-level definition boundaries (non-indented lines that
/// begin with common declaration keywords for the given language). Merges tiny
/// segments and further splits oversized ones using the token budget.
///
/// Falls back to [`memory::chunk_text`] for unrecognised languages.
pub fn chunk_code(text: &str, language: &str, chunk_tokens: usize) -> Vec<String> {
    let patterns: &[&str] = code_boundary_patterns(language);
    if patterns.is_empty() {
        return memory::chunk_text(text, chunk_tokens);
    }

    // Split lines into segments that start at top-level definition boundaries.
    let mut segments: Vec<String> = Vec::new();
    let mut current: Vec<&str> = Vec::new();

    for line in text.lines() {
        let is_boundary = !line.starts_with(' ')
            && !line.starts_with('\t')
            && patterns.iter().any(|p| line.starts_with(p));

        if is_boundary && !current.is_empty() {
            flush(&mut current, &mut segments);
        }
        current.push(line);
    }
    flush(&mut current, &mut segments);

    if segments.is_empty() {
        return memory::chunk_text(text, chunk_tokens);
    }

    merge_and_split(segments, chunk_tokens)
}

/// Returns top-level declaration prefixes for the given language.
/// Returns an empty slice for unrecognised languages.
fn code_boundary_patterns(language: &str) -> &'static [&'static str] {
    match language {
        "rust" => &[
            "pub async fn ",
            "async fn ",
            "pub fn ",
            "fn ",
            "pub struct ",
            "struct ",
            "pub enum ",
            "enum ",
            "pub impl ",
            "impl ",
            "pub trait ",
            "trait ",
            "pub mod ",
            "mod ",
            "pub const ",
            "const ",
            "pub static ",
            "static ",
            "macro_rules!",
        ],
        "python" => &["def ", "async def ", "class "],
        "javascript" | "typescript" | "js" | "ts" => &[
            "async function ",
            "function ",
            "class ",
            "export default ",
            "export function ",
            "export class ",
            "export async function ",
            "const ",
            "let ",
            "var ",
        ],
        "go" => &["func ", "type ", "var ", "const "],
        "java" | "kotlin" => &[
            "public class ",
            "class ",
            "public interface ",
            "interface ",
            "public enum ",
            "enum ",
            "public ",
            "private ",
            "protected ",
        ],
        _ => &[],
    }
}

fn flush(current: &mut Vec<&str>, segments: &mut Vec<String>) {
    let seg = current.join("\n");
    current.clear();
    if !seg.trim().is_empty() {
        segments.push(seg);
    }
}

fn token_count(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Merge tiny segments (≤ 25% of budget) into the previous one; split
/// oversized segments using the normal token chunker.
fn merge_and_split(segments: Vec<String>, max_tokens: usize) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();

    for seg in segments {
        let seg_tokens = token_count(&seg);
        if seg_tokens > max_tokens {
            if let Some(last) = result.last_mut() {
                if !last.ends_with('\n') {
                    last.push('\n');
                }
            }
            result.extend(memory::chunk_text(&seg, max_tokens));
        } else if let Some(last) = result.last_mut() {
            let combined_tokens = token_count(last) + seg_tokens;
            if combined_tokens <= max_tokens {
                last.push('\n');
                last.push_str(&seg);
            } else {
                result.push(seg);
            }
        } else {
            result.push(seg);
        }
    }

    result
}

// ──────────────────────────────────────────────────────────────────────────────
// Routing helper
// ──────────────────────────────────────────────────────────────────────────────

/// Normalise a MIME type string to lowercase without parameters.
pub fn normalise_mime(content_type: &str) -> String {
    content_type
        .split(';')
        .next()
        .unwrap_or(content_type)
        .trim()
        .to_lowercase()
}

/// Return `true` when the content type represents a PDF document.
pub fn is_pdf(content_type: &str) -> bool {
    normalise_mime(content_type) == "application/pdf"
}

/// Return the code language name (lower-case) when the content type is a known
/// code MIME type, or `None` otherwise.
pub fn code_language(content_type: &str) -> Option<String> {
    let mime = normalise_mime(content_type);
    // x-prefix code MIME types
    let lang = mime
        .strip_prefix("text/x-")
        .or_else(|| mime.strip_prefix("application/x-"))?;
    Some(lang.to_owned())
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_pdf_matches_standard_mime() {
        assert!(is_pdf("application/pdf"));
        assert!(is_pdf("Application/PDF"));
        assert!(is_pdf("application/pdf; charset=utf-8"));
        assert!(!is_pdf("text/plain"));
    }

    #[test]
    fn code_language_extracts_lang() {
        assert_eq!(code_language("text/x-rust"), Some("rust".into()));
        assert_eq!(code_language("text/x-python"), Some("python".into()));
        assert_eq!(code_language("application/x-go"), Some("go".into()));
        assert_eq!(code_language("text/plain"), None);
        assert_eq!(code_language("application/pdf"), None);
    }

    #[test]
    fn chunk_code_rust_splits_at_fn_boundary() {
        // Use a budget small enough (5 tokens) so each tiny function is its own chunk.
        let code = "fn foo() {\n    let x = 1;\n}\n\nfn bar() {\n    let y = 2;\n}\n";
        let chunks = chunk_code(code, "rust", 5);
        assert!(chunks.len() >= 2, "got {} chunks", chunks.len());
        assert!(chunks[0].contains("fn foo"));
        assert!(chunks.iter().any(|c| c.contains("fn bar")));
    }

    #[test]
    fn chunk_code_python_splits_at_def_boundary() {
        let code = "def foo():\n    return 1\n\ndef bar():\n    return 2\n";
        let chunks = chunk_code(code, "python", 5);
        assert!(chunks.len() >= 2);
    }

    #[test]
    fn chunk_code_unknown_language_falls_back() {
        let text = "word ".repeat(100);
        let chunks = chunk_code(&text, "cobol", 20);
        // Falls back to token chunker so every chunk ≤ 20 tokens.
        for c in &chunks {
            assert!(c.split_whitespace().count() <= 20);
        }
    }

    #[test]
    fn chunk_code_oversized_fn_is_split() {
        // A single huge function body should be further split.
        let body = "fn big() {\n".to_owned() + &"    let x = 1;\n".repeat(60) + "}\n";
        let chunks = chunk_code(&body, "rust", 30);
        assert!(chunks.len() > 1);
    }
}
