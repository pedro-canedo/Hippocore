//! Core data model: the strongly-typed entities Hippocore stores and returns.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::errors::{HippocoreError, Result};

/// String key/value metadata attached to documents and memories. Used for
/// exact-match filtering at recall time.
pub type Metadata = HashMap<String, String>;

/// An embedding vector. May be empty (then the item is not vector-searchable).
pub type Embedding = Vec<f32>;

/// A tenant: the top-level isolation boundary. No recall ever crosses tenants.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tenant {
    /// Stable tenant identifier.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Creation time (epoch milliseconds).
    pub created_at: i64,
}

impl Tenant {
    /// Validate required fields.
    pub fn validate(&self) -> Result<()> {
        non_empty("tenant id", &self.id)
    }
}

/// A collection groups related documents and memories within a tenant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Collection {
    /// Collection name, unique within its tenant.
    pub name: String,
    /// Owning tenant id.
    pub tenant_id: String,
    /// Free-form description.
    pub description: String,
    /// Creation time (epoch milliseconds).
    pub created_at: i64,
}

impl Collection {
    /// Validate required fields.
    pub fn validate(&self) -> Result<()> {
        non_empty("collection name", &self.name)?;
        non_empty("tenant id", &self.tenant_id)
    }
}

/// Provenance for a stored item (where the content came from).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Source {
    /// Short label, e.g. "support-ticket" or "user-note".
    pub label: String,
    /// Optional URI / locator.
    pub uri: Option<String>,
    /// Optional human title.
    pub title: Option<String>,
}

impl Source {
    /// Construct a source from just a label.
    pub fn label(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            uri: None,
            title: None,
        }
    }
}

/// A stored document. Its `text` is split into [`Chunk`]s for retrieval.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    /// Document id, unique within (tenant, collection).
    pub id: String,
    /// Owning tenant.
    pub tenant_id: String,
    /// Owning collection.
    pub collection: String,
    /// Full document text.
    pub text: String,
    /// Exact-match metadata.
    #[serde(default)]
    pub metadata: Metadata,
    /// Optional provenance.
    #[serde(default)]
    pub source: Option<Source>,
    /// Creation time (epoch milliseconds).
    pub created_at: i64,
    /// Last update time (epoch milliseconds).
    pub updated_at: i64,
    /// Version; starts at 0 and increments on overwrite.
    pub version: u64,
}

impl Document {
    /// Validate required fields.
    pub fn validate(&self) -> Result<()> {
        non_empty("document id", &self.id)?;
        non_empty("tenant id", &self.tenant_id)?;
        non_empty("collection", &self.collection)?;
        non_empty("document text", &self.text)
    }
}

/// A retrievable slice of a [`Document`], carrying its own embedding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chunk {
    /// Chunk id (`<document_id>#<ordinal>`).
    pub id: String,
    /// Parent document id.
    pub document_id: String,
    /// Owning tenant.
    pub tenant_id: String,
    /// Owning collection.
    pub collection: String,
    /// Zero-based position within the document.
    pub ordinal: usize,
    /// Chunk text.
    pub text: String,
    /// Embedding for this chunk.
    pub embedding: Embedding,
}

/// The category of a [`Memory`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryType {
    /// A discrete event or observation.
    Episodic,
    /// A durable fact or preference.
    Semantic,
    /// A procedure or instruction.
    Procedural,
    /// Anything else.
    Note,
}

impl MemoryType {
    /// Parse from a case-insensitive string.
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "episodic" => Ok(MemoryType::Episodic),
            "semantic" => Ok(MemoryType::Semantic),
            "procedural" => Ok(MemoryType::Procedural),
            "note" => Ok(MemoryType::Note),
            other => Err(HippocoreError::validation(format!(
                "unknown memory type: {other:?} (expected episodic|semantic|procedural|note)"
            ))),
        }
    }

    /// Stable lowercase string form.
    pub fn as_str(&self) -> &'static str {
        match self {
            MemoryType::Episodic => "episodic",
            MemoryType::Semantic => "semantic",
            MemoryType::Procedural => "procedural",
            MemoryType::Note => "note",
        }
    }
}

/// A long-term memory item belonging to a tenant (and optionally a user).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Memory {
    /// Memory id, unique within (tenant, collection).
    pub id: String,
    /// Owning tenant.
    pub tenant_id: String,
    /// Owning collection.
    pub collection: String,
    /// Optional owning user / agent.
    #[serde(default)]
    pub user_id: Option<String>,
    /// Category of memory.
    pub memory_type: MemoryType,
    /// Memory content.
    pub text: String,
    /// Embedding for this memory.
    #[serde(default)]
    pub embedding: Embedding,
    /// Exact-match metadata.
    #[serde(default)]
    pub metadata: Metadata,
    /// Optional provenance.
    #[serde(default)]
    pub source: Option<Source>,
    /// Creation time (epoch milliseconds).
    pub created_at: i64,
}

impl Memory {
    /// Validate required fields.
    pub fn validate(&self) -> Result<()> {
        non_empty("memory id", &self.id)?;
        non_empty("tenant id", &self.tenant_id)?;
        non_empty("collection", &self.collection)?;
        non_empty("memory text", &self.text)
    }
}

/// A structured JSON record stored in a logical table namespace.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Record {
    /// Record id, unique within (tenant, collection, table).
    pub id: String,
    /// Owning tenant.
    pub tenant_id: String,
    /// Owning collection.
    pub collection: String,
    /// Logical table / dataset namespace.
    pub table: String,
    /// Original structured payload. Must be a JSON object.
    pub payload: serde_json::Value,
    /// Deterministic text projection used for retrieval.
    pub projection: String,
    /// Embedding for the projection.
    #[serde(default)]
    pub embedding: Embedding,
    /// Exact-match metadata.
    #[serde(default)]
    pub metadata: Metadata,
    /// Optional provenance.
    #[serde(default)]
    pub source: Option<Source>,
    /// Creation time (epoch milliseconds).
    pub created_at: i64,
    /// Last update time (epoch milliseconds).
    pub updated_at: i64,
    /// Version; starts at 0 and increments on overwrite.
    pub version: u64,
}

impl Record {
    /// Validate required fields.
    pub fn validate(&self) -> Result<()> {
        non_empty("record id", &self.id)?;
        non_empty("tenant id", &self.tenant_id)?;
        non_empty("collection", &self.collection)?;
        non_empty("table", &self.table)?;
        if !self.payload.is_object() {
            return Err(HippocoreError::validation(
                "record payload must be a JSON object",
            ));
        }
        non_empty("record projection", &self.projection)
    }
}

/// Metadata for an imported file whose extracted text is projected into a
/// derived [`Document`] for retrieval.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileObject {
    /// File id, unique within (tenant, collection).
    pub id: String,
    /// Owning tenant.
    pub tenant_id: String,
    /// Owning collection.
    pub collection: String,
    /// Original path supplied during import.
    pub path: String,
    /// File name from the original path.
    pub name: String,
    /// Detected media type.
    pub media_type: String,
    /// CRC32 checksum of the imported bytes, hex encoded.
    pub checksum: String,
    /// Imported byte size.
    pub size_bytes: u64,
    /// Derived document id containing extracted text.
    pub document_id: String,
    /// Exact-match metadata.
    #[serde(default)]
    pub metadata: Metadata,
    /// Optional provenance.
    #[serde(default)]
    pub source: Option<Source>,
    /// Creation time (epoch milliseconds).
    pub created_at: i64,
    /// Last update time (epoch milliseconds).
    pub updated_at: i64,
    /// Version; starts at 0 and increments on overwrite.
    pub version: u64,
}

impl FileObject {
    /// Validate required fields.
    pub fn validate(&self) -> Result<()> {
        non_empty("file id", &self.id)?;
        non_empty("tenant id", &self.tenant_id)?;
        non_empty("collection", &self.collection)?;
        non_empty("file path", &self.path)?;
        non_empty("file name", &self.name)?;
        non_empty("media type", &self.media_type)?;
        non_empty("checksum", &self.checksum)?;
        non_empty("document id", &self.document_id)
    }
}

/// What kind of item a [`RecallResult`] refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemKind {
    /// A document chunk.
    DocumentChunk,
    /// A memory.
    Memory,
    /// A structured record projected into context.
    Record,
}

/// A single scored recall/search hit, with the reason it was returned.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecallResult {
    /// Id of the underlying chunk or memory.
    pub id: String,
    /// Whether this is a document chunk or a memory.
    pub kind: ItemKind,
    /// Owning tenant.
    pub tenant_id: String,
    /// Owning collection.
    pub collection: String,
    /// Parent document id, when `kind == DocumentChunk`.
    pub document_id: Option<String>,
    /// Table name, when `kind == Record`.
    pub record_table: Option<String>,
    /// Owning user, when present.
    pub user_id: Option<String>,
    /// Memory type, when `kind == Memory`.
    pub memory_type: Option<MemoryType>,
    /// The text that matched.
    pub text: String,
    /// Metadata of the underlying item.
    pub metadata: Metadata,
    /// Provenance, when present.
    pub source: Option<Source>,
    /// Final combined score used for ranking.
    pub score: f32,
    /// Vector component of the score in `[0.0, 1.0]` (0 if not applicable).
    pub vector_score: f32,
    /// Text component of the score in `[0.0, 1.0]` (0 if not applicable).
    pub text_score: f32,
    /// Human-readable explanation of why this result was returned.
    pub reason: String,
}

fn non_empty(field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(HippocoreError::validation(format!(
            "{field} must not be empty"
        )));
    }
    Ok(())
}
