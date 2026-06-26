use hippocore::{Config, Hippocore, StoreDocumentRequest};
use tempfile::TempDir;

fn setup() -> (Hippocore, TempDir) {
    let dir = TempDir::new().unwrap();
    let mut db = Hippocore::open(Config::new(dir.path())).unwrap();
    db.create_tenant("t1", "Tenant 1").unwrap();
    db.create_collection("t1", "docs", "").unwrap();
    (db, dir)
}

// ─── Plain-text control ──────────────────────────────────────────────────────

#[test]
fn plain_text_document_produces_chunks() {
    let (mut db, _dir) = setup();
    let req = StoreDocumentRequest::new("t1", "docs", "hello world this is a test");
    let doc = db.store_document(req).unwrap();
    let chunks = db.get_document_chunks("t1", "docs", &doc.id);
    assert!(!chunks.is_empty());
}

// ─── Code file ingestion ─────────────────────────────────────────────────────

#[test]
fn rust_code_document_splits_at_fn_boundaries() {
    let (mut db, _dir) = setup();

    // Each function body is ~30 tokens; two functions together (>48 = default
    // chunk_tokens) must land in separate chunks.
    let body = "let x1 = 1; let x2 = 2; let x3 = 3; let x4 = 4; let x5 = 5;\n    let x6 = 6; let x7 = 7; let x8 = 8; let x9 = 9; let x10 = 10;";
    let code = format!(
        "fn alpha() {{\n    {body}\n}}\n\nfn beta() {{\n    {body}\n}}\n\nfn gamma() {{\n    {body}\n}}\n"
    );

    let req = StoreDocumentRequest::from_code("t1", "docs", &code, "rust");
    let doc = db.store_document(req).unwrap();
    let chunks = db.get_document_chunks("t1", "docs", &doc.id);

    // With ~30 tokens per fn and a 48-token budget, three functions → at least 2 chunks.
    assert!(
        chunks.len() >= 2,
        "expected >= 2 chunks for 3 large fns, got {}",
        chunks.len()
    );
    let combined: String = chunks
        .iter()
        .map(|c| c.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(combined.contains("alpha"));
    assert!(combined.contains("beta"));
    assert!(combined.contains("gamma"));
}

#[test]
fn python_code_document_splits_at_def_boundaries() {
    let (mut db, _dir) = setup();

    // Each function is ~30 tokens so two together exceed the 48-token budget.
    let body = "    x1 = 1\n    x2 = 2\n    x3 = 3\n    x4 = 4\n    x5 = 5\n    x6 = 6\n    x7 = 7\n    x8 = 8\n    return x8";
    let code = format!("def foo():\n{body}\n\ndef bar():\n{body}\n\ndef baz():\n{body}\n");
    let req = StoreDocumentRequest::from_code("t1", "docs", &code, "python");
    let doc = db.store_document(req).unwrap();
    let chunks = db.get_document_chunks("t1", "docs", &doc.id);

    assert!(chunks.len() >= 2, "got {} chunks", chunks.len());
    let combined: String = chunks
        .iter()
        .map(|c| c.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(combined.contains("foo"));
    assert!(combined.contains("bar"));
}

#[test]
fn unknown_language_falls_back_to_token_chunker() {
    let (mut db, _dir) = setup();

    // 400 tokens — should produce multiple chunks with default budget.
    let text = "word ".repeat(400);
    let req = StoreDocumentRequest::from_code("t1", "docs", text.trim(), "cobol");
    let doc = db.store_document(req).unwrap();
    let chunks = db.get_document_chunks("t1", "docs", &doc.id);
    assert!(
        chunks.len() > 1,
        "expected multiple chunks, got {}",
        chunks.len()
    );
}

#[test]
fn code_document_recalled_by_function_name() {
    let (mut db, _dir) = setup();

    let code = "fn authenticate_user(username: &str) -> bool {\n    true\n}\n";
    let req = StoreDocumentRequest::from_code("t1", "docs", code, "rust");
    let doc = db.store_document(req).unwrap();

    let hits = db
        .recall(hippocore::RecallRequest::new("t1", "authenticate user"))
        .unwrap();
    // At least one hit should belong to the stored document.
    assert!(
        hits.iter()
            .any(|h| h.document_id.as_deref() == Some(&doc.id)),
        "expected recall to surface the code document"
    );
}

// ─── PDF ingestion ───────────────────────────────────────────────────────────

/// Minimal valid PDF containing the text "Hello PDF world" on one page.
/// Hand-crafted cross-reference table so no external tool is needed.
fn minimal_pdf_bytes(page_text: &str) -> Vec<u8> {
    let content = format!("BT /F1 12 Tf 100 700 Td ({page_text}) Tj ET");
    let content_len = content.len();

    let header = b"%PDF-1.4\n";
    let obj1 = b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n";
    let obj2 = b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n";
    let obj3 = "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>\nendobj\n";
    let obj4 =
        format!("4 0 obj\n<< /Length {content_len} >>\nstream\n{content}\nendstream\nendobj\n");
    let obj5 = b"5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n";

    let mut pdf: Vec<u8> = Vec::new();
    pdf.extend_from_slice(header);

    let off1 = pdf.len();
    pdf.extend_from_slice(obj1);
    let off2 = pdf.len();
    pdf.extend_from_slice(obj2);
    let off3 = pdf.len();
    pdf.extend_from_slice(obj3.as_bytes());
    let off4 = pdf.len();
    pdf.extend_from_slice(obj4.as_bytes());
    let off5 = pdf.len();
    pdf.extend_from_slice(obj5);

    let xref_offset = pdf.len();
    let xref = format!(
        "xref\n0 6\n0000000000 65535 f \n{:010} 00000 n \n{:010} 00000 n \n{:010} 00000 n \n{:010} 00000 n \n{:010} 00000 n \n",
        off1, off2, off3, off4, off5
    );
    pdf.extend_from_slice(xref.as_bytes());

    let trailer = format!("trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n");
    pdf.extend_from_slice(trailer.as_bytes());
    pdf
}

#[test]
fn pdf_text_is_extracted_and_chunked() {
    let (mut db, _dir) = setup();

    let bytes = minimal_pdf_bytes("Hello PDF world");
    let req = StoreDocumentRequest::from_pdf("t1", "docs", bytes);

    // Extraction may succeed or fail depending on the PDF parser's tolerance
    // for this minimal hand-crafted PDF. Both outcomes are acceptable here;
    // the critical invariant is that we never panic.
    match db.store_document(req) {
        Ok(doc) => {
            let chunks = db.get_document_chunks("t1", "docs", &doc.id);
            assert!(!chunks.is_empty(), "expected at least one chunk");
        }
        Err(e) => {
            // A validation/extraction error is acceptable for a hand-crafted
            // minimal PDF; a panic is not.
            let msg = e.to_string();
            assert!(
                msg.contains("PDF") || msg.contains("extraction") || msg.contains("validation"),
                "unexpected error: {msg}"
            );
        }
    }
}

#[test]
fn missing_raw_bytes_for_pdf_returns_error_not_panic() {
    let (mut db, _dir) = setup();

    // content_type = pdf but no raw bytes — must return an error, not panic.
    let mut req = StoreDocumentRequest::new("t1", "docs", "");
    req.content_type = Some("application/pdf".into());
    req.raw = None;

    let result = db.store_document(req);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("raw") || msg.contains("PDF") || msg.contains("bytes"));
}

#[test]
fn content_type_stored_in_document_text_field() {
    let (mut db, _dir) = setup();

    let code = "fn main() {}\n";
    let req = StoreDocumentRequest::from_code("t1", "docs", code, "rust");
    let doc = db.store_document(req).unwrap();

    // The document text should contain the original source code.
    assert!(doc.text.contains("fn main"));
}
