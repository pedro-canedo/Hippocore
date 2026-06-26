//! Document chunking regression tests.
//!
//! `store_document` auto-chunks the text and indexes each chunk independently.
//! These tests lock the chunking path: chunks are created, chunk-level recall
//! returns the parent document id, and chunks respect lifecycle events (compact,
//! delete).

use hippocore::{Config, Hippocore, RecallRequest, StoreDocumentRequest};
use tempfile::TempDir;

fn open(dir: &TempDir) -> Hippocore {
    Hippocore::open(Config::new(dir.path())).expect("open")
}

fn seed(db: &mut Hippocore) {
    db.create_tenant("acme", "Acme").unwrap();
    db.create_collection("acme", "kb", "").unwrap();
}

fn store_doc(db: &mut Hippocore, id: &str, text: &str) -> hippocore::model::Document {
    db.store_document(StoreDocumentRequest {
        tenant_id: "acme".into(),
        collection: "kb".into(),
        id: Some(id.into()),
        text: text.into(),
        content_type: None,
        raw: None,
        metadata: Default::default(),
        source: None,
        chunks: None,
        valid_from: None,
        valid_until: None,
    })
    .unwrap()
}

// ── store_document produces at least one chunk ────────────────────────────────

#[test]
fn stored_document_produces_chunks() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    seed(&mut db);

    let doc = store_doc(&mut db, "doc-1", "backpropagation through time is a method for training recurrent neural networks on sequential data");

    let chunks = db.get_document_chunks("acme", "kb", &doc.id);
    assert!(
        !chunks.is_empty(),
        "at least one chunk must be created for the document"
    );
}

// ── Recall returns the parent document id via its chunks ──────────────────────

#[test]
fn recall_returns_parent_document_id() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    seed(&mut db);

    let doc = store_doc(
        &mut db,
        "doc-2",
        "variational autoencoders learn a latent space representation of input data",
    );

    let hits = db
        .recall(RecallRequest::new("acme", "latent space variational"))
        .unwrap();
    // For DocumentChunk results, `id` is the chunk id and `document_id` is the
    // parent document id.
    assert!(
        hits.iter()
            .any(|h| h.document_id.as_deref() == Some(&doc.id)),
        "recall must surface the document via its chunks (document_id field)"
    );
}

// ── Two documents each produce their own chunk set ────────────────────────────

#[test]
fn two_documents_produce_separate_chunks() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    seed(&mut db);

    let doc_a = store_doc(
        &mut db,
        "doc-a",
        "gradient descent minimises the loss function",
    );
    let doc_b = store_doc(
        &mut db,
        "doc-b",
        "attention mechanism scales dot products by sqrt of dimension",
    );

    let chunks_a = db.get_document_chunks("acme", "kb", &doc_a.id);
    let chunks_b = db.get_document_chunks("acme", "kb", &doc_b.id);

    assert!(!chunks_a.is_empty(), "doc-a must have chunks");
    assert!(!chunks_b.is_empty(), "doc-b must have chunks");

    // No chunk from doc-a should appear in doc-b's chunk list.
    let a_ids: std::collections::HashSet<&str> = chunks_a.iter().map(|c| c.id.as_str()).collect();
    let b_ids: std::collections::HashSet<&str> = chunks_b.iter().map(|c| c.id.as_str()).collect();
    assert!(
        a_ids.is_disjoint(&b_ids),
        "chunk ids from doc-a and doc-b must be disjoint"
    );
}

// ── Chunks survive compact + cold reopen ─────────────────────────────────────

#[test]
fn chunks_survive_compact_reopen() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    seed(&mut db);

    let doc = store_doc(
        &mut db,
        "doc-3",
        "convolutional neural networks extract hierarchical features from images",
    );

    db.compact().unwrap();
    drop(db);

    let db2 = open(&dir);
    let chunks = db2.get_document_chunks("acme", "kb", &doc.id);
    assert!(
        !chunks.is_empty(),
        "chunks must survive compact + cold reopen"
    );
}

// ── Deleting a document removes its chunks ────────────────────────────────────

#[test]
fn deleting_document_removes_chunks() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    seed(&mut db);

    let doc = store_doc(
        &mut db,
        "doc-4",
        "transformer encoder decoder architecture for sequence to sequence tasks",
    );

    assert!(
        !db.get_document_chunks("acme", "kb", &doc.id).is_empty(),
        "chunks must exist before deletion"
    );

    db.delete_document("acme", "kb", &doc.id).unwrap();

    let chunks_after = db.get_document_chunks("acme", "kb", &doc.id);
    assert!(
        chunks_after.is_empty(),
        "chunks must be removed after document deletion"
    );
}
