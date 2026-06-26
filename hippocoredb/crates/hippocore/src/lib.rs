//! # Hippocore DB
//!
//! Hippocore DB is an open-source, **local-first** AI-native database that acts
//! as long-term **memory and contextual retrieval** infrastructure for agents,
//! copilots, RAG applications and multi-tenant AI systems.
//!
//! It is **not just a vector store**: it persists documents and memories
//! durably, isolates data by tenant, and recalls context through **vector**,
//! **text**, and **hybrid** search.
//!
//! ```no_run
//! use hippocore::{Config, Hippocore, RememberRequest, RecallRequest};
//! use hippocore::model::MemoryType;
//!
//! # fn main() -> hippocore::Result<()> {
//! let mut db = Hippocore::open(Config::new("./hippocore-data"))?;
//! db.create_tenant("acme", "Acme Corp")?;
//! db.create_collection("acme", "support", "support knowledge")?;
//!
//! db.remember(RememberRequest::new("acme", "support", MemoryType::Semantic,
//!     "A customer runs Oracle in the staging environment"))?;
//!
//! let hits = db.recall(RecallRequest::new("acme", "oracle staging environment"))?;
//! assert!(!hits.is_empty());
//! db.close()?;
//! # Ok(())
//! # }
//! ```

pub mod config;
pub mod errors;
pub mod index;
pub mod memory;
pub mod model;
pub mod query;
pub mod storage;

pub mod cli;

use std::fmt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub use config::Config;
pub use errors::{HippocoreError, Result};
pub use model::{
    Chunk, Collection, Document, Embedding, FileObject, ItemKind, Memory, MemoryType, Metadata,
    RecallResult, Record, Source, Tenant,
};
pub use query::SearchMode;

use index::{Index, IndexEntry};
use query::{Filter, QueryRequest};
use storage::{Operation, State, Storage};

/// The main database handle.
pub struct Hippocore {
    config: Config,
    embedder: memory::Embedder,
    storage: Storage,
    state: State,
    index: Index,
}

/// A point-in-time summary of database contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseStats {
    /// Registered tenants.
    pub tenants: usize,
    /// Registered collections.
    pub collections: usize,
    /// Stored documents.
    pub documents: usize,
    /// Stored chunks (searchable document units).
    pub chunks: usize,
    /// Stored memories.
    pub memories: usize,
    /// Stored structured records.
    pub records: usize,
    /// Imported file objects.
    pub files: usize,
    /// Total entries in the retrieval index (chunks + memories + records).
    pub indexed_entries: usize,
    /// Operations currently in the WAL (since the last compaction).
    pub wal_entries: usize,
    /// Bytes used on disk by WAL + snapshot.
    pub disk_bytes: u64,
}

impl fmt::Display for DatabaseStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Hippocore DB stats")?;
        writeln!(f, "  tenants:         {}", self.tenants)?;
        writeln!(f, "  collections:     {}", self.collections)?;
        writeln!(f, "  documents:       {}", self.documents)?;
        writeln!(f, "  chunks:          {}", self.chunks)?;
        writeln!(f, "  memories:        {}", self.memories)?;
        writeln!(f, "  records:         {}", self.records)?;
        writeln!(f, "  files:           {}", self.files)?;
        writeln!(f, "  indexed entries: {}", self.indexed_entries)?;
        writeln!(f, "  wal entries:     {}", self.wal_entries)?;
        write!(f, "  disk bytes:      {}", self.disk_bytes)
    }
}

/// A caller-supplied chunk: text plus its precomputed embedding.
#[derive(Debug, Clone)]
pub struct ChunkInput {
    /// Chunk text.
    pub text: String,
    /// Embedding for this chunk (e.g. from an external model).
    pub embedding: Embedding,
}

impl ChunkInput {
    /// Build a chunk input from text and an embedding.
    pub fn new(text: impl Into<String>, embedding: Embedding) -> Self {
        Self {
            text: text.into(),
            embedding,
        }
    }
}

/// Request to store a document.
///
/// By default the text is chunked and embedded with the built-in embedder. To
/// use embeddings from an external model, set [`chunks`](Self::chunks) with
/// pre-embedded [`ChunkInput`]s; then `text` is stored as the document body but
/// chunking/embedding is taken entirely from the supplied chunks.
#[derive(Debug, Clone)]
pub struct StoreDocumentRequest {
    /// Owning tenant (must exist).
    pub tenant_id: String,
    /// Owning collection (must exist).
    pub collection: String,
    /// Optional explicit id; generated if `None`.
    pub id: Option<String>,
    /// Full document text.
    pub text: String,
    /// Exact-match metadata.
    pub metadata: Metadata,
    /// Optional provenance.
    pub source: Option<Source>,
    /// Optional caller-supplied, pre-embedded chunks. When `None`, the document
    /// is auto-chunked and embedded with the built-in embedder.
    pub chunks: Option<Vec<ChunkInput>>,
}

impl StoreDocumentRequest {
    /// Build a minimal request (auto chunk + embed).
    pub fn new(
        tenant_id: impl Into<String>,
        collection: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            collection: collection.into(),
            id: None,
            text: text.into(),
            metadata: Metadata::new(),
            source: None,
            chunks: None,
        }
    }
}

/// Request to store a memory.
#[derive(Debug, Clone)]
pub struct RememberRequest {
    /// Owning tenant (must exist).
    pub tenant_id: String,
    /// Owning collection (must exist).
    pub collection: String,
    /// Optional explicit id; generated if `None`.
    pub id: Option<String>,
    /// Optional owning user / agent.
    pub user_id: Option<String>,
    /// Memory category.
    pub memory_type: MemoryType,
    /// Memory content.
    pub text: String,
    /// Exact-match metadata.
    pub metadata: Metadata,
    /// Optional provenance.
    pub source: Option<Source>,
    /// Optional caller-supplied embedding (else the built-in embedder is used).
    pub embedding: Option<Embedding>,
}

impl RememberRequest {
    /// Build a minimal request.
    pub fn new(
        tenant_id: impl Into<String>,
        collection: impl Into<String>,
        memory_type: MemoryType,
        text: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            collection: collection.into(),
            id: None,
            user_id: None,
            memory_type,
            text: text.into(),
            metadata: Metadata::new(),
            source: None,
            embedding: None,
        }
    }
}

/// Request to store a structured JSON record.
#[derive(Debug, Clone)]
pub struct PutRecordRequest {
    /// Owning tenant (must exist).
    pub tenant_id: String,
    /// Owning collection (must exist).
    pub collection: String,
    /// Logical table / dataset namespace.
    pub table: String,
    /// Optional explicit id; generated if `None`.
    pub id: Option<String>,
    /// Original JSON payload. Must be an object.
    pub payload: serde_json::Value,
    /// Exact-match metadata.
    pub metadata: Metadata,
    /// Optional provenance.
    pub source: Option<Source>,
    /// Optional caller-supplied embedding for the projection.
    pub embedding: Option<Embedding>,
}

impl PutRecordRequest {
    /// Build a minimal structured-record request.
    pub fn new(
        tenant_id: impl Into<String>,
        collection: impl Into<String>,
        table: impl Into<String>,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            collection: collection.into(),
            table: table.into(),
            id: None,
            payload,
            metadata: Metadata::new(),
            source: None,
            embedding: None,
        }
    }
}

/// Request to recall/search context.
#[derive(Debug, Clone)]
pub struct RecallRequest {
    /// Tenant to search within (mandatory; enforces isolation).
    pub tenant_id: String,
    /// Natural-language query text.
    pub query: String,
    /// Optional caller-supplied query embedding (overrides text embedding).
    pub embedding: Option<Embedding>,
    /// Restrict to a collection.
    pub collection: Option<String>,
    /// Restrict to an owning user.
    pub user_id: Option<String>,
    /// Restrict to a memory type.
    pub memory_type: Option<MemoryType>,
    /// Restrict to an item kind (document chunks or memories).
    pub kind: Option<ItemKind>,
    /// Require these exact metadata pairs.
    pub metadata: Metadata,
    /// Ranking mode (used by [`Hippocore::search`]; `recall` forces hybrid).
    pub mode: SearchMode,
    /// Maximum results.
    pub top_k: usize,
}

impl RecallRequest {
    /// Build a minimal hybrid recall request.
    pub fn new(tenant_id: impl Into<String>, query: impl Into<String>) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            query: query.into(),
            embedding: None,
            collection: None,
            user_id: None,
            memory_type: None,
            kind: None,
            metadata: Metadata::new(),
            mode: SearchMode::Hybrid,
            top_k: 10,
        }
    }
}

impl Hippocore {
    /// Open (creating if needed) a database at `config.data_dir`, recovering any
    /// previously persisted state.
    pub fn open(config: Config) -> Result<Self> {
        let (storage, state) = Storage::open(&config.data_dir, config.sync_writes)?;
        let embedder = memory::Embedder::new(config.embedding_dim);
        let mut db = Self {
            config,
            embedder,
            storage,
            state,
            index: Index::new(),
        };
        db.rebuild_index();
        Ok(db)
    }

    fn rebuild_index(&mut self) {
        self.index = Index::new();
        // Index document chunks, carrying their parent document's metadata/source.
        for chunk in &self.state.chunks {
            let parent = self
                .state
                .documents
                .iter()
                .find(|d| d.id == chunk.document_id);
            let (metadata, source) = parent
                .map(|d| (d.metadata.clone(), d.source.clone()))
                .unwrap_or_default();
            self.index
                .insert(IndexEntry::from_chunk(chunk, metadata, source));
        }
        for memory in &self.state.memories {
            self.index.insert(IndexEntry::from_memory(memory));
        }
        for record in &self.state.records {
            self.index.insert(IndexEntry::from_record(record));
        }
    }

    /// Register a tenant (idempotent: returns the existing tenant if present).
    pub fn create_tenant(
        &mut self,
        id: impl Into<String>,
        name: impl Into<String>,
    ) -> Result<Tenant> {
        let id = id.into();
        let tenant = Tenant {
            id: id.clone(),
            name: name.into(),
            created_at: now_millis(),
        };
        tenant.validate()?;
        if let Some(existing) = self.state.tenants.iter().find(|t| t.id == id) {
            return Ok(existing.clone());
        }
        self.commit(Operation::CreateTenant(tenant.clone()))?;
        Ok(tenant)
    }

    /// Register a collection under an existing tenant (idempotent).
    pub fn create_collection(
        &mut self,
        tenant_id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Result<Collection> {
        let tenant_id = tenant_id.into();
        let name = name.into();
        self.require_tenant(&tenant_id)?;
        let collection = Collection {
            name: name.clone(),
            tenant_id: tenant_id.clone(),
            description: description.into(),
            created_at: now_millis(),
        };
        collection.validate()?;
        if let Some(existing) = self
            .state
            .collections
            .iter()
            .find(|c| c.tenant_id == tenant_id && c.name == name)
        {
            return Ok(existing.clone());
        }
        self.commit(Operation::CreateCollection(collection.clone()))?;
        Ok(collection)
    }

    /// Store (or overwrite) a document, chunking and embedding its text.
    pub fn store_document(&mut self, req: StoreDocumentRequest) -> Result<Document> {
        self.require_collection(&req.tenant_id, &req.collection)?;

        let id = req.id.unwrap_or_else(|| gen_id("doc"));
        let now = now_millis();
        let (created_at, version) = match self.find_document(&req.tenant_id, &req.collection, &id) {
            Some(existing) => (existing.created_at, existing.version + 1),
            None => (now, 0),
        };

        let document = Document {
            id: id.clone(),
            tenant_id: req.tenant_id.clone(),
            collection: req.collection.clone(),
            text: req.text.clone(),
            metadata: req.metadata,
            source: req.source,
            created_at,
            updated_at: now,
            version,
        };
        document.validate()?;

        // Either use caller-supplied per-chunk embeddings, or auto chunk+embed.
        let chunks = match req.chunks {
            Some(inputs) => self.build_chunks_from_inputs(&document, inputs)?,
            None => self.build_chunks(&document),
        };
        self.commit(Operation::PutDocument {
            document: document.clone(),
            chunks,
        })?;
        Ok(document)
    }

    /// Store (or overwrite) a memory.
    pub fn remember(&mut self, req: RememberRequest) -> Result<Memory> {
        self.require_collection(&req.tenant_id, &req.collection)?;

        let id = req.id.unwrap_or_else(|| gen_id("mem"));
        let created_at = self
            .state
            .memories
            .iter()
            .find(|m| m.tenant_id == req.tenant_id && m.collection == req.collection && m.id == id)
            .map(|m| m.created_at)
            .unwrap_or_else(now_millis);

        let embedding = match req.embedding {
            Some(e) => {
                if e.is_empty() {
                    return Err(HippocoreError::InvalidEmbedding(
                        "supplied embedding is empty".into(),
                    ));
                }
                e
            }
            None => self.embedder.embed(&req.text),
        };

        let mem = Memory {
            id,
            tenant_id: req.tenant_id,
            collection: req.collection,
            user_id: req.user_id,
            memory_type: req.memory_type,
            text: req.text,
            embedding,
            metadata: req.metadata,
            source: req.source,
            created_at,
        };
        mem.validate()?;
        self.commit(Operation::PutMemory(mem.clone()))?;
        Ok(mem)
    }

    /// Store (or overwrite) a structured JSON record.
    pub fn put_record(&mut self, req: PutRecordRequest) -> Result<Record> {
        self.require_collection(&req.tenant_id, &req.collection)?;

        let id = req.id.unwrap_or_else(|| gen_id("rec"));
        let now = now_millis();
        let (created_at, version) =
            match self.find_record(&req.tenant_id, &req.collection, &req.table, &id) {
                Some(existing) => (existing.created_at, existing.version + 1),
                None => (now, 0),
            };
        let projection = project_record(&req.table, &id, &req.payload)?;
        let embedding = match req.embedding {
            Some(e) => {
                if e.is_empty() {
                    return Err(HippocoreError::InvalidEmbedding(
                        "record embedding must not be empty".into(),
                    ));
                }
                e
            }
            None => self.embedder.embed(&projection),
        };

        let record = Record {
            id,
            tenant_id: req.tenant_id,
            collection: req.collection,
            table: req.table,
            payload: req.payload,
            projection,
            embedding,
            metadata: req.metadata,
            source: req.source,
            created_at,
            updated_at: now,
            version,
        };
        record.validate()?;
        self.commit(Operation::PutRecord(record.clone()))?;
        Ok(record)
    }

    /// Forget (delete) a memory by identity.
    ///
    /// The removal is durable (a tombstone is written to the WAL and replayed on
    /// recovery). Forgetting a memory that does not exist is a successful no-op,
    /// so the call is idempotent and never corrupts state.
    pub fn forget(&mut self, tenant_id: &str, collection: &str, id: &str) -> Result<()> {
        self.commit(Operation::DeleteMemory {
            tenant_id: tenant_id.to_string(),
            collection: collection.to_string(),
            id: id.to_string(),
        })
    }

    /// Delete a document and all of its chunks by identity.
    ///
    /// Durable and idempotent, like [`Hippocore::forget`].
    pub fn delete_document(&mut self, tenant_id: &str, collection: &str, id: &str) -> Result<()> {
        self.commit(Operation::DeleteDocument {
            tenant_id: tenant_id.to_string(),
            collection: collection.to_string(),
            id: id.to_string(),
        })
    }

    /// Delete a structured record by identity.
    ///
    /// Durable and idempotent, like [`Hippocore::forget`].
    pub fn delete_record(
        &mut self,
        tenant_id: &str,
        collection: &str,
        table: &str,
        id: &str,
    ) -> Result<()> {
        self.commit(Operation::DeleteRecord {
            tenant_id: tenant_id.to_string(),
            collection: collection.to_string(),
            table: table.to_string(),
            id: id.to_string(),
        })
    }

    /// Hybrid contextual recall (vector + text) within a tenant.
    pub fn recall(&self, mut req: RecallRequest) -> Result<Vec<RecallResult>> {
        req.mode = SearchMode::Hybrid;
        self.run_query(req)
    }

    /// Search within a tenant using the request's [`SearchMode`].
    pub fn search(&self, req: RecallRequest) -> Result<Vec<RecallResult>> {
        self.run_query(req)
    }

    fn run_query(&self, req: RecallRequest) -> Result<Vec<RecallResult>> {
        if req.tenant_id.trim().is_empty() {
            return Err(HippocoreError::validation("recall requires a tenant_id"));
        }
        if req.mode != SearchMode::Vector && req.query.trim().is_empty() && req.embedding.is_none()
        {
            return Err(HippocoreError::validation(
                "recall requires a query text or embedding",
            ));
        }
        let filter = Filter {
            tenant_id: req.tenant_id,
            collection: req.collection,
            user_id: req.user_id,
            memory_type: req.memory_type,
            kind: req.kind,
            metadata: req.metadata,
        };
        let query_text = if req.query.trim().is_empty() {
            None
        } else {
            Some(req.query)
        };
        let request = QueryRequest {
            filter,
            query_text,
            query_embedding: req.embedding,
            mode: req.mode,
            hybrid_alpha: self.config.hybrid_alpha,
            top_k: req.top_k,
        };
        Ok(query::execute(
            &self.index,
            |t| self.embedder.embed(t),
            &request,
        ))
    }

    /// A snapshot of database metrics.
    pub fn stats(&self) -> Result<DatabaseStats> {
        Ok(DatabaseStats {
            tenants: self.state.tenants.len(),
            collections: self.state.collections.len(),
            documents: self.state.documents.len(),
            chunks: self.state.chunks.len(),
            memories: self.state.memories.len(),
            records: self.state.records.len(),
            indexed_entries: self.index.len(),
            wal_entries: self.storage.wal_len,
            disk_bytes: self.storage.disk_bytes()?,
        })
    }

    /// List tenants (clones).
    pub fn tenants(&self) -> Vec<Tenant> {
        self.state.tenants.clone()
    }

    /// List collections for a tenant.
    pub fn collections(&self, tenant_id: &str) -> Vec<Collection> {
        self.state
            .collections
            .iter()
            .filter(|c| c.tenant_id == tenant_id)
            .cloned()
            .collect()
    }

    /// Fold the WAL into a fresh snapshot and truncate the log.
    pub fn compact(&mut self) -> Result<()> {
        self.storage.compact(&self.state)
    }

    /// Flush and `fsync`, then consume the handle.
    pub fn close(mut self) -> Result<()> {
        self.storage.sync()
    }

    /// The active configuration.
    pub fn config(&self) -> &Config {
        &self.config
    }

    // --- internals ---

    fn commit(&mut self, op: Operation) -> Result<()> {
        self.storage.append(&op)?;
        // Update in-memory state and index.
        if let Operation::PutDocument { document, .. } = &op {
            self.index
                .remove_document(&document.tenant_id, &document.collection, &document.id);
        }
        let to_index = op.clone();
        self.state.apply(op);
        self.index_op(&to_index);

        // Auto-compaction keeps the WAL (and recovery time) bounded.
        if self.should_auto_compact() {
            self.storage.compact(&self.state)?;
        }
        Ok(())
    }

    fn should_auto_compact(&self) -> bool {
        let ops = self.config.auto_compact_after_ops;
        let bytes = self.config.auto_compact_after_bytes;
        (ops > 0 && self.storage.wal_len >= ops) || (bytes > 0 && self.storage.wal_bytes >= bytes)
    }

    fn index_op(&mut self, op: &Operation) {
        match op {
            Operation::PutDocument { document, chunks } => {
                for chunk in chunks {
                    self.index.insert(IndexEntry::from_chunk(
                        chunk,
                        document.metadata.clone(),
                        document.source.clone(),
                    ));
                }
            }
            Operation::PutMemory(m) => {
                self.index.insert(IndexEntry::from_memory(m));
            }
            Operation::PutRecord(r) => {
                self.index.insert(IndexEntry::from_record(r));
            }
            Operation::DeleteMemory {
                tenant_id,
                collection,
                id,
            } => {
                self.index.remove_memory(tenant_id, collection, id);
            }
            Operation::DeleteDocument {
                tenant_id,
                collection,
                id,
            } => {
                self.index.remove_document(tenant_id, collection, id);
            }
            Operation::DeleteRecord {
                tenant_id,
                collection,
                table,
                id,
            } => {
                self.index.remove_record(tenant_id, collection, table, id);
            }
            Operation::CreateTenant(_) | Operation::CreateCollection(_) => {}
        }
    }

    fn build_chunks(&self, document: &Document) -> Vec<Chunk> {
        memory::chunk_text(&document.text, self.config.chunk_tokens)
            .into_iter()
            .enumerate()
            .map(|(ordinal, text)| {
                let embedding = self.embedder.embed(&text);
                Chunk {
                    id: format!("{}#{}", document.id, ordinal),
                    document_id: document.id.clone(),
                    tenant_id: document.tenant_id.clone(),
                    collection: document.collection.clone(),
                    ordinal,
                    text,
                    embedding,
                }
            })
            .collect()
    }

    /// Build chunks from caller-supplied text + embeddings, validating each.
    fn build_chunks_from_inputs(
        &self,
        document: &Document,
        inputs: Vec<ChunkInput>,
    ) -> Result<Vec<Chunk>> {
        inputs
            .into_iter()
            .enumerate()
            .map(|(ordinal, input)| {
                if input.text.trim().is_empty() {
                    return Err(HippocoreError::validation("chunk text must not be empty"));
                }
                if input.embedding.is_empty() {
                    return Err(HippocoreError::InvalidEmbedding(
                        "chunk embedding must not be empty".into(),
                    ));
                }
                Ok(Chunk {
                    id: format!("{}#{}", document.id, ordinal),
                    document_id: document.id.clone(),
                    tenant_id: document.tenant_id.clone(),
                    collection: document.collection.clone(),
                    ordinal,
                    text: input.text,
                    embedding: input.embedding,
                })
            })
            .collect()
    }

    fn require_tenant(&self, tenant_id: &str) -> Result<()> {
        if self.state.tenants.iter().any(|t| t.id == tenant_id) {
            Ok(())
        } else {
            Err(HippocoreError::UnknownTenant(tenant_id.to_string()))
        }
    }

    fn require_collection(&self, tenant_id: &str, collection: &str) -> Result<()> {
        self.require_tenant(tenant_id)?;
        if self
            .state
            .collections
            .iter()
            .any(|c| c.tenant_id == tenant_id && c.name == collection)
        {
            Ok(())
        } else {
            Err(HippocoreError::UnknownCollection {
                tenant: tenant_id.to_string(),
                collection: collection.to_string(),
            })
        }
    }

    fn find_document(&self, tenant_id: &str, collection: &str, id: &str) -> Option<&Document> {
        self.state
            .documents
            .iter()
            .find(|d| d.tenant_id == tenant_id && d.collection == collection && d.id == id)
    }

    fn find_record(
        &self,
        tenant_id: &str,
        collection: &str,
        table: &str,
        id: &str,
    ) -> Option<&Record> {
        self.state.records.iter().find(|r| {
            r.tenant_id == tenant_id && r.collection == collection && r.table == table && r.id == id
        })
    }
}

fn project_record(table: &str, id: &str, payload: &serde_json::Value) -> Result<String> {
    if !payload.is_object() {
        return Err(HippocoreError::validation(
            "record payload must be a JSON object",
        ));
    }

    let mut parts = vec![
        "table".to_string(),
        table.to_string(),
        "record".to_string(),
        id.to_string(),
    ];
    append_json_projection(&mut parts, payload);
    Ok(parts.join(" "))
}

fn append_json_projection(parts: &mut Vec<String>, value: &serde_json::Value) {
    match value {
        serde_json::Value::Null => parts.push("null".to_string()),
        serde_json::Value::Bool(v) => parts.push(v.to_string()),
        serde_json::Value::Number(v) => parts.push(v.to_string()),
        serde_json::Value::String(v) => parts.push(v.clone()),
        serde_json::Value::Array(values) => {
            for value in values {
                append_json_projection(parts, value);
            }
        }
        serde_json::Value::Object(map) => {
            for (key, value) in map {
                parts.push(key.clone());
                append_json_projection(parts, value);
            }
        }
    }
}

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

static ID_COUNTER: AtomicU64 = AtomicU64::new(0);

fn gen_id(prefix: &str) -> String {
    let n = ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}-{}-{}", now_millis(), n)
}
