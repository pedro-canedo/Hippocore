//! Integration tests for the Hippocore memory database.

use hippocore::model::{ItemKind, MemoryType};
use hippocore::{
    ChunkInput, Config, Hippocore, HippocoreError, RecallRequest, RememberRequest, SearchMode,
    StoreDocumentRequest,
};
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
    assert!(hits[0].reason.contains("boost: query targets postgresql"));
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
    assert!(hits[0].reason.contains("boost: query targets oracle"));
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
    assert!(hits
        .iter()
        .any(|h| h.reason.contains("python+postgresql connection")));
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
