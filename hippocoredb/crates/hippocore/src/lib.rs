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
pub mod hnsw;
pub mod index;
pub mod memory;
pub mod model;
pub mod query;
pub mod storage;

pub mod cli;

use std::collections::HashSet;
use std::fmt;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

pub use config::Config;
pub use errors::{HippocoreError, Result};
pub use index::VectorIndexKind;
pub use model::{
    AuditItem, AuditRecord, Chunk, Collection, ContextItemSource, Document, Embedding, FileObject,
    GraphEdge, ItemKind, Memory, MemoryType, Metadata, RecallResult, Record, Source, Tenant,
};
pub use query::SearchMode;

use index::{Index, IndexEntry};
use query::{Filter, QueryRequest};
use storage::{Operation, State, Storage};

const AUDIT_FILE: &str = "audit.log";

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
    /// Stored graph relationship edges.
    pub graph_edges: usize,
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
        writeln!(f, "  graph edges:     {}", self.graph_edges)?;
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
    /// Optional validity start (epoch ms). `None` = valid from creation time.
    pub valid_from: Option<i64>,
    /// Optional validity end (epoch ms). `None` = never expires.
    pub valid_until: Option<i64>,
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
            valid_from: None,
            valid_until: None,
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
    /// Optional validity start (epoch ms). `None` = valid from creation time.
    pub valid_from: Option<i64>,
    /// Optional validity end (epoch ms). `None` = never expires.
    pub valid_until: Option<i64>,
    /// Ids of memories this one replaces. Each must exist in the same tenant.
    pub supersedes: Vec<String>,
    /// Ids of memories this one contradicts (advisory).
    pub contradicts: Vec<String>,
    /// Optional confidence score in `[0.0, 1.0]`. `None` = unrated.
    pub confidence: Option<f32>,
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
            valid_from: None,
            valid_until: None,
            supersedes: Vec::new(),
            contradicts: Vec::new(),
            confidence: None,
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

/// Request to import a text-like file as database context.
#[derive(Debug, Clone)]
pub struct ImportFileRequest {
    /// Owning tenant (must exist).
    pub tenant_id: String,
    /// Owning collection (must exist).
    pub collection: String,
    /// Optional explicit file id; generated if `None`.
    pub id: Option<String>,
    /// Path to a local file.
    pub path: PathBuf,
    /// Exact-match metadata.
    pub metadata: Metadata,
    /// Optional provenance.
    pub source: Option<Source>,
}

impl ImportFileRequest {
    /// Build a minimal import request.
    pub fn new(
        tenant_id: impl Into<String>,
        collection: impl Into<String>,
        path: impl Into<PathBuf>,
    ) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            collection: collection.into(),
            id: None,
            path: path.into(),
            metadata: Metadata::new(),
            source: None,
        }
    }
}

/// Request to add a durable relationship between two context items.
#[derive(Debug, Clone)]
pub struct AddGraphEdgeRequest {
    /// Owning tenant for both endpoints.
    pub tenant_id: String,
    /// Optional explicit id; generated if `None`.
    pub id: Option<String>,
    /// Source item id.
    pub from_id: String,
    /// Source item kind.
    pub from_kind: ItemKind,
    /// Target item id.
    pub to_id: String,
    /// Target item kind.
    pub to_kind: ItemKind,
    /// Relationship label.
    pub relation: String,
    /// Exact-match metadata.
    pub metadata: Metadata,
}

impl AddGraphEdgeRequest {
    /// Build a minimal graph edge request.
    pub fn new(
        tenant_id: impl Into<String>,
        from_kind: ItemKind,
        from_id: impl Into<String>,
        to_kind: ItemKind,
        to_id: impl Into<String>,
        relation: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            id: None,
            from_id: from_id.into(),
            from_kind,
            to_id: to_id.into(),
            to_kind,
            relation: relation.into(),
            metadata: Metadata::new(),
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
    /// Query as of this epoch ms. `None` = current time (default: excludes
    /// expired entries). Pass an explicit timestamp to query historical state.
    pub as_of: Option<i64>,
    /// When `false` (default), superseded memories are excluded from results.
    /// Set to `true` to surface the full history including superseded entries.
    pub include_superseded: bool,
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
            as_of: None,
            include_superseded: false,
        }
    }
}

/// Request to compile a token-budget-aware context block for an LLM prompt.
#[derive(Debug, Clone)]
pub struct BuildContextRequest {
    /// Tenant to recall from (mandatory).
    pub tenant_id: String,
    /// Natural-language query driving the recall.
    pub query: String,
    /// Restrict recall to an owning user.
    pub user_id: Option<String>,
    /// Hard token ceiling for the assembled context string.
    pub max_tokens: usize,
    /// How many candidates to recall before budget trimming (default 20).
    pub top_k_candidates: usize,
    /// Recall mode (default `Hybrid`).
    pub mode: SearchMode,
    /// Restrict recall to a collection.
    pub collection: Option<String>,
    /// Require these exact metadata pairs on recalled items.
    pub metadata_filter: Metadata,
    /// Include direct graph neighbours of recalled items when they fit the
    /// token budget. Default is `false`.
    pub include_related: bool,
    /// Maximum number of graph-expanded neighbour items to consider.
    pub related_limit: usize,
}

impl BuildContextRequest {
    /// Build a minimal request with sensible defaults.
    pub fn new(tenant_id: impl Into<String>, query: impl Into<String>, max_tokens: usize) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            query: query.into(),
            user_id: None,
            max_tokens,
            top_k_candidates: 20,
            mode: SearchMode::Hybrid,
            collection: None,
            metadata_filter: Metadata::new(),
            include_related: false,
            related_limit: 8,
        }
    }
}

/// A single item that was included in a [`ContextBlock`].
#[derive(Debug, Clone, PartialEq)]
pub struct ContextItem {
    /// Id of the underlying memory, chunk, or record.
    pub id: String,
    /// Whether this is a document chunk, memory, or record.
    pub kind: ItemKind,
    /// Recall score used for ranking.
    pub score: f32,
    /// Confidence of the underlying memory, or `None` for chunks/records.
    pub confidence: Option<f32>,
    /// Approximate token count for this item's text (1 token ≈ 4 bytes).
    pub token_count: usize,
    /// First 120 characters of the item's text.
    pub snippet: String,
    /// Directly related neighbour ids known at context-build time.
    pub related_item_ids: Vec<String>,
    /// Whether this item came from recall or graph expansion.
    pub inclusion_source: ContextItemSource,
}

/// The assembled context string and provenance returned by
/// [`Hippocore::build_context`].
#[derive(Debug, Clone, PartialEq)]
pub struct ContextBlock {
    /// LLM-ready context string. Format: `[<kind>:<id>]\n<text>` per item,
    /// separated by `\n\n`.
    pub text: String,
    /// Approximate token count of `text` (1 token ≈ 4 bytes).
    pub token_count: usize,
    /// Items that were included (highest score first).
    pub items_included: Vec<ContextItem>,
    /// Number of recalled items that did not fit within `max_tokens`.
    pub items_dropped: usize,
}

#[derive(Debug, Clone)]
struct ContextCandidate {
    id: String,
    kind: ItemKind,
    collection: String,
    text: String,
    score: f32,
    vector_score: f32,
    text_score: f32,
    confidence: Option<f32>,
    inclusion_source: ContextItemSource,
    related_item_ids: Vec<String>,
}

impl Hippocore {
    /// Open (creating if needed) a database at `config.data_dir`, recovering any
    /// previously persisted state.
    pub fn open(config: Config) -> Result<Self> {
        let (storage, state) = Storage::open(&config.data_dir, config.sync_writes)?;
        let embedder = memory::Embedder::new(config.embedding_dim);
        let index = Index::with_vector_backend(config.vector_index);
        let mut db = Self {
            config,
            embedder,
            storage,
            state,
            index,
        };
        db.rebuild_index();
        Ok(db)
    }

    fn rebuild_index(&mut self) {
        self.index = Index::with_vector_backend(self.config.vector_index);
        // Index document chunks, carrying their parent document's metadata/source/validity.
        for chunk in &self.state.chunks {
            let parent = self
                .state
                .documents
                .iter()
                .find(|d| d.id == chunk.document_id);
            let (metadata, source, valid_from, valid_until) = parent
                .map(|d| {
                    (
                        d.metadata.clone(),
                        d.source.clone(),
                        d.valid_from,
                        d.valid_until,
                    )
                })
                .unwrap_or_default();
            self.index.insert(IndexEntry::from_chunk(
                chunk,
                metadata,
                source,
                valid_from,
                valid_until,
            ));
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
            valid_from: req.valid_from,
            valid_until: req.valid_until,
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

        // Validate that supersedes/contradicts ids exist in the same tenant.
        for sid in &req.supersedes {
            if !self
                .state
                .memories
                .iter()
                .any(|m| m.tenant_id == req.tenant_id && m.id == *sid)
            {
                return Err(HippocoreError::validation(format!(
                    "supersedes id {sid:?} not found in tenant {:?}",
                    req.tenant_id
                )));
            }
        }
        for cid in &req.contradicts {
            if !self
                .state
                .memories
                .iter()
                .any(|m| m.tenant_id == req.tenant_id && m.id == *cid)
            {
                return Err(HippocoreError::validation(format!(
                    "contradicts id {cid:?} not found in tenant {:?}",
                    req.tenant_id
                )));
            }
        }

        let new_id = id.clone();
        let mem = Memory {
            id,
            tenant_id: req.tenant_id.clone(),
            collection: req.collection,
            user_id: req.user_id,
            memory_type: req.memory_type,
            text: req.text,
            embedding,
            metadata: req.metadata,
            source: req.source,
            created_at,
            valid_from: req.valid_from,
            valid_until: req.valid_until,
            supersedes: req.supersedes.clone(),
            contradicts: req.contradicts,
            superseded_by: None,
            confidence: req.confidence,
        };
        mem.validate()?;
        self.commit(Operation::PutMemory(mem.clone()))?;

        // Mark superseded memories.
        for sid in &req.supersedes {
            if let Some(old) = self
                .state
                .memories
                .iter()
                .find(|m| m.tenant_id == req.tenant_id && m.id == *sid)
                .cloned()
            {
                let mut updated = old;
                updated.superseded_by = Some(new_id.clone());
                self.commit(Operation::PutMemory(updated))?;
            }
        }

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

    /// Import a text-like file and index its extracted text as a derived document.
    pub fn import_file(&mut self, req: ImportFileRequest) -> Result<FileObject> {
        self.require_collection(&req.tenant_id, &req.collection)?;

        let id = req.id.unwrap_or_else(|| gen_id("file"));
        let path = req.path;
        let path_string = path.display().to_string();
        let name = path
            .file_name()
            .and_then(|v| v.to_str())
            .ok_or_else(|| HippocoreError::validation("file path must include a valid name"))?
            .to_string();
        let bytes = std::fs::read(&path)?;
        let size_bytes = bytes.len() as u64;
        let checksum = format!("{:08x}", storage::crc32(&bytes));
        let (media_type, extracted_text) = extract_file_text(&path, &bytes)?;
        let now = now_millis();
        let (created_at, version, document_id) =
            match self.find_file(&req.tenant_id, &req.collection, &id) {
                Some(existing) => (
                    existing.created_at,
                    existing.version + 1,
                    existing.document_id.clone(),
                ),
                None => (now, 0, format!("file:{id}")),
            };

        let source = req.source.or_else(|| {
            Some(Source {
                label: "file".to_string(),
                uri: Some(path_string.clone()),
                title: Some(name.clone()),
            })
        });
        let mut metadata = req.metadata;
        metadata.insert("file_id".to_string(), id.clone());
        metadata.insert("file_name".to_string(), name.clone());
        metadata.insert("media_type".to_string(), media_type.clone());
        metadata.insert("checksum".to_string(), checksum.clone());

        let file = FileObject {
            id: id.clone(),
            tenant_id: req.tenant_id.clone(),
            collection: req.collection.clone(),
            path: path_string,
            name,
            media_type,
            checksum,
            size_bytes,
            document_id: document_id.clone(),
            metadata: metadata.clone(),
            source: source.clone(),
            created_at,
            updated_at: now,
            version,
        };
        file.validate()?;

        let document = Document {
            id: document_id,
            tenant_id: req.tenant_id,
            collection: req.collection,
            text: extracted_text,
            metadata,
            source,
            created_at,
            updated_at: now,
            version,
            valid_from: None,
            valid_until: None,
        };
        document.validate()?;
        let chunks = self.build_chunks(&document);
        self.commit(Operation::PutFile {
            file: Box::new(file.clone()),
            document,
            chunks,
        })?;
        Ok(file)
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

    /// Delete an imported file and its derived document/chunks by identity.
    ///
    /// Durable and idempotent, like [`Hippocore::forget`].
    pub fn delete_file(&mut self, tenant_id: &str, collection: &str, id: &str) -> Result<()> {
        let document_id = self
            .find_file(tenant_id, collection, id)
            .map(|file| file.document_id.clone());
        if let Some(document_id) = document_id {
            self.index
                .remove_document(tenant_id, collection, &document_id);
        }
        self.commit(Operation::DeleteFile {
            tenant_id: tenant_id.to_string(),
            collection: collection.to_string(),
            id: id.to_string(),
        })
    }

    /// Update the confidence score of an existing memory (human-in-the-loop rating).
    ///
    /// `confidence` must be in `[0.0, 1.0]`. Returns an error if the memory is
    /// not found or the value is out of range. The update is written durably to
    /// the WAL before returning.
    pub fn rate_memory(
        &mut self,
        tenant_id: &str,
        collection: &str,
        id: &str,
        confidence: f32,
    ) -> Result<Memory> {
        if !(0.0..=1.0).contains(&confidence) {
            return Err(HippocoreError::validation(format!(
                "confidence must be in [0.0, 1.0], got {confidence}"
            )));
        }
        let mut mem = self
            .state
            .memories
            .iter()
            .find(|m| m.tenant_id == tenant_id && m.collection == collection && m.id == id)
            .cloned()
            .ok_or_else(|| HippocoreError::NotFound(format!("memory {id:?} not found")))?;
        mem.confidence = Some(confidence);
        self.commit(Operation::PutMemory(mem.clone()))?;
        Ok(mem)
    }

    /// Add or overwrite a durable direct relationship between two context items.
    pub fn add_graph_edge(&mut self, req: AddGraphEdgeRequest) -> Result<GraphEdge> {
        self.require_tenant(&req.tenant_id)?;
        self.require_unique_endpoint(&req.tenant_id, req.from_kind, &req.from_id)?;
        self.require_unique_endpoint(&req.tenant_id, req.to_kind, &req.to_id)?;

        let id = req.id.unwrap_or_else(|| gen_id("edge"));
        let now = now_millis();
        let created_at = self
            .state
            .graph_edges
            .iter()
            .find(|edge| edge.tenant_id == req.tenant_id && edge.id == id)
            .map(|edge| edge.created_at)
            .unwrap_or(now);
        let edge = GraphEdge {
            id,
            tenant_id: req.tenant_id,
            from_id: req.from_id,
            from_kind: req.from_kind,
            to_id: req.to_id,
            to_kind: req.to_kind,
            relation: req.relation,
            metadata: req.metadata,
            created_at,
            updated_at: now,
        };
        edge.validate()?;
        self.commit(Operation::PutGraphEdge(edge.clone()))?;
        Ok(edge)
    }

    /// Delete a graph edge by id. Missing ids are a safe no-op.
    pub fn delete_graph_edge(&mut self, tenant_id: &str, id: &str) -> Result<()> {
        self.require_tenant(tenant_id)?;
        self.commit(Operation::DeleteGraphEdge {
            tenant_id: tenant_id.to_string(),
            id: id.to_string(),
        })
    }

    /// Store multiple memories in a single WAL write + single fsync (group-commit).
    ///
    /// Semantically identical to calling [`remember`] for each request in order,
    /// but all operations are appended to the WAL in one batched write, reducing
    /// the number of `fsync` calls from N to 1. Useful for bulk ingestion.
    ///
    /// Returns an error on the first validation failure; no memories are written
    /// if validation of any request fails.
    ///
    /// [`remember`]: Hippocore::remember
    pub fn remember_many(&mut self, reqs: Vec<RememberRequest>) -> Result<Vec<Memory>> {
        let mut memories = Vec::with_capacity(reqs.len());
        let mut ops: Vec<storage::Operation> = Vec::new();

        for req in reqs {
            self.require_collection(&req.tenant_id, &req.collection)?;

            let id = req.id.unwrap_or_else(|| gen_id("mem"));
            let created_at = self
                .state
                .memories
                .iter()
                .find(|m| {
                    m.tenant_id == req.tenant_id && m.collection == req.collection && m.id == id
                })
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

            for sid in &req.supersedes {
                if !self
                    .state
                    .memories
                    .iter()
                    .any(|m| m.tenant_id == req.tenant_id && m.id == *sid)
                {
                    return Err(HippocoreError::validation(format!(
                        "supersedes id {sid:?} not found in tenant {:?}",
                        req.tenant_id
                    )));
                }
            }

            let new_id = id.clone();
            let mem = Memory {
                id,
                tenant_id: req.tenant_id.clone(),
                collection: req.collection,
                user_id: req.user_id,
                memory_type: req.memory_type,
                text: req.text,
                embedding,
                metadata: req.metadata,
                source: req.source,
                created_at,
                valid_from: req.valid_from,
                valid_until: req.valid_until,
                supersedes: req.supersedes.clone(),
                contradicts: req.contradicts,
                superseded_by: None,
                confidence: req.confidence,
            };
            mem.validate()?;
            ops.push(storage::Operation::PutMemory(mem.clone()));

            for sid in &req.supersedes {
                if let Some(old) = self
                    .state
                    .memories
                    .iter()
                    .find(|m| m.tenant_id == req.tenant_id && m.id == *sid)
                    .cloned()
                {
                    let mut updated = old;
                    updated.superseded_by = Some(new_id.clone());
                    ops.push(storage::Operation::PutMemory(updated));
                }
            }

            memories.push(mem);
        }

        // Write all ops in one batch write + one fsync.
        self.storage.append_many(&ops)?;
        for op in ops {
            self.state.apply(op.clone());
            self.index_op(&op);
        }
        if self.should_auto_compact() {
            self.storage.compact(&self.state)?;
        }
        Ok(memories)
    }

    /// Store multiple documents in a single WAL write + single fsync (group-commit).
    ///
    /// Semantically identical to calling [`store_document`] for each request,
    /// but amortizes fsync cost over the batch. Returns an error on the first
    /// validation failure; no documents are written if any request is invalid.
    ///
    /// [`store_document`]: Hippocore::store_document
    pub fn store_documents(&mut self, reqs: Vec<StoreDocumentRequest>) -> Result<Vec<Document>> {
        let mut documents = Vec::with_capacity(reqs.len());
        let mut ops: Vec<storage::Operation> = Vec::new();

        for req in reqs {
            self.require_collection(&req.tenant_id, &req.collection)?;

            let id = req.id.unwrap_or_else(|| gen_id("doc"));
            let now = now_millis();
            let (created_at, version) =
                match self.find_document(&req.tenant_id, &req.collection, &id) {
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
                valid_from: req.valid_from,
                valid_until: req.valid_until,
            };
            document.validate()?;

            let chunks = match req.chunks {
                Some(inputs) => self.build_chunks_from_inputs(&document, inputs)?,
                None => self.build_chunks(&document),
            };

            // Remove old chunks from the index eagerly (before applying state).
            self.index
                .remove_document(&document.tenant_id, &document.collection, &document.id);
            ops.push(storage::Operation::PutDocument {
                document: document.clone(),
                chunks,
            });
            documents.push(document);
        }

        self.storage.append_many(&ops)?;
        for op in ops {
            self.state.apply(op.clone());
            self.index_op(&op);
        }
        if self.should_auto_compact() {
            self.storage.compact(&self.state)?;
        }
        Ok(documents)
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

    /// Recall the best items for `req.query` and assemble them into a
    /// token-budget-aware context block ready for an LLM prompt.
    ///
    /// Items are ranked by recall score (highest first). Each item is added
    /// greedily until the next item would exceed `max_tokens`. The token budget
    /// uses the approximation 1 token ≈ 4 UTF-8 bytes.
    pub fn build_context(&self, req: BuildContextRequest) -> Result<ContextBlock> {
        let started = Instant::now();
        let tenant_id = req.tenant_id.clone();
        let query = req.query.clone();
        let mode = req.mode;
        let collection = req.collection.clone();
        let max_tokens = req.max_tokens;

        let mut recall_req = RecallRequest::new(req.tenant_id, req.query);
        recall_req.user_id = req.user_id;
        recall_req.top_k = req.top_k_candidates;
        recall_req.mode = mode;
        recall_req.collection = req.collection;
        recall_req.metadata = req.metadata_filter;

        let mut hits = self.run_query(recall_req)?;

        // Confidence-aware re-ranking: when contradictions are present among the
        // candidates, promote higher-confidence memories by blending their
        // confidence into the effective score.  Items without a confidence rating
        // are treated as confidence = 0.5 (neutral) so they are not penalised.
        //
        // effective_score = recall_score * 0.7 + confidence * 0.3
        //
        // This is only applied to Memory items (chunks and records do not carry
        // confidence); other items keep their original recall score.
        let has_contradictions = hits.iter().any(|h| !h.contradictions.is_empty());
        if has_contradictions {
            hits.sort_by(|a, b| {
                let eff_a = if a.kind == ItemKind::Memory {
                    a.score * 0.7 + a.confidence.unwrap_or(0.5) * 0.3
                } else {
                    a.score
                };
                let eff_b = if b.kind == ItemKind::Memory {
                    b.score * 0.7 + b.confidence.unwrap_or(0.5) * 0.3
                } else {
                    b.score
                };
                eff_b
                    .partial_cmp(&eff_a)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        let mut candidates: Vec<ContextCandidate> = hits
            .iter()
            .map(|hit| ContextCandidate {
                id: hit.id.clone(),
                kind: hit.kind,
                collection: hit.collection.clone(),
                text: hit.text.clone(),
                score: hit.score,
                vector_score: hit.vector_score,
                text_score: hit.text_score,
                confidence: hit.confidence,
                inclusion_source: ContextItemSource::Recalled,
                related_item_ids: self.related_item_ids(&hit.tenant_id, hit.kind, &hit.id),
            })
            .collect();
        if req.include_related && req.related_limit > 0 {
            candidates.extend(self.graph_expanded_candidates(&hits, req.related_limit));
        }

        let mut included: Vec<ContextItem> = Vec::new();
        let mut included_text_blocks: Vec<String> = Vec::new();
        let mut included_keys: HashSet<(ItemKind, String)> = HashSet::new();
        let mut dropped: usize = 0;
        let mut budget_used: usize = 0;

        for candidate in &candidates {
            let block = context_text_block(candidate.kind, &candidate.id, &candidate.text);
            let item_tokens = approx_tokens(&block);

            if budget_used + item_tokens > req.max_tokens {
                dropped += 1;
                continue;
            }

            let snippet: String = candidate.text.chars().take(120).collect();
            included_keys.insert((candidate.kind, candidate.id.clone()));
            included_text_blocks.push(block);
            included.push(ContextItem {
                id: candidate.id.clone(),
                kind: candidate.kind,
                score: candidate.score,
                confidence: candidate.confidence,
                token_count: item_tokens,
                snippet,
                related_item_ids: candidate.related_item_ids.clone(),
                inclusion_source: candidate.inclusion_source,
            });
            budget_used += item_tokens;
        }

        let text = included_text_blocks.join("\n\n");

        let block = ContextBlock {
            token_count: approx_tokens(&text),
            text,
            items_included: included,
            items_dropped: dropped,
        };

        let audit = AuditRecord {
            timestamp_ms: now_millis(),
            tenant_id,
            query,
            mode: mode.as_str().to_string(),
            collection,
            max_tokens,
            token_count: block.token_count,
            latency_ms: started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64,
            items_dropped: block.items_dropped,
            items: candidates
                .iter()
                .map(|candidate| {
                    let item_tokens = approx_tokens(&context_text_block(
                        candidate.kind,
                        &candidate.id,
                        &candidate.text,
                    ));
                    let included = included_keys.contains(&(candidate.kind, candidate.id.clone()));
                    AuditItem {
                        id: candidate.id.clone(),
                        kind: candidate.kind,
                        collection: candidate.collection.clone(),
                        score: candidate.score,
                        vector_score: candidate.vector_score,
                        text_score: candidate.text_score,
                        confidence: candidate.confidence,
                        token_count: item_tokens,
                        included,
                        inclusion_source: if included {
                            candidate.inclusion_source
                        } else {
                            ContextItemSource::NotIncluded
                        },
                        related_item_ids: candidate.related_item_ids.clone(),
                    }
                })
                .collect(),
        };
        self.append_audit_record(&audit)?;

        Ok(block)
    }

    /// Replay RAG audit records for one tenant within an inclusive epoch-ms range.
    pub fn query_audit(
        &self,
        from_ms: i64,
        to_ms: i64,
        tenant_id: impl AsRef<str>,
    ) -> Result<Vec<AuditRecord>> {
        if from_ms > to_ms {
            return Err(HippocoreError::validation(
                "query_audit requires from_ms <= to_ms",
            ));
        }
        let tenant_id = tenant_id.as_ref();
        if tenant_id.trim().is_empty() {
            return Err(HippocoreError::validation(
                "query_audit requires a tenant_id",
            ));
        }

        let path = self.storage.data_dir().join(AUDIT_FILE);
        if !path.exists() {
            return Ok(Vec::new());
        }
        let file = std::fs::File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        for line in reader.lines() {
            let line = match line {
                Ok(line) => line,
                Err(_) => break,
            };
            if line.trim().is_empty() {
                continue;
            }
            let record = match serde_json::from_str::<AuditRecord>(&line) {
                Ok(record) => record,
                Err(_) => break,
            };
            if record.tenant_id == tenant_id
                && record.timestamp_ms >= from_ms
                && record.timestamp_ms <= to_ms
            {
                records.push(record);
            }
        }
        Ok(records)
    }

    fn append_audit_record(&self, record: &AuditRecord) -> Result<()> {
        let path = self.storage.data_dir().join(AUDIT_FILE);
        let json = serde_json::to_vec(record)?;
        let mut line = json;
        line.push(b'\n');

        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        file.write_all(&line)?;
        file.flush()?;
        if self.config.sync_writes {
            file.sync_all()?;
        }
        Ok(())
    }

    fn related_item_ids(&self, tenant_id: &str, kind: ItemKind, id: &str) -> Vec<String> {
        let mut ids: Vec<String> = self
            .graph_neighbors(tenant_id, kind, id)
            .into_iter()
            .filter_map(|edge| edge.neighbor_id(kind, id).map(str::to_string))
            .collect();
        ids.sort();
        ids.dedup();
        ids
    }

    fn graph_expanded_candidates(
        &self,
        hits: &[RecallResult],
        related_limit: usize,
    ) -> Vec<ContextCandidate> {
        let mut seen: HashSet<(ItemKind, String)> =
            hits.iter().map(|hit| (hit.kind, hit.id.clone())).collect();
        let mut expanded = Vec::new();

        for hit in hits {
            for edge in self.graph_neighbors(&hit.tenant_id, hit.kind, &hit.id) {
                let (neighbor_kind, neighbor_id) =
                    if edge.from_kind == hit.kind && edge.from_id == hit.id {
                        (edge.to_kind, edge.to_id.as_str())
                    } else {
                        (edge.from_kind, edge.from_id.as_str())
                    };
                let key = (neighbor_kind, neighbor_id.to_string());
                if seen.contains(&key) {
                    continue;
                }
                if let Some(candidate) =
                    self.context_candidate_for_endpoint(&hit.tenant_id, neighbor_kind, neighbor_id)
                {
                    seen.insert(key);
                    expanded.push(candidate);
                    if expanded.len() >= related_limit {
                        return expanded;
                    }
                }
            }
        }

        expanded
    }

    fn context_candidate_for_endpoint(
        &self,
        tenant_id: &str,
        kind: ItemKind,
        id: &str,
    ) -> Option<ContextCandidate> {
        match kind {
            ItemKind::Memory => self
                .state
                .memories
                .iter()
                .find(|memory| memory.tenant_id == tenant_id && memory.id == id)
                .map(|memory| ContextCandidate {
                    id: memory.id.clone(),
                    kind,
                    collection: memory.collection.clone(),
                    text: memory.text.clone(),
                    score: 0.0,
                    vector_score: 0.0,
                    text_score: 0.0,
                    confidence: memory.confidence,
                    inclusion_source: ContextItemSource::GraphExpanded,
                    related_item_ids: self.related_item_ids(tenant_id, kind, id),
                }),
            ItemKind::DocumentChunk => self
                .state
                .chunks
                .iter()
                .find(|chunk| chunk.tenant_id == tenant_id && chunk.id == id)
                .map(|chunk| ContextCandidate {
                    id: chunk.id.clone(),
                    kind,
                    collection: chunk.collection.clone(),
                    text: chunk.text.clone(),
                    score: 0.0,
                    vector_score: 0.0,
                    text_score: 0.0,
                    confidence: None,
                    inclusion_source: ContextItemSource::GraphExpanded,
                    related_item_ids: self.related_item_ids(tenant_id, kind, id),
                }),
            ItemKind::Record => self
                .state
                .records
                .iter()
                .find(|record| record.tenant_id == tenant_id && record.id == id)
                .map(|record| ContextCandidate {
                    id: record.id.clone(),
                    kind,
                    collection: record.collection.clone(),
                    text: record.projection.clone(),
                    score: 0.0,
                    vector_score: 0.0,
                    text_score: 0.0,
                    confidence: None,
                    inclusion_source: ContextItemSource::GraphExpanded,
                    related_item_ids: self.related_item_ids(tenant_id, kind, id),
                }),
        }
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
            // Default as_of = now: expired entries are excluded by default.
            as_of: Some(req.as_of.unwrap_or_else(now_millis)),
            include_superseded: req.include_superseded,
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
            files: self.state.files.len(),
            graph_edges: self.state.graph_edges.len(),
            indexed_entries: self.index.len(),
            wal_entries: self.storage.wal_len,
            disk_bytes: self.storage.disk_bytes()?,
        })
    }

    /// List tenants (clones).
    pub fn tenants(&self) -> Vec<Tenant> {
        self.state.tenants.clone()
    }

    /// List collections for a tenant (or all tenants when `tenant_id` is `None`).
    pub fn collections(&self, tenant_id: &str) -> Vec<Collection> {
        self.state
            .collections
            .iter()
            .filter(|c| c.tenant_id == tenant_id)
            .cloned()
            .collect()
    }

    /// List all collections across all tenants (admin view).
    pub fn all_collections(&self) -> Vec<Collection> {
        self.state.collections.clone()
    }

    /// List documents within a tenant, optionally scoped to a collection.
    pub fn list_documents(&self, tenant_id: &str, collection: Option<&str>) -> Vec<Document> {
        self.state
            .documents
            .iter()
            .filter(|d| d.tenant_id == tenant_id && collection.map_or(true, |c| d.collection == c))
            .cloned()
            .collect()
    }

    /// List memories within a tenant, optionally scoped to a collection.
    pub fn list_memories(&self, tenant_id: &str, collection: Option<&str>) -> Vec<Memory> {
        self.state
            .memories
            .iter()
            .filter(|m| m.tenant_id == tenant_id && collection.map_or(true, |c| m.collection == c))
            .cloned()
            .collect()
    }

    /// List records within a tenant, optionally scoped to a collection and table.
    pub fn list_records(
        &self,
        tenant_id: &str,
        collection: Option<&str>,
        table: Option<&str>,
    ) -> Vec<Record> {
        self.state
            .records
            .iter()
            .filter(|r| {
                r.tenant_id == tenant_id
                    && collection.map_or(true, |c| r.collection == c)
                    && table.map_or(true, |t| r.table == t)
            })
            .cloned()
            .collect()
    }

    /// List files within a tenant, optionally scoped to a collection.
    pub fn list_files(&self, tenant_id: &str, collection: Option<&str>) -> Vec<FileObject> {
        self.state
            .files
            .iter()
            .filter(|f| f.tenant_id == tenant_id && collection.map_or(true, |c| f.collection == c))
            .cloned()
            .collect()
    }

    /// List graph edges for a tenant, optionally scoped to a source endpoint id.
    pub fn list_graph_edges(&self, tenant_id: &str, from_id: Option<&str>) -> Vec<GraphEdge> {
        self.state
            .graph_edges
            .iter()
            .filter(|edge| {
                edge.tenant_id == tenant_id && from_id.map_or(true, |id| edge.from_id == id)
            })
            .cloned()
            .collect()
    }

    /// Return direct graph edges touching one endpoint.
    pub fn graph_neighbors(&self, tenant_id: &str, kind: ItemKind, id: &str) -> Vec<GraphEdge> {
        self.state
            .graph_edges
            .iter()
            .filter(|edge| edge.tenant_id == tenant_id && edge.touches(kind, id))
            .cloned()
            .collect()
    }

    /// Get a single document by identity (`None` when not found).
    pub fn get_document(&self, tenant_id: &str, collection: &str, id: &str) -> Option<Document> {
        self.find_document(tenant_id, collection, id).cloned()
    }

    /// Get a single memory by identity (`None` when not found).
    pub fn get_memory(&self, tenant_id: &str, collection: &str, id: &str) -> Option<Memory> {
        self.state
            .memories
            .iter()
            .find(|m| m.tenant_id == tenant_id && m.collection == collection && m.id == id)
            .cloned()
    }

    /// Get a single record by table and identity (`None` when not found).
    pub fn get_record(
        &self,
        tenant_id: &str,
        collection: &str,
        table: &str,
        id: &str,
    ) -> Option<Record> {
        self.find_record(tenant_id, collection, table, id).cloned()
    }

    /// Get a single file by identity (`None` when not found).
    pub fn get_file(&self, tenant_id: &str, collection: &str, id: &str) -> Option<FileObject> {
        self.find_file(tenant_id, collection, id).cloned()
    }

    /// Get the chunks that belong to a document, in ordinal order.
    pub fn get_document_chunks(
        &self,
        tenant_id: &str,
        collection: &str,
        document_id: &str,
    ) -> Vec<Chunk> {
        let mut chunks: Vec<Chunk> = self
            .state
            .chunks
            .iter()
            .filter(|c| {
                c.tenant_id == tenant_id
                    && c.collection == collection
                    && c.document_id == document_id
            })
            .cloned()
            .collect();
        chunks.sort_by_key(|c| c.ordinal);
        chunks
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
        match &op {
            Operation::PutDocument { document, .. } | Operation::PutFile { document, .. } => {
                self.index
                    .remove_document(&document.tenant_id, &document.collection, &document.id);
            }
            _ => {}
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
                        document.valid_from,
                        document.valid_until,
                    ));
                }
            }
            Operation::PutMemory(m) => {
                self.index.insert(IndexEntry::from_memory(m));
            }
            Operation::PutRecord(r) => {
                self.index.insert(IndexEntry::from_record(r));
            }
            Operation::PutFile {
                document, chunks, ..
            } => {
                for chunk in chunks {
                    self.index.insert(IndexEntry::from_chunk(
                        chunk,
                        document.metadata.clone(),
                        document.source.clone(),
                        document.valid_from,
                        document.valid_until,
                    ));
                }
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
            Operation::DeleteFile { .. } => {}
            Operation::PutGraphEdge(_) | Operation::DeleteGraphEdge { .. } => {}
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

    fn require_unique_endpoint(&self, tenant_id: &str, kind: ItemKind, id: &str) -> Result<()> {
        let matches = match kind {
            ItemKind::Memory => self
                .state
                .memories
                .iter()
                .filter(|m| m.tenant_id == tenant_id && m.id == id)
                .count(),
            ItemKind::DocumentChunk => self
                .state
                .chunks
                .iter()
                .filter(|ch| ch.tenant_id == tenant_id && ch.id == id)
                .count(),
            ItemKind::Record => self
                .state
                .records
                .iter()
                .filter(|r| r.tenant_id == tenant_id && r.id == id)
                .count(),
        };
        match matches {
            1 => Ok(()),
            0 => Err(HippocoreError::validation(format!(
                "{} endpoint {id:?} not found in tenant {tenant_id:?}",
                kind.as_str()
            ))),
            _ => Err(HippocoreError::validation(format!(
                "{} endpoint {id:?} is ambiguous in tenant {tenant_id:?}",
                kind.as_str()
            ))),
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

    fn find_file(&self, tenant_id: &str, collection: &str, id: &str) -> Option<&FileObject> {
        self.state
            .files
            .iter()
            .find(|f| f.tenant_id == tenant_id && f.collection == collection && f.id == id)
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

fn extract_file_text(path: &std::path::Path, bytes: &[u8]) -> Result<(String, String)> {
    let extension = path
        .extension()
        .and_then(|v| v.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| HippocoreError::validation("file extension is required"))?;
    let raw_text = std::str::from_utf8(bytes)
        .map_err(|e| HippocoreError::validation(format!("file must be valid UTF-8: {e}")))?;

    match extension.as_str() {
        "txt" => Ok(("text/plain".to_string(), raw_text.to_string())),
        "md" => Ok(("text/markdown".to_string(), raw_text.to_string())),
        "csv" => Ok(("text/csv".to_string(), raw_text.to_string())),
        "json" => {
            let value: serde_json::Value = serde_json::from_str(raw_text)?;
            let mut parts = Vec::new();
            append_json_projection(&mut parts, &value);
            Ok(("application/json".to_string(), parts.join(" ")))
        }
        other => Err(HippocoreError::validation(format!(
            "unsupported file extension: {other:?} (expected txt|md|json|csv)"
        ))),
    }
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

fn context_text_block(kind: ItemKind, id: &str, text: &str) -> String {
    let kind_label = match kind {
        ItemKind::Memory => "memory",
        ItemKind::DocumentChunk => "chunk",
        ItemKind::Record => "record",
    };
    format!("[{kind_label}:{id}]\n{text}")
}

/// Approximate token count: 1 token ≈ 4 UTF-8 bytes (fast, no tokenizer dep).
fn approx_tokens(text: &str) -> usize {
    text.len().div_ceil(4)
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
