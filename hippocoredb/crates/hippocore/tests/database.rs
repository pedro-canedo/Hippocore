//! Integration tests for the Hippocore memory database.

use hippocore::model::{ItemKind, MemoryType};
use hippocore::{
    ChunkInput, Config, Hippocore, HippocoreError, ImportFileRequest, PutRecordRequest,
    RecallRequest, RememberRequest, SearchMode, StoreDocumentRequest,
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
