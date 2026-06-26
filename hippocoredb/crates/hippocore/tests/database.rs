//! Integration tests for the Hippocore memory database.

use hippocore::model::{ItemKind, MemoryType};
use hippocore::{
    AddGraphEdgeRequest, BuildContextRequest, ChunkInput, Config, Hippocore, HippocoreError,
    ImportFileRequest, PutRecordRequest, RecallRequest, RememberRequest, SearchMode,
    StoreDocumentRequest,
};
use serde_json::json;
use tempfile::TempDir;

fn open(dir: &TempDir) -> Hippocore {
    Hippocore::open(Config::new(dir.path())).expect("open")
}

fn seeded(dir: &TempDir) -> Hippocore {
    let mut db = open(dir);
    db.create_tenant("acme", "Acme").unwrap();
    db.create_collection("acme", "support", "").unwrap();
    db
}

fn remember(db: &mut Hippocore, collection: &str, text: &str) {
    db.remember(RememberRequest::new(
        "acme",
        collection,
        MemoryType::Note,
        text,
    ))
    .unwrap();
}

fn remember_id(db: &mut Hippocore, id: &str, text: &str) {
    db.remember(RememberRequest {
        id: Some(id.into()),
        ..RememberRequest::new("acme", "support", MemoryType::Semantic, text)
    })
    .unwrap();
}

#[test]
fn store_and_recall_memory() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);
    remember(
        &mut db,
        "support",
        "Oracle ORA-12514 listener service name issue",
    );
    remember(&mut db, "support", "How to bake sourdough bread at home");

    let hits = db
        .recall(RecallRequest::new("acme", "oracle listener service"))
        .unwrap();
    assert!(!hits.is_empty());
    assert!(hits[0].text.to_lowercase().contains("oracle"));
}

#[test]
fn store_document_chunks_and_recalls() {
    let dir = TempDir::new().unwrap();
    let mut cfg = Config::new(dir.path());
    cfg.chunk_tokens = 4; // force multiple chunks
    let mut db = Hippocore::open(cfg).unwrap();
    db.create_tenant("acme", "Acme").unwrap();
    db.create_collection("acme", "support", "").unwrap();
    let req = StoreDocumentRequest::new(
        "acme",
        "support",
        "one two three four five six seven eight nine ten eleven twelve",
    );
    let doc = db.store_document(req).unwrap();
    assert_eq!(doc.version, 0);

    let stats = db.stats().unwrap();
    assert_eq!(stats.documents, 1);
    assert!(
        stats.chunks >= 2,
        "expected multiple chunks, got {}",
        stats.chunks
    );

    let mut req = RecallRequest::new("acme", "three four");
    req.kind = Some(ItemKind::DocumentChunk);
    let hits = db.search(req).unwrap();
    assert!(!hits.is_empty());
    assert_eq!(hits[0].kind, ItemKind::DocumentChunk);
}

#[test]
fn overwrite_document_increments_version() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);
    let mut req = StoreDocumentRequest::new("acme", "support", "first version text");
    req.id = Some("doc-1".into());
    let v0 = db.store_document(req.clone()).unwrap();
    assert_eq!(v0.version, 0);

    req.text = "second version text".into();
    let v1 = db.store_document(req).unwrap();
    assert_eq!(v1.version, 1);
    assert_eq!(v1.created_at, v0.created_at);
    // Only one document remains; old chunks were replaced.
    assert_eq!(db.stats().unwrap().documents, 1);
}

#[test]
fn import_file_recalls_and_tracks_stats() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("postgres.md");
    std::fs::write(
        &file_path,
        "PostgreSQL starts with sudo systemctl start postgresql on port 5432.",
    )
    .unwrap();

    let mut db = seeded(&dir);
    let mut req = ImportFileRequest::new("acme", "support", &file_path);
    req.id = Some("pg-guide".into());
    req.metadata.insert("topic".into(), "postgres".into());
    let file = db.import_file(req).unwrap();
    assert_eq!(file.id, "pg-guide");
    assert_eq!(file.media_type, "text/markdown");

    let stats = db.stats().unwrap();
    assert_eq!(stats.files, 1);
    assert_eq!(stats.documents, 1);
    assert!(stats.chunks >= 1);

    let mut recall = RecallRequest::new("acme", "systemctl postgresql 5432");
    recall.metadata.insert("file_id".into(), "pg-guide".into());
    let hits = db.recall(recall).unwrap();
    assert!(!hits.is_empty());
    assert_eq!(
        hits[0].metadata.get("topic").map(String::as_str),
        Some("postgres")
    );
    assert_eq!(hits[0].document_id.as_deref(), Some("file:pg-guide"));
}

#[test]
fn import_json_file_projects_structured_text() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("db.json");
    std::fs::write(
        &file_path,
        r#"{"name":"billing","engine":"postgresql","port":5432}"#,
    )
    .unwrap();

    let mut db = seeded(&dir);
    db.import_file(ImportFileRequest {
        id: Some("db-json".into()),
        ..ImportFileRequest::new("acme", "support", &file_path)
    })
    .unwrap();

    let hits = db
        .recall(RecallRequest::new("acme", "billing postgresql 5432"))
        .unwrap();
    assert!(!hits.is_empty());
    assert_eq!(
        hits[0].metadata.get("media_type").map(String::as_str),
        Some("application/json")
    );
    assert!(hits[0].text.contains("postgresql"));
}

#[test]
fn imported_file_survives_restart_and_delete() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("oracle.txt");
    std::fs::write(&file_path, "Oracle listener starts with lsnrctl start.").unwrap();

    {
        let mut db = seeded(&dir);
        db.import_file(ImportFileRequest {
            id: Some("oracle-file".into()),
            ..ImportFileRequest::new("acme", "support", &file_path)
        })
        .unwrap();
        db.close().unwrap();
    }

    {
        let mut db = open(&dir);
        assert_eq!(db.stats().unwrap().files, 1);
        let hits = db
            .recall(RecallRequest::new("acme", "lsnrctl listener"))
            .unwrap();
        assert!(!hits.is_empty());
        db.delete_file("acme", "support", "oracle-file").unwrap();
        db.close().unwrap();
    }

    let db = open(&dir);
    let stats = db.stats().unwrap();
    assert_eq!(stats.files, 0);
    assert_eq!(stats.documents, 0);
    assert_eq!(stats.chunks, 0);
    assert!(db
        .recall(RecallRequest::new("acme", "lsnrctl listener"))
        .unwrap()
        .is_empty());
}

#[test]
fn import_file_rejects_unsupported_extension() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("data.bin");
    std::fs::write(&file_path, b"postgresql").unwrap();

    let mut db = seeded(&dir);
    let err = db
        .import_file(ImportFileRequest::new("acme", "support", &file_path))
        .unwrap_err();
    assert!(matches!(err, HippocoreError::Validation(_)));
}

#[test]
fn vector_text_and_hybrid_modes_all_work() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);
    remember(
        &mut db,
        "support",
        "oracle database connection error listener",
    );
    remember(&mut db, "support", "postgres database vacuum tuning guide");

    for mode in [SearchMode::Vector, SearchMode::Text, SearchMode::Hybrid] {
        let mut req = RecallRequest::new("acme", "oracle listener");
        req.mode = mode;
        let hits = db.search(req).unwrap();
        assert!(!hits.is_empty(), "mode {mode:?} returned nothing");
        assert!(
            hits[0].text.contains("oracle"),
            "mode {mode:?} ranked wrong result first"
        );
    }
}

#[test]
fn results_sorted_by_score_descending() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);
    remember(&mut db, "support", "oracle listener oracle listener oracle");
    remember(&mut db, "support", "oracle once mentioned here");
    remember(&mut db, "support", "completely unrelated cooking recipe");

    let mut req = RecallRequest::new("acme", "oracle listener");
    req.mode = SearchMode::Text;
    let hits = db.search(req).unwrap();
    assert!(hits.len() >= 2);
    for w in hits.windows(2) {
        assert!(w[0].score >= w[1].score);
    }
}

#[test]
fn postgres_query_prefers_postgres_memories() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);
    remember_id(
        &mut db,
        "oracle-listener",
        "Oracle ORA-12541 listener error uses lsnrctl start and tnsnames.ora.",
    );
    remember_id(
        &mut db,
        "postgres-start",
        "PostgreSQL starts with sudo systemctl start postgresql and listens on TCP port 5432.",
    );

    let hits = db
        .recall(RecallRequest::new("acme", "Como postgress funciona?"))
        .unwrap();
    assert!(!hits.is_empty());
    assert_eq!(hits[0].id, "postgres-start");
}

#[test]
fn oracle_query_prefers_oracle_memories() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);
    remember_id(
        &mut db,
        "postgres-start",
        "PostgreSQL starts with sudo systemctl start postgresql and uses TCP port 5432.",
    );
    remember_id(
        &mut db,
        "oracle-listener",
        "Oracle listener errors are checked with lsnrctl status and started with lsnrctl start.",
    );

    let hits = db
        .recall(RecallRequest::new(
            "acme",
            "Como inicio o listener do Oracle?",
        ))
        .unwrap();
    assert!(!hits.is_empty());
    assert_eq!(hits[0].id, "oracle-listener");
}

#[test]
fn mixed_oracle_postgres_query_allows_both() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);
    remember_id(
        &mut db,
        "postgres-start",
        "PostgreSQL starts with sudo systemctl start postgresql and uses TCP port 5432.",
    );
    remember_id(
        &mut db,
        "oracle-listener",
        "Oracle listener starts with lsnrctl start and status is checked with lsnrctl status.",
    );

    let mut req = RecallRequest::new("acme", "listener Oracle e postgress");
    req.top_k = 2;
    let hits = db.recall(req).unwrap();
    let ids: Vec<&str> = hits.iter().map(|h| h.id.as_str()).collect();
    assert!(ids.contains(&"postgres-start"), "{ids:?}");
    assert!(ids.contains(&"oracle-listener"), "{ids:?}");
}

#[test]
fn python_postgres_connection_does_not_rank_oracle_first() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);
    remember_id(
        &mut db,
        "oracle-listener",
        "Oracle ORA-12514 and ORA-12541 are listener errors resolved with lsnrctl status.",
    );
    remember_id(
        &mut db,
        "postgres-python",
        "Python connects to PostgreSQL using psycopg or psycopg2 on TCP port 5432.",
    );

    let hits = db
        .recall(RecallRequest::new(
            "acme",
            "como faço uma conexão python no postgress?",
        ))
        .unwrap();
    assert!(!hits.is_empty());
    assert_eq!(hits[0].id, "postgres-python");
    assert_ne!(hits[0].id, "oracle-listener");
}

#[test]
fn metadata_filter_is_respected() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);
    let mut a = RememberRequest::new("acme", "support", MemoryType::Note, "oracle issue in hml");
    a.metadata.insert("env".into(), "hml".into());
    db.remember(a).unwrap();
    let mut b = RememberRequest::new("acme", "support", MemoryType::Note, "oracle issue in prod");
    b.metadata.insert("env".into(), "prod".into());
    db.remember(b).unwrap();

    let mut req = RecallRequest::new("acme", "oracle issue");
    req.metadata.insert("env".into(), "hml".into());
    let hits = db.search(req).unwrap();
    assert_eq!(hits.len(), 1);
    assert!(hits[0].text.contains("hml"));
}

#[test]
fn memory_type_and_user_filters() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);
    let mut sem = RememberRequest::new("acme", "support", MemoryType::Semantic, "oracle fact");
    sem.user_id = Some("u1".into());
    db.remember(sem).unwrap();
    let mut epi = RememberRequest::new("acme", "support", MemoryType::Episodic, "oracle event");
    epi.user_id = Some("u2".into());
    db.remember(epi).unwrap();

    let mut by_type = RecallRequest::new("acme", "oracle");
    by_type.memory_type = Some(MemoryType::Semantic);
    assert_eq!(db.search(by_type).unwrap().len(), 1);

    let mut by_user = RecallRequest::new("acme", "oracle");
    by_user.user_id = Some("u2".into());
    let hits = db.search(by_user).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].user_id.as_deref(), Some("u2"));
}

#[test]
fn tenant_isolation() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    db.create_tenant("a", "A").unwrap();
    db.create_tenant("b", "B").unwrap();
    db.create_collection("a", "c", "").unwrap();
    db.create_collection("b", "c", "").unwrap();

    db.remember(RememberRequest::new(
        "a",
        "c",
        MemoryType::Note,
        "oracle secret of tenant a",
    ))
    .unwrap();
    db.remember(RememberRequest::new(
        "b",
        "c",
        MemoryType::Note,
        "oracle secret of tenant b",
    ))
    .unwrap();

    let hits = db.recall(RecallRequest::new("a", "oracle secret")).unwrap();
    assert_eq!(hits.len(), 1);
    assert!(hits.iter().all(|h| h.tenant_id == "a"));
    assert!(hits[0].text.contains("tenant a"));
}

#[test]
fn persistence_survives_restart() {
    let dir = TempDir::new().unwrap();
    {
        let mut db = seeded(&dir);
        remember(&mut db, "support", "oracle listener persisted memory");
        db.store_document(StoreDocumentRequest::new(
            "acme",
            "support",
            "a persisted document about oracle networking",
        ))
        .unwrap();
        db.close().unwrap();
    }
    // Reopen: WAL replay must restore everything.
    let db = open(&dir);
    let stats = db.stats().unwrap();
    assert_eq!(stats.tenants, 1);
    assert_eq!(stats.collections, 1);
    assert_eq!(stats.memories, 1);
    assert_eq!(stats.documents, 1);
    let hits = db.recall(RecallRequest::new("acme", "oracle")).unwrap();
    assert!(!hits.is_empty());
}

#[test]
fn persistence_survives_compaction() {
    let dir = TempDir::new().unwrap();
    {
        let mut db = seeded(&dir);
        remember(&mut db, "support", "oracle memory before compaction");
        db.compact().unwrap();
        remember(&mut db, "support", "oracle memory after compaction");
        db.close().unwrap();
    }
    let db = open(&dir);
    let hits = db
        .recall(RecallRequest::new("acme", "oracle memory"))
        .unwrap();
    assert_eq!(hits.len(), 2);
    // WAL was truncated at compaction, so only the post-compaction op remains.
    assert_eq!(db.stats().unwrap().wal_entries, 1);
}

#[test]
fn user_supplied_embedding_is_used() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);
    let mut req = RememberRequest::new("acme", "support", MemoryType::Note, "anything");
    req.embedding = Some(vec![1.0, 0.0, 0.0]);
    db.remember(req).unwrap();

    let mut q = RecallRequest::new("acme", "");
    q.embedding = Some(vec![1.0, 0.0, 0.0]);
    q.mode = SearchMode::Vector;
    let hits = db.search(q).unwrap();
    assert_eq!(hits.len(), 1);
    assert!(hits[0].vector_score > 0.0);
}

#[test]
fn empty_database_open_and_recall() {
    let dir = TempDir::new().unwrap();
    let db = open(&dir);
    assert_eq!(db.stats().unwrap().memories, 0);
    let hits = db.recall(RecallRequest::new("ghost", "anything")).unwrap();
    assert!(hits.is_empty());
}

#[test]
fn create_collection_unknown_tenant_errors() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    assert!(matches!(
        db.create_collection("nope", "c", ""),
        Err(HippocoreError::UnknownTenant(_))
    ));
}

#[test]
fn store_into_unknown_collection_errors() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    db.create_tenant("acme", "Acme").unwrap();
    assert!(matches!(
        db.store_document(StoreDocumentRequest::new("acme", "missing", "text")),
        Err(HippocoreError::UnknownCollection { .. })
    ));
}

#[test]
fn empty_required_fields_are_rejected() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);
    assert!(matches!(
        db.remember(RememberRequest::new(
            "acme",
            "support",
            MemoryType::Note,
            "   "
        )),
        Err(HippocoreError::Validation(_))
    ));
    assert!(matches!(
        db.create_tenant("", "x"),
        Err(HippocoreError::Validation(_))
    ));
}

#[test]
fn empty_supplied_embedding_is_rejected() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);
    let mut req = RememberRequest::new("acme", "support", MemoryType::Note, "text");
    req.embedding = Some(vec![]);
    assert!(matches!(
        db.remember(req),
        Err(HippocoreError::InvalidEmbedding(_))
    ));
}

#[test]
fn recall_without_query_or_embedding_errors() {
    let dir = TempDir::new().unwrap();
    let db = seeded(&dir);
    assert!(matches!(
        db.recall(RecallRequest::new("acme", "   ")),
        Err(HippocoreError::Validation(_))
    ));
}

#[test]
fn vector_dimension_mismatch_does_not_panic() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);
    let mut req = RememberRequest::new("acme", "support", MemoryType::Note, "text");
    req.embedding = Some(vec![1.0, 2.0, 3.0]);
    db.remember(req).unwrap();

    let mut q = RecallRequest::new("acme", "");
    q.embedding = Some(vec![1.0, 2.0]); // wrong dimension
    q.mode = SearchMode::Vector;
    let hits = db.search(q).unwrap();
    assert!(hits.is_empty()); // skipped, no panic
}

#[test]
fn store_document_with_user_embeddings_uses_them() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);
    let mut req = StoreDocumentRequest::new("acme", "support", "full document body");
    req.id = Some("d1".into());
    req.chunks = Some(vec![
        ChunkInput::new("first chunk about oracle", vec![1.0, 0.0, 0.0]),
        ChunkInput::new("second chunk about postgres", vec![0.0, 1.0, 0.0]),
    ]);
    db.store_document(req).unwrap();
    assert_eq!(db.stats().unwrap().chunks, 2);

    // Vector recall with a query embedding equal to the first chunk's vector
    // must rank that chunk first — proving the supplied vectors are used.
    let mut q = RecallRequest::new("acme", "");
    q.embedding = Some(vec![1.0, 0.0, 0.0]);
    q.mode = SearchMode::Vector;
    let hits = db.search(q).unwrap();
    assert!(!hits.is_empty());
    assert_eq!(hits[0].text, "first chunk about oracle");
    assert!(hits[0].vector_score > 0.0);

    // And it survives a restart (chunks persisted with their embeddings).
    db.close().unwrap();
    let db = open(&dir);
    assert_eq!(db.stats().unwrap().chunks, 2);
}

#[test]
fn store_document_rejects_empty_chunk_inputs() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);

    let mut empty_text = StoreDocumentRequest::new("acme", "support", "body");
    empty_text.chunks = Some(vec![ChunkInput::new("  ", vec![1.0, 0.0])]);
    assert!(matches!(
        db.store_document(empty_text),
        Err(HippocoreError::Validation(_))
    ));

    let mut empty_embedding = StoreDocumentRequest::new("acme", "support", "body");
    empty_embedding.chunks = Some(vec![ChunkInput::new("text", vec![])]);
    assert!(matches!(
        db.store_document(empty_embedding),
        Err(HippocoreError::InvalidEmbedding(_))
    ));
}

#[test]
fn put_record_projects_and_recalls_context() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);
    let mut req = PutRecordRequest::new(
        "acme",
        "support",
        "systems",
        json!({
            "name": "billing-db",
            "engine": "postgresql",
            "port": 5432,
            "owner": "payments"
        }),
    );
    req.id = Some("sys-1".into());
    req.metadata.insert("env".into(), "prod".into());
    let record = db.put_record(req).unwrap();
    assert_eq!(record.version, 0);
    assert_eq!(db.stats().unwrap().records, 1);

    let hits = db
        .recall(RecallRequest::new("acme", "billing postgresql 5432"))
        .unwrap();
    assert!(!hits.is_empty());
    assert_eq!(hits[0].kind, ItemKind::Record);
    assert_eq!(hits[0].id, "sys-1");
    assert_eq!(hits[0].record_table.as_deref(), Some("systems"));
    assert!(hits[0].text.contains("billing-db"));
}

#[test]
fn record_metadata_filter_is_respected() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);
    let mut prod = PutRecordRequest::new(
        "acme",
        "support",
        "systems",
        json!({"name": "billing-db", "engine": "postgresql"}),
    );
    prod.id = Some("prod".into());
    prod.metadata.insert("env".into(), "prod".into());
    db.put_record(prod).unwrap();

    let mut dev = PutRecordRequest::new(
        "acme",
        "support",
        "systems",
        json!({"name": "sandbox-db", "engine": "postgresql"}),
    );
    dev.id = Some("dev".into());
    dev.metadata.insert("env".into(), "dev".into());
    db.put_record(dev).unwrap();

    let mut req = RecallRequest::new("acme", "postgresql");
    req.kind = Some(ItemKind::Record);
    req.metadata.insert("env".into(), "prod".into());
    let hits = db.search(req).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].id, "prod");
}

#[test]
fn record_survives_restart_and_delete() {
    let dir = TempDir::new().unwrap();
    {
        let mut db = seeded(&dir);
        let mut req = PutRecordRequest::new(
            "acme",
            "support",
            "systems",
            json!({"name": "billing-db", "engine": "postgresql"}),
        );
        req.id = Some("sys-1".into());
        db.put_record(req).unwrap();
        db.close().unwrap();
    }

    {
        let mut db = open(&dir);
        assert_eq!(db.stats().unwrap().records, 1);
        let hits = db
            .recall(RecallRequest::new("acme", "billing postgresql"))
            .unwrap();
        assert_eq!(hits[0].id, "sys-1");

        db.delete_record("acme", "support", "systems", "sys-1")
            .unwrap();
        assert_eq!(db.stats().unwrap().records, 0);
        assert!(db
            .recall(RecallRequest::new("acme", "billing postgresql"))
            .unwrap()
            .is_empty());
        db.close().unwrap();
    }

    let db = open(&dir);
    assert_eq!(db.stats().unwrap().records, 0);
}

#[test]
fn record_tenant_isolation() {
    let dir = TempDir::new().unwrap();
    let mut db = open(&dir);
    db.create_tenant("a", "A").unwrap();
    db.create_tenant("b", "B").unwrap();
    db.create_collection("a", "c", "").unwrap();
    db.create_collection("b", "c", "").unwrap();

    let mut a = PutRecordRequest::new(
        "a",
        "c",
        "systems",
        json!({"name": "secret-db", "engine": "postgresql", "tenant": "a"}),
    );
    a.id = Some("same-id".into());
    db.put_record(a).unwrap();
    let mut b = PutRecordRequest::new(
        "b",
        "c",
        "systems",
        json!({"name": "secret-db", "engine": "postgresql", "tenant": "b"}),
    );
    b.id = Some("same-id".into());
    db.put_record(b).unwrap();

    let hits = db
        .recall(RecallRequest::new("a", "secret postgresql"))
        .unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].tenant_id, "a");
    assert!(hits[0].text.contains("tenant a"));
}

#[test]
fn forget_removes_memory_and_survives_restart() {
    let dir = TempDir::new().unwrap();
    {
        let mut db = seeded(&dir);
        db.remember(RememberRequest {
            id: Some("m1".into()),
            ..RememberRequest::new("acme", "support", MemoryType::Note, "oracle keep this")
        })
        .unwrap();
        db.remember(RememberRequest {
            id: Some("m2".into()),
            ..RememberRequest::new("acme", "support", MemoryType::Note, "oracle forget this")
        })
        .unwrap();

        db.forget("acme", "support", "m2").unwrap();

        let hits = db.recall(RecallRequest::new("acme", "oracle")).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, "m1");
        assert_eq!(db.stats().unwrap().memories, 1);
        db.close().unwrap();
    }
    // Tombstone is durable: m2 stays gone after reopen.
    let db = open(&dir);
    assert_eq!(db.stats().unwrap().memories, 1);
    let hits = db.recall(RecallRequest::new("acme", "oracle")).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].id, "m1");
}

#[test]
fn delete_document_removes_all_chunks_and_survives_restart() {
    let dir = TempDir::new().unwrap();
    {
        let mut cfg = Config::new(dir.path());
        cfg.chunk_tokens = 3; // several chunks
        let mut db = Hippocore::open(cfg).unwrap();
        db.create_tenant("acme", "Acme").unwrap();
        db.create_collection("acme", "support", "").unwrap();
        let mut req = StoreDocumentRequest::new(
            "acme",
            "support",
            "oracle listener service name tnsnames guide",
        );
        req.id = Some("doc-1".into());
        db.store_document(req).unwrap();
        assert!(db.stats().unwrap().chunks >= 2);

        db.delete_document("acme", "support", "doc-1").unwrap();
        let stats = db.stats().unwrap();
        assert_eq!(stats.documents, 0);
        assert_eq!(stats.chunks, 0);
        assert!(db
            .recall(RecallRequest::new("acme", "oracle"))
            .unwrap()
            .is_empty());
        db.close().unwrap();
    }
    let db = open(&dir);
    let stats = db.stats().unwrap();
    assert_eq!(stats.documents, 0);
    assert_eq!(stats.chunks, 0);
}

#[test]
fn delete_missing_is_safe() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);
    // No-op deletes on absent ids succeed and don't corrupt state.
    db.forget("acme", "support", "ghost").unwrap();
    db.delete_document("acme", "support", "ghost-doc").unwrap();
    remember(&mut db, "support", "oracle real memory");
    db.forget("acme", "support", "still-ghost").unwrap();
    assert_eq!(db.stats().unwrap().memories, 1);
    assert_eq!(
        db.recall(RecallRequest::new("acme", "oracle"))
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn compaction_drops_deleted_items() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);
    db.remember(RememberRequest {
        id: Some("m1".into()),
        ..RememberRequest::new("acme", "support", MemoryType::Note, "oracle one")
    })
    .unwrap();
    db.remember(RememberRequest {
        id: Some("m2".into()),
        ..RememberRequest::new("acme", "support", MemoryType::Note, "oracle two")
    })
    .unwrap();
    db.forget("acme", "support", "m1").unwrap();
    db.compact().unwrap();

    // After compaction the tombstone + dead record are gone; only m2 remains.
    assert_eq!(db.stats().unwrap().memories, 1);
    assert_eq!(db.stats().unwrap().wal_entries, 0);
    db.close().unwrap();
    let db = open(&dir);
    assert_eq!(db.stats().unwrap().memories, 1);
}

#[test]
fn auto_compaction_bounds_the_wal() {
    let dir = TempDir::new().unwrap();
    let mut cfg = Config::new(dir.path());
    cfg.auto_compact_after_ops = 5; // tiny threshold to force compaction
    cfg.auto_compact_after_bytes = 0; // ops-based only
    let mut db = Hippocore::open(cfg).unwrap();
    db.create_tenant("acme", "Acme").unwrap();
    db.create_collection("acme", "support", "").unwrap();

    for i in 0..50 {
        remember(&mut db, "support", &format!("oracle memory number {i}"));
    }

    // The WAL never grows past the threshold, yet all data is present.
    let stats = db.stats().unwrap();
    assert!(
        stats.wal_entries < 5,
        "wal should stay bounded, got {}",
        stats.wal_entries
    );
    assert_eq!(stats.memories, 50);

    // And it still recovers correctly after reopen (snapshot + residual WAL).
    db.close().unwrap();
    let db = open(&dir);
    assert_eq!(db.stats().unwrap().memories, 50);
    assert!(!db
        .recall(RecallRequest::new("acme", "oracle memory"))
        .unwrap()
        .is_empty());
}

#[test]
fn checksum_mismatch_stops_recovery_without_loading_bad_data() {
    let dir = TempDir::new().unwrap();
    {
        let mut cfg = Config::new(dir.path());
        cfg.auto_compact_after_ops = 0; // keep everything in the WAL
        cfg.auto_compact_after_bytes = 0;
        let mut db = Hippocore::open(cfg).unwrap();
        db.create_tenant("acme", "Acme").unwrap();
        db.create_collection("acme", "support", "").unwrap();
        remember(&mut db, "support", "first oracle memory");
        remember(&mut db, "support", "second oracle memory");
        remember(&mut db, "support", "third oracle memory");
        db.close().unwrap();
    }

    // Corrupt the payload of the LAST committed WAL line (flip a bit after the
    // tab). Its stored CRC no longer matches, simulating on-disk bit-rot.
    let wal = dir.path().join("wal.log");
    let content = std::fs::read_to_string(&wal).unwrap();
    let mut lines: Vec<String> = content.lines().map(str::to_string).collect();
    let last = lines.last_mut().unwrap();
    let tab = last.find('\t').unwrap();
    let mut bytes = last.clone().into_bytes();
    bytes[tab + 1] ^= 0x01;
    *last = String::from_utf8_lossy(&bytes).into_owned();
    std::fs::write(&wal, lines.join("\n") + "\n").unwrap();

    // Recovery stops at the corrupt line: the first two memories survive, the
    // corrupted third is NOT silently loaded, and nothing panics.
    let db = open(&dir);
    assert_eq!(db.stats().unwrap().memories, 2);
}

#[test]
fn trailing_corrupt_wal_line_recovers() {
    let dir = TempDir::new().unwrap();
    {
        let mut db = seeded(&dir);
        remember(&mut db, "support", "oracle good memory");
        db.close().unwrap();
    }
    // Append a garbage partial line to the WAL.
    let wal = dir.path().join("wal.log");
    let mut bytes = std::fs::read(&wal).unwrap();
    bytes.extend_from_slice(b"{ this is not valid json");
    std::fs::write(&wal, bytes).unwrap();

    let db = open(&dir);
    let hits = db.recall(RecallRequest::new("acme", "oracle")).unwrap();
    assert_eq!(hits.len(), 1);
}

#[test]
fn temporal_valid_from_filters_future_memories() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);

    // Memory valid from far in the future should not appear in a default recall.
    let far_future = i64::MAX;
    let mut future_req = RememberRequest::new(
        "acme",
        "support",
        MemoryType::Semantic,
        "future fact about unicorns",
    );
    future_req.id = Some("future".into());
    future_req.valid_from = Some(far_future);
    db.remember(future_req).unwrap();

    // Memory with no temporal constraints — always available.
    let mut now_req = RememberRequest::new(
        "acme",
        "support",
        MemoryType::Semantic,
        "present fact about unicorns",
    );
    now_req.id = Some("present".into());
    db.remember(now_req).unwrap();

    let hits = db.recall(RecallRequest::new("acme", "unicorns")).unwrap();
    let ids: Vec<&str> = hits.iter().map(|h| h.id.as_str()).collect();
    assert!(ids.contains(&"present"), "present should appear: {ids:?}");
    assert!(
        !ids.contains(&"future"),
        "future-valid memory should not appear: {ids:?}"
    );
}

#[test]
fn temporal_valid_until_expires_memory() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);

    // Already-expired memory (valid_until set to the past).
    let past = 1_000_000_i64; // epoch 1000s, Jan 1 1970 + 1000s
    let mut expired_req = RememberRequest::new(
        "acme",
        "support",
        MemoryType::Semantic,
        "ancient fact about dragons",
    );
    expired_req.id = Some("expired".into());
    expired_req.valid_until = Some(past);
    db.remember(expired_req).unwrap();

    // Non-expired memory.
    let mut current_req = RememberRequest::new(
        "acme",
        "support",
        MemoryType::Semantic,
        "current fact about dragons",
    );
    current_req.id = Some("current".into());
    db.remember(current_req).unwrap();

    let hits = db.recall(RecallRequest::new("acme", "dragons")).unwrap();
    let ids: Vec<&str> = hits.iter().map(|h| h.id.as_str()).collect();
    assert!(ids.contains(&"current"), "current should appear: {ids:?}");
    assert!(
        !ids.contains(&"expired"),
        "expired memory should not appear: {ids:?}"
    );
}

#[test]
fn temporal_as_of_queries_past_state() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);

    let t_past = 1_000_000_i64; // somewhere in the past
    let t_future = i64::MAX / 2; // well in the future

    // A memory valid only in a narrow window in the past.
    let mut past_req = RememberRequest::new(
        "acme",
        "support",
        MemoryType::Semantic,
        "historical truth about robots",
    );
    past_req.id = Some("historical".into());
    past_req.valid_from = Some(t_past - 1000);
    past_req.valid_until = Some(t_past + 1000);
    db.remember(past_req).unwrap();

    // A memory valid only in the future.
    let mut future_req = RememberRequest::new(
        "acme",
        "support",
        MemoryType::Semantic,
        "future truth about robots",
    );
    future_req.id = Some("future-robots".into());
    future_req.valid_from = Some(t_future);
    db.remember(future_req).unwrap();

    // Query at t_past: only the historical memory should appear.
    let mut req = RecallRequest::new("acme", "robots");
    req.as_of = Some(t_past);
    let hits = db.recall(req).unwrap();
    let ids: Vec<&str> = hits.iter().map(|h| h.id.as_str()).collect();
    assert!(
        ids.contains(&"historical"),
        "historical should appear at t_past: {ids:?}"
    );
    assert!(
        !ids.contains(&"future-robots"),
        "future-robots should not appear at t_past: {ids:?}"
    );

    // Default recall (as_of = now): neither should appear (historical expired, future not yet valid).
    let hits_now = db.recall(RecallRequest::new("acme", "robots")).unwrap();
    let ids_now: Vec<&str> = hits_now.iter().map(|h| h.id.as_str()).collect();
    assert!(
        !ids_now.contains(&"historical"),
        "historical should not appear at now: {ids_now:?}"
    );
    assert!(
        !ids_now.contains(&"future-robots"),
        "future-robots should not appear at now: {ids_now:?}"
    );
}

#[test]
fn temporal_legacy_entries_always_valid() {
    // Entries deserialized from storage without valid_from/valid_until
    // should behave as always-valid (None/None).
    let dir = TempDir::new().unwrap();
    {
        let mut db = seeded(&dir);
        // Store a memory with no temporal fields.
        let req = RememberRequest::new(
            "acme",
            "support",
            MemoryType::Semantic,
            "durable legacy fact about servers",
        );
        db.remember(req).unwrap();
        db.close().unwrap();
    }
    // Reopen (simulates legacy WAL recovery) and verify it still appears.
    let db = open(&dir);
    let hits = db.recall(RecallRequest::new("acme", "servers")).unwrap();
    assert!(
        !hits.is_empty(),
        "legacy memories must still appear after reopen"
    );
    assert!(
        hits.iter().any(|h| h.text.contains("legacy fact")),
        "legacy memory text should appear: {hits:?}"
    );
}

#[test]
fn temporal_document_chunks_inherit_validity() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);

    // Document valid only in the past.
    let past = 1_000_000_i64;
    let mut doc_req =
        StoreDocumentRequest::new("acme", "support", "expired document about satellites");
    doc_req.id = Some("sat-doc".into());
    doc_req.valid_until = Some(past);
    db.store_document(doc_req).unwrap();

    // Default recall (now) should not return expired document's chunks.
    let hits = db.recall(RecallRequest::new("acme", "satellites")).unwrap();
    assert!(
        hits.is_empty()
            || !hits
                .iter()
                .any(|h| h.document_id.as_deref() == Some("sat-doc")),
        "expired document chunks should not appear: {hits:?}"
    );

    // Query at t_past should return the document's chunks.
    let mut req = RecallRequest::new("acme", "satellites");
    req.as_of = Some(past - 1); // just before expiry
    let hits_past = db.recall(req).unwrap();
    assert!(
        hits_past
            .iter()
            .any(|h| h.document_id.as_deref() == Some("sat-doc")),
        "document chunks should appear before expiry: {hits_past:?}"
    );
}

#[test]
fn supersedes_excludes_old_memory_from_default_recall() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);

    // Store an original memory.
    let mut old_req = RememberRequest::new(
        "acme",
        "support",
        MemoryType::Semantic,
        "old fact about servers version one",
    );
    old_req.id = Some("srv-v1".into());
    db.remember(old_req).unwrap();

    // Store a replacement that supersedes it.
    let mut new_req = RememberRequest::new(
        "acme",
        "support",
        MemoryType::Semantic,
        "new fact about servers version two",
    );
    new_req.id = Some("srv-v2".into());
    new_req.supersedes = vec!["srv-v1".into()];
    db.remember(new_req).unwrap();

    // Default recall: only the new version should appear.
    let hits = db.recall(RecallRequest::new("acme", "servers")).unwrap();
    let ids: Vec<&str> = hits.iter().map(|h| h.id.as_str()).collect();
    assert!(
        ids.contains(&"srv-v2"),
        "new version should appear: {ids:?}"
    );
    assert!(
        !ids.contains(&"srv-v1"),
        "superseded old version should not appear: {ids:?}"
    );
}

#[test]
fn include_superseded_surfaces_history() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);

    let mut old_req = RememberRequest::new(
        "acme",
        "support",
        MemoryType::Semantic,
        "old fact about routers",
    );
    old_req.id = Some("rtr-v1".into());
    db.remember(old_req).unwrap();

    let mut new_req = RememberRequest::new(
        "acme",
        "support",
        MemoryType::Semantic,
        "new fact about routers",
    );
    new_req.id = Some("rtr-v2".into());
    new_req.supersedes = vec!["rtr-v1".into()];
    db.remember(new_req).unwrap();

    // With include_superseded=true both should appear.
    let mut req = RecallRequest::new("acme", "routers");
    req.include_superseded = true;
    let hits = db.recall(req).unwrap();
    let ids: Vec<&str> = hits.iter().map(|h| h.id.as_str()).collect();
    assert!(
        ids.contains(&"rtr-v2"),
        "new version should appear: {ids:?}"
    );
    assert!(
        ids.contains(&"rtr-v1"),
        "superseded version should appear with include_superseded: {ids:?}"
    );
}

#[test]
fn contradicts_surfaces_in_recall_result() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);

    let mut a_req = RememberRequest::new(
        "acme",
        "support",
        MemoryType::Semantic,
        "fact A about databases: they use port 5432",
    );
    a_req.id = Some("db-a".into());
    db.remember(a_req).unwrap();

    let mut b_req = RememberRequest::new(
        "acme",
        "support",
        MemoryType::Semantic,
        "fact B about databases: they use port 3306",
    );
    b_req.id = Some("db-b".into());
    b_req.contradicts = vec!["db-a".into()];
    db.remember(b_req).unwrap();

    let hits = db
        .recall(RecallRequest::new("acme", "databases port"))
        .unwrap();
    let b_hit = hits.iter().find(|h| h.id == "db-b");
    assert!(b_hit.is_some(), "db-b should appear in results");
    assert!(
        b_hit.unwrap().contradictions.contains(&"db-a".to_string()),
        "db-b result should carry db-a as contradiction: {:?}",
        b_hit.unwrap().contradictions
    );
}

#[test]
fn supersedes_unknown_id_returns_error() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);

    let mut req = RememberRequest::new(
        "acme",
        "support",
        MemoryType::Semantic,
        "memory with invalid supersede",
    );
    req.supersedes = vec!["nonexistent-id".into()];
    assert!(
        db.remember(req).is_err(),
        "superseding a nonexistent id should fail"
    );
}

#[test]
fn supersedes_survives_restart() {
    let dir = TempDir::new().unwrap();
    {
        let mut db = seeded(&dir);
        let mut old = RememberRequest::new(
            "acme",
            "support",
            MemoryType::Semantic,
            "old network config",
        );
        old.id = Some("net-v1".into());
        db.remember(old).unwrap();

        let mut new = RememberRequest::new(
            "acme",
            "support",
            MemoryType::Semantic,
            "new network config",
        );
        new.id = Some("net-v2".into());
        new.supersedes = vec!["net-v1".into()];
        db.remember(new).unwrap();
        db.close().unwrap();
    }
    // After recovery, superseded memory must still be excluded.
    let db = open(&dir);
    let hits = db
        .recall(RecallRequest::new("acme", "network config"))
        .unwrap();
    let ids: Vec<&str> = hits.iter().map(|h| h.id.as_str()).collect();
    assert!(
        ids.contains(&"net-v2"),
        "net-v2 should appear after restart: {ids:?}"
    );
    assert!(
        !ids.contains(&"net-v1"),
        "superseded net-v1 should not appear after restart: {ids:?}"
    );
}

// ── Context Compiler tests ────────────────────────────────────────────────────

#[test]
fn build_context_respects_token_budget() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);

    // Store several memories with distinct content.
    for i in 0..10 {
        remember(
            &mut db,
            "support",
            &format!("generic system fact number {i} about network configuration and routing"),
        );
    }

    // Request a very tight budget (32 tokens ≈ 128 bytes).
    let req = BuildContextRequest::new("acme", "network configuration", 32);
    let block = db.build_context(req).unwrap();

    // Text must respect the budget (allow one item's worth of slack).
    assert!(
        block.token_count <= 32 + 50,
        "token_count {} should be near 32",
        block.token_count
    );
    // Must have assembled something.
    assert!(
        !block.items_included.is_empty(),
        "should include at least one item"
    );
}

#[test]
fn build_context_ranks_by_score_and_includes_provenance() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);

    remember(
        &mut db,
        "support",
        "the default port for PostgreSQL is 5432",
    );
    remember(&mut db, "support", "the default port for MySQL is 3306");
    remember(
        &mut db,
        "support",
        "this memory is about something completely unrelated to databases",
    );

    let req = BuildContextRequest::new("acme", "postgresql port", 2048);
    let block = db.build_context(req).unwrap();

    assert!(!block.items_included.is_empty(), "should return results");

    // Items must be in descending score order.
    let scores: Vec<f32> = block.items_included.iter().map(|i| i.score).collect();
    for w in scores.windows(2) {
        assert!(
            w[0] >= w[1],
            "items must be sorted by score desc: {scores:?}"
        );
    }

    // Each included item must have a non-empty snippet.
    for item in &block.items_included {
        assert!(!item.snippet.is_empty(), "snippet must not be empty");
        assert!(
            item.token_count > 0,
            "token_count must be positive for non-empty text"
        );
    }

    // Assembled text must contain all included item ids.
    for item in &block.items_included {
        assert!(
            block.text.contains(&item.id),
            "text must contain item id {}: {}",
            item.id,
            block.text
        );
    }
}

#[test]
fn build_context_json_output_roundtrip() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);

    remember(&mut db, "support", "alpha bravo charlie delta system info");
    remember(&mut db, "support", "echo foxtrot golf hotel system info");

    let req = BuildContextRequest::new("acme", "system info", 2048);
    let block = db.build_context(req).unwrap();

    // ContextBlock fields must be self-consistent.
    assert_eq!(block.token_count, block.text.len().div_ceil(4));
    let included_tokens: usize = block.items_included.iter().map(|i| i.token_count).sum();
    // Sum of individual token counts may differ slightly from whole-text due to separator tokens.
    assert!(
        included_tokens <= block.token_count + block.items_included.len() * 5,
        "included token sum should be close to total"
    );
}

#[test]
fn build_context_writes_audit_record() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);
    remember(&mut db, "support", "postgresql connection pool tuning");

    let block = db
        .build_context(BuildContextRequest::new(
            "acme",
            "postgresql connection",
            2048,
        ))
        .unwrap();
    assert!(!block.items_included.is_empty());

    let audit_path = dir.path().join("audit.log");
    assert!(audit_path.exists(), "build_context must create audit.log");
    let raw = std::fs::read_to_string(audit_path).unwrap();
    let lines: Vec<&str> = raw.lines().collect();
    assert_eq!(lines.len(), 1, "one build_context call writes one record");

    let record: hippocore::AuditRecord = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(record.tenant_id, "acme");
    assert_eq!(record.query, "postgresql connection");
    assert_eq!(record.mode, "hybrid");
    assert_eq!(record.token_count, block.token_count);
    assert!(
        serde_json::from_str::<serde_json::Value>(lines[0]).unwrap()["latency_ms"].is_u64(),
        "audit record must serialize latency_ms"
    );
    assert!(
        !record.items.is_empty(),
        "audit must include retrieved items"
    );
    assert!(
        record.items.iter().any(|item| item.included),
        "audit must mark final context items"
    );
}

#[test]
fn query_audit_filters_by_time_range_and_tenant() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);
    db.create_tenant("other", "Other").unwrap();
    db.create_collection("other", "support", "").unwrap();
    remember(&mut db, "support", "postgresql backup restore runbook");
    db.remember(RememberRequest::new(
        "other",
        "support",
        MemoryType::Semantic,
        "oracle listener runbook",
    ))
    .unwrap();

    db.build_context(BuildContextRequest::new("acme", "postgresql backup", 2048))
        .unwrap();
    db.build_context(BuildContextRequest::new("other", "oracle listener", 2048))
        .unwrap();

    let all_acme = db.query_audit(0, i64::MAX, "acme").unwrap();
    assert_eq!(all_acme.len(), 1);
    assert_eq!(all_acme[0].tenant_id, "acme");
    assert_eq!(all_acme[0].query, "postgresql backup");

    let outside = db
        .query_audit(0, all_acme[0].timestamp_ms - 1, "acme")
        .unwrap();
    assert!(
        outside.is_empty(),
        "time range before the record must not return it"
    );
}

#[test]
fn query_audit_survives_reopen() {
    let dir = TempDir::new().unwrap();
    {
        let mut db = seeded(&dir);
        remember(&mut db, "support", "redis cache invalidation guide");
        db.build_context(BuildContextRequest::new("acme", "redis cache", 2048))
            .unwrap();
        db.close().unwrap();
    }

    let db = open(&dir);
    let records = db.query_audit(0, i64::MAX, "acme").unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].query, "redis cache");
    assert!(!records[0].items.is_empty());
}

// ── Graph Memory tests ───────────────────────────────────────────────────────

#[test]
fn graph_edge_adds_and_lists_direct_neighbors() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);
    remember_id(&mut db, "pg", "postgresql connection setup");
    remember_id(&mut db, "py", "python psycopg connection example");

    let edge = db
        .add_graph_edge(AddGraphEdgeRequest::new(
            "acme",
            ItemKind::Memory,
            "pg",
            ItemKind::Memory,
            "py",
            "mentions",
        ))
        .unwrap();

    let edges = db.list_graph_edges("acme", Some("pg"));
    assert_eq!(edges, vec![edge.clone()]);

    let neighbours = db.graph_neighbors("acme", ItemKind::Memory, "pg");
    assert_eq!(neighbours, vec![edge]);
}

#[test]
fn graph_edge_rejects_unknown_endpoint() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);
    remember_id(&mut db, "pg", "postgresql connection setup");

    let err = db
        .add_graph_edge(AddGraphEdgeRequest::new(
            "acme",
            ItemKind::Memory,
            "pg",
            ItemKind::Memory,
            "missing",
            "mentions",
        ))
        .unwrap_err();
    assert!(matches!(err, HippocoreError::Validation(_)));
}

#[test]
fn graph_edges_are_tenant_isolated() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);
    db.create_tenant("other", "Other").unwrap();
    db.create_collection("other", "support", "").unwrap();
    remember_id(&mut db, "pg", "postgresql connection setup");
    remember_id(&mut db, "py", "python psycopg connection example");
    db.remember(RememberRequest {
        id: Some("pg".into()),
        ..RememberRequest::new(
            "other",
            "support",
            MemoryType::Semantic,
            "other tenant postgresql setup",
        )
    })
    .unwrap();
    db.remember(RememberRequest {
        id: Some("py".into()),
        ..RememberRequest::new(
            "other",
            "support",
            MemoryType::Semantic,
            "other tenant python setup",
        )
    })
    .unwrap();

    db.add_graph_edge(AddGraphEdgeRequest::new(
        "acme",
        ItemKind::Memory,
        "pg",
        ItemKind::Memory,
        "py",
        "mentions",
    ))
    .unwrap();

    assert_eq!(db.list_graph_edges("acme", None).len(), 1);
    assert!(db.list_graph_edges("other", None).is_empty());
}

#[test]
fn graph_edges_survive_restart() {
    let dir = TempDir::new().unwrap();
    {
        let mut db = seeded(&dir);
        remember_id(&mut db, "pg", "postgresql connection setup");
        remember_id(&mut db, "py", "python psycopg connection example");
        db.add_graph_edge(AddGraphEdgeRequest::new(
            "acme",
            ItemKind::Memory,
            "pg",
            ItemKind::Memory,
            "py",
            "mentions",
        ))
        .unwrap();
        db.close().unwrap();
    }

    let db = open(&dir);
    let edges = db.list_graph_edges("acme", Some("pg"));
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].relation, "mentions");
}

#[test]
fn deleting_item_removes_dangling_graph_edges() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);
    remember_id(&mut db, "pg", "postgresql connection setup");
    remember_id(&mut db, "py", "python psycopg connection example");
    db.add_graph_edge(AddGraphEdgeRequest::new(
        "acme",
        ItemKind::Memory,
        "pg",
        ItemKind::Memory,
        "py",
        "mentions",
    ))
    .unwrap();

    db.forget("acme", "support", "py").unwrap();
    assert!(
        db.list_graph_edges("acme", None).is_empty(),
        "edges touching a deleted item must be removed"
    );
}

#[test]
fn build_context_and_audit_include_related_item_ids() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);
    remember_id(&mut db, "pg", "postgresql connection setup");
    remember_id(&mut db, "py", "python psycopg connection example");
    db.add_graph_edge(AddGraphEdgeRequest::new(
        "acme",
        ItemKind::Memory,
        "pg",
        ItemKind::Memory,
        "py",
        "mentions",
    ))
    .unwrap();

    let block = db
        .build_context(BuildContextRequest::new(
            "acme",
            "postgresql connection",
            2048,
        ))
        .unwrap();
    let pg_item = block
        .items_included
        .iter()
        .find(|item| item.id == "pg")
        .expect("pg item must be included");
    assert_eq!(pg_item.related_item_ids, vec!["py".to_string()]);

    let audit = db.query_audit(0, i64::MAX, "acme").unwrap();
    let pg_audit_item = audit[0]
        .items
        .iter()
        .find(|item| item.id == "pg")
        .expect("pg audit item must be present");
    assert_eq!(pg_audit_item.related_item_ids, vec!["py".to_string()]);
}

// ── Batch write / group-commit tests ─────────────────────────────────────────

#[test]
fn remember_many_stores_all_in_one_batch() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);

    let reqs: Vec<RememberRequest> = (0..5)
        .map(|i| {
            RememberRequest::new(
                "acme",
                "support",
                MemoryType::Semantic,
                format!("batch memory item number {i} about network routing"),
            )
        })
        .collect();

    let memories = db.remember_many(reqs).unwrap();
    assert_eq!(memories.len(), 5);

    let stats = db.stats().unwrap();
    assert_eq!(stats.memories, 5);

    // All 5 should be recallable.
    let hits = db
        .recall(RecallRequest::new("acme", "network routing batch"))
        .unwrap();
    assert!(
        hits.len() >= 5,
        "all batch memories should be indexed: {}",
        hits.len()
    );
}

#[test]
fn store_documents_stores_all_in_one_batch() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);

    let reqs: Vec<StoreDocumentRequest> = (0..3)
        .map(|i| {
            StoreDocumentRequest::new(
                "acme",
                "support",
                format!("batch document {i} about postgresql database configuration"),
            )
        })
        .collect();

    let docs = db.store_documents(reqs).unwrap();
    assert_eq!(docs.len(), 3);

    let stats = db.stats().unwrap();
    assert_eq!(stats.documents, 3);
    assert!(stats.chunks >= 3);

    let hits = db
        .recall(RecallRequest::new("acme", "postgresql database"))
        .unwrap();
    assert!(!hits.is_empty(), "batch documents should be recallable");
}

#[test]
fn batch_writes_survive_restart() {
    let dir = TempDir::new().unwrap();
    {
        let mut db = seeded(&dir);
        let reqs: Vec<RememberRequest> = (0..4)
            .map(|i| {
                RememberRequest::new(
                    "acme",
                    "support",
                    MemoryType::Semantic,
                    format!("durable batch item {i} about oracle database listener"),
                )
            })
            .collect();
        db.remember_many(reqs).unwrap();
        db.close().unwrap();
    }

    let db = open(&dir);
    assert_eq!(
        db.stats().unwrap().memories,
        4,
        "all batch memories must survive restart"
    );
    let hits = db
        .recall(RecallRequest::new("acme", "oracle listener"))
        .unwrap();
    assert!(
        !hits.is_empty(),
        "batch memories must be recallable after restart"
    );
}

// ─── Phase 7: confidence-aware resolution ────────────────────────────────────

#[test]
fn remember_with_confidence_stores_and_recalls_correctly() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);

    let mut req = RememberRequest::new(
        "acme",
        "support",
        MemoryType::Semantic,
        "redis caching strategy",
    );
    req.confidence = Some(0.9);
    let mem = db.remember(req).unwrap();
    assert_eq!(mem.confidence, Some(0.9));

    let hits = db
        .recall(RecallRequest::new("acme", "redis caching"))
        .unwrap();
    let found = hits.iter().find(|h| h.id == mem.id).unwrap();
    assert_eq!(found.confidence, Some(0.9));
}

#[test]
fn confidence_out_of_range_is_rejected() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);

    let mut req = RememberRequest::new("acme", "support", MemoryType::Note, "some fact");
    req.confidence = Some(1.5);
    let err = db.remember(req).unwrap_err();
    assert!(
        matches!(err, HippocoreError::Validation(_)),
        "confidence > 1.0 should fail validation"
    );

    let mut req2 = RememberRequest::new("acme", "support", MemoryType::Note, "another fact");
    req2.confidence = Some(-0.1);
    let err2 = db.remember(req2).unwrap_err();
    assert!(matches!(err2, HippocoreError::Validation(_)));
}

#[test]
fn rate_memory_updates_confidence() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);

    let mem = db
        .remember(RememberRequest::new(
            "acme",
            "support",
            MemoryType::Semantic,
            "postgresql connection pooling",
        ))
        .unwrap();
    assert_eq!(mem.confidence, None);

    let updated = db.rate_memory("acme", "support", &mem.id, 0.85).unwrap();
    assert_eq!(updated.confidence, Some(0.85));

    // Verify it persists through re-open.
    db.close().unwrap();
    let db2 = open(&dir);
    let hits = db2
        .recall(RecallRequest::new("acme", "postgresql connection"))
        .unwrap();
    let found = hits.iter().find(|h| h.id == mem.id).unwrap();
    assert_eq!(found.confidence, Some(0.85));
}

#[test]
fn rate_memory_invalid_confidence_is_rejected() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);

    let mem = db
        .remember(RememberRequest::new(
            "acme",
            "support",
            MemoryType::Note,
            "some note",
        ))
        .unwrap();

    let err = db.rate_memory("acme", "support", &mem.id, 2.0).unwrap_err();
    assert!(matches!(err, HippocoreError::Validation(_)));
}

#[test]
fn rate_memory_nonexistent_returns_not_found() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);

    let err = db
        .rate_memory("acme", "support", "nonexistent-id", 0.5)
        .unwrap_err();
    assert!(matches!(err, HippocoreError::NotFound(_)));
}

#[test]
fn confidence_aware_context_prefers_higher_confidence_on_conflict() {
    let dir = TempDir::new().unwrap();
    let mut db = seeded(&dir);

    // Store two memories that contradict each other: same topic, opposite claims.
    let mut high = RememberRequest::new(
        "acme",
        "support",
        MemoryType::Semantic,
        "database server hostname is primary.example.internal",
    );
    high.confidence = Some(0.95);
    let hi_mem = db.remember(high).unwrap();

    let mut low = RememberRequest::new(
        "acme",
        "support",
        MemoryType::Semantic,
        "database server hostname is replica.example.internal",
    );
    low.contradicts = vec![hi_mem.id.clone()];
    low.confidence = Some(0.3);
    db.remember(low).unwrap();

    let block = db
        .build_context(BuildContextRequest {
            tenant_id: "acme".into(),
            query: "database server hostname".into(),
            user_id: None,
            max_tokens: 4096,
            top_k_candidates: 10,
            mode: SearchMode::Hybrid,
            collection: None,
            metadata_filter: Default::default(),
        })
        .unwrap();

    // The high-confidence item must appear before the low-confidence one.
    let positions: Vec<usize> = block
        .items_included
        .iter()
        .enumerate()
        .filter_map(
            |(i, item)| {
                if item.id == hi_mem.id {
                    Some(i)
                } else {
                    None
                }
            },
        )
        .collect();
    assert!(
        !positions.is_empty(),
        "high-confidence memory must be included in context"
    );
    let hi_pos = positions[0];
    let lo_pos = block
        .items_included
        .iter()
        .enumerate()
        .find_map(|(i, item)| {
            if item.confidence == Some(0.3) {
                Some(i)
            } else {
                None
            }
        });
    if let Some(lo) = lo_pos {
        assert!(
            hi_pos < lo,
            "high-confidence item (pos={hi_pos}) must come before low-confidence (pos={lo})"
        );
    }
}

// ─── Phase 4: pluggable VectorIndex trait + HNSW ─────────────────────────────

#[test]
fn hnsw_backend_stores_and_recalls() {
    let dir = TempDir::new().unwrap();
    let mut cfg = Config::new(dir.path());
    cfg.vector_index = hippocore::VectorIndexKind::Hnsw;
    let mut db = Hippocore::open(cfg).unwrap();
    db.create_tenant("acme", "Acme Corp").unwrap();
    db.create_collection("acme", "knowledge", "").unwrap();

    // Store several memories so the HNSW graph has at least a few nodes.
    let texts = [
        "postgresql database replication streaming setup",
        "redis pub sub message broker configuration",
        "kafka topic partition consumer group offset",
        "elasticsearch index mapping analyzer settings",
        "mongodb aggregation pipeline stages operators",
    ];
    for t in &texts {
        db.remember(RememberRequest::new(
            "acme",
            "knowledge",
            MemoryType::Semantic,
            *t,
        ))
        .unwrap();
    }

    let hits = db
        .recall(RecallRequest::new("acme", "postgresql replication"))
        .unwrap();
    assert!(
        !hits.is_empty(),
        "HNSW backend must return at least one result"
    );
    // The most relevant result should mention postgresql or replication.
    let top = &hits[0];
    assert!(
        top.text.contains("postgresql") || top.text.contains("replication"),
        "top hit should be the postgresql memory, got: {}",
        top.text
    );
}

#[test]
fn hnsw_backend_survives_restart() {
    let dir = TempDir::new().unwrap();
    {
        let mut cfg = Config::new(dir.path());
        cfg.vector_index = hippocore::VectorIndexKind::Hnsw;
        let mut db = Hippocore::open(cfg).unwrap();
        db.create_tenant("t1", "Tenant 1").unwrap();
        db.create_collection("t1", "col", "").unwrap();
        for i in 0..8u32 {
            db.remember(RememberRequest::new(
                "t1",
                "col",
                MemoryType::Note,
                format!("item {i}: oracle listener service control command restart"),
            ))
            .unwrap();
        }
        db.close().unwrap();
    }

    // Re-open with HNSW (index is rebuilt from WAL/snapshot).
    let mut cfg = Config::new(dir.path());
    cfg.vector_index = hippocore::VectorIndexKind::Hnsw;
    let db = Hippocore::open(cfg).unwrap();
    assert_eq!(
        db.stats().unwrap().memories,
        8,
        "all memories must survive restart"
    );
    let hits = db
        .recall(RecallRequest::new("t1", "oracle listener restart"))
        .unwrap();
    assert!(
        !hits.is_empty(),
        "HNSW index must be queryable after restart"
    );
}

#[test]
fn bruteforce_and_hnsw_both_recall_relevant_item() {
    // Verifies that HNSW and brute-force agree on retrieving the most relevant
    // item for a pure vector-mode query. We use Vector mode so that only the
    // vector backend (not RRF rank fusion with BM25) determines the result.
    let dir_bf = TempDir::new().unwrap();
    let dir_hnsw = TempDir::new().unwrap();

    let setup = |dir: &TempDir, kind: hippocore::VectorIndexKind| {
        let mut cfg = Config::new(dir.path());
        cfg.vector_index = kind;
        let mut db = Hippocore::open(cfg).unwrap();
        db.create_tenant("t", "T").unwrap();
        db.create_collection("t", "c", "").unwrap();
        let items = [
            "nginx reverse proxy load balancing upstream server",
            "apache httpd virtual host configuration ssl certificate",
            "haproxy tcp udp load balancer health check frontend backend",
            "caddy automatic https tls acme certificate authority",
            "traefik docker swarm ingress router middleware service",
        ];
        for text in &items {
            db.remember(RememberRequest::new("t", "c", MemoryType::Semantic, *text))
                .unwrap();
        }
        db
    };

    let bf = setup(&dir_bf, hippocore::VectorIndexKind::BruteForce);
    let hnsw = setup(&dir_hnsw, hippocore::VectorIndexKind::Hnsw);

    let query = "nginx reverse proxy";
    let mut req_bf = RecallRequest::new("t", query);
    req_bf.mode = SearchMode::Vector;
    req_bf.top_k = 5;
    let mut req_hnsw = RecallRequest::new("t", query);
    req_hnsw.mode = SearchMode::Vector;
    req_hnsw.top_k = 5;

    let hits_bf = bf.recall(req_bf).unwrap();
    let hits_hnsw = hnsw.recall(req_hnsw).unwrap();

    // Both backends must return the nginx memory somewhere in their results.
    assert!(!hits_bf.is_empty(), "brute-force must return results");
    assert!(!hits_hnsw.is_empty(), "HNSW must return results");

    let _nginx_id = hits_bf
        .iter()
        .find(|h| h.text.contains("nginx"))
        .expect("brute-force must find the nginx memory")
        .id
        .clone();

    assert!(
        hits_bf[0].text.contains("nginx"),
        "brute-force top-1 must be the nginx memory, got: {}",
        hits_bf[0].text
    );
    assert!(
        hits_hnsw.iter().any(|h| h.text.contains("nginx")),
        "HNSW must recall the nginx memory in its results (got: {:?})",
        hits_hnsw
            .iter()
            .map(|h| h.text.as_str())
            .collect::<Vec<_>>()
    );
}
