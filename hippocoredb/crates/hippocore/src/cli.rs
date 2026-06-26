//! Command-line interface logic, shared by the `hippocore` binary.
//!
//! Kept in the library so it can be unit-tested and reused. The binary crate is
//! a thin wrapper that calls [`run`].

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;

use clap::{Args, Parser, Subcommand};

use crate::model::{ItemKind, MemoryType, Source};
use crate::query::SearchMode;
use crate::{
    AddGraphEdgeRequest, BuildContextRequest, Config, Hippocore, ImportFileRequest,
    PutRecordRequest, RecallRequest, RememberRequest, StoreDocumentRequest,
};

/// Hippocore DB command-line interface.
#[derive(Parser)]
#[command(
    name = "hippocore",
    version,
    about = "Local-first AI-native memory database"
)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize a data directory.
    Init(DbArg),
    /// Store a document (chunked and embedded automatically).
    PutDocument(PutDocumentArgs),
    /// Store a memory.
    Remember(RememberArgs),
    /// Store a structured JSON record.
    PutRecord(PutRecordArgs),
    /// Import a text-like file as retrievable context.
    ImportFile(ImportFileArgs),
    /// Recall context (hybrid by default; choose --mode for vector/text).
    Recall(RecallArgs),
    /// Forget (delete) a memory by id.
    Forget(IdArgs),
    /// Delete a document and all its chunks by id.
    DeleteDocument(IdArgs),
    /// Delete a structured record by table and id.
    DeleteRecord(RecordIdArgs),
    /// Delete an imported file and its derived document chunks by id.
    DeleteFile(IdArgs),
    /// Print database statistics.
    Stats(DbArg),
    /// Inspect tenants, collections and counts.
    Inspect(InspectArgs),
    /// Fold the WAL into a fresh snapshot and truncate the log.
    Compact(DbArg),
    /// List all tenants.
    ListTenants(ListTenantsArgs),
    /// List collections (optionally for one tenant).
    ListCollections(ListCollectionsArgs),
    /// List documents in a tenant.
    ListDocuments(ListObjectsArgs),
    /// List memories in a tenant.
    ListMemories(ListObjectsArgs),
    /// List structured records in a tenant.
    ListRecords(ListRecordsArgs),
    /// List imported files in a tenant.
    ListFiles(ListObjectsArgs),
    /// Add a durable graph edge between context items.
    AddEdge(AddEdgeArgs),
    /// List graph edges in a tenant.
    ListEdges(ListEdgesArgs),
    /// Delete a graph edge by id.
    DeleteEdge(DeleteEdgeArgs),
    /// Show a document and its chunks.
    ShowDocument(ShowIdArgs),
    /// Show a memory.
    ShowMemory(ShowIdArgs),
    /// Show a structured record.
    ShowRecord(ShowRecordArgs),
    /// Show an imported file.
    ShowFile(ShowIdArgs),
    /// Run a retrieval quality fixture evaluation and report metrics.
    EvalQuality(EvalQualityArgs),
    /// Assemble a token-budget-aware context block for an LLM prompt.
    BuildContext(BuildContextArgs),
    /// Replay RAG audit records for a tenant and time window.
    Audit(AuditArgs),
    /// Update the confidence score of a stored memory (human-in-the-loop rating).
    RateMemory(RateMemoryArgs),
    /// Compact the audit log, keeping only the newest records within a retention limit.
    CompactAudit(CompactAuditArgs),
}

#[derive(Args)]
struct IdArgs {
    #[arg(long)]
    db: PathBuf,
    #[arg(long)]
    tenant: String,
    #[arg(long)]
    collection: String,
    #[arg(long)]
    id: String,
}

#[derive(Args)]
struct RecordIdArgs {
    #[arg(long)]
    db: PathBuf,
    #[arg(long)]
    tenant: String,
    #[arg(long)]
    collection: String,
    #[arg(long)]
    table: String,
    #[arg(long)]
    id: String,
}

#[derive(Args)]
struct DbArg {
    /// Path to the data directory.
    #[arg(long)]
    db: PathBuf,
}

#[derive(Args)]
struct PutDocumentArgs {
    #[arg(long)]
    db: PathBuf,
    #[arg(long)]
    tenant: String,
    #[arg(long)]
    collection: String,
    #[arg(long)]
    id: Option<String>,
    #[arg(long)]
    text: String,
    #[arg(long = "meta", value_name = "KEY=VALUE")]
    meta: Vec<String>,
    #[arg(long)]
    source: Option<String>,
    /// Validity start (epoch ms). Omit to make the document always valid.
    #[arg(long)]
    valid_from: Option<i64>,
    /// Validity end (epoch ms). Omit to make the document never expire.
    #[arg(long)]
    valid_until: Option<i64>,
}

#[derive(Args)]
struct RememberArgs {
    #[arg(long)]
    db: PathBuf,
    #[arg(long)]
    tenant: String,
    #[arg(long)]
    collection: String,
    #[arg(long)]
    id: Option<String>,
    #[arg(long)]
    user: Option<String>,
    /// Memory type: episodic | semantic | procedural | note.
    #[arg(long = "type", default_value = "note")]
    memory_type: String,
    #[arg(long)]
    text: String,
    #[arg(long = "meta", value_name = "KEY=VALUE")]
    meta: Vec<String>,
    #[arg(long)]
    source: Option<String>,
    /// Optional comma-separated embedding overriding the built-in embedder.
    /// `allow_hyphen_values` lets negative components (e.g. -0.34) through.
    #[arg(long, allow_hyphen_values = true)]
    embedding: Option<String>,
    /// Validity start (epoch ms). Omit to make the memory always valid.
    #[arg(long)]
    valid_from: Option<i64>,
    /// Validity end (epoch ms). Omit to make the memory never expire.
    #[arg(long)]
    valid_until: Option<i64>,
    /// Ids of memories this one replaces (repeatable). Superseded memories
    /// are excluded from default recall.
    #[arg(long, value_name = "ID")]
    supersedes: Vec<String>,
    /// Ids of memories this one contradicts (repeatable). Surfaced as an
    /// advisory in recall results.
    #[arg(long, value_name = "ID")]
    contradicts: Vec<String>,
    /// Optional confidence score in [0.0, 1.0] for this memory.
    #[arg(long)]
    confidence: Option<f32>,
}

#[derive(Args)]
struct PutRecordArgs {
    #[arg(long)]
    db: PathBuf,
    #[arg(long)]
    tenant: String,
    #[arg(long)]
    collection: String,
    #[arg(long)]
    table: String,
    #[arg(long)]
    id: Option<String>,
    /// JSON object payload.
    #[arg(long)]
    json: String,
    #[arg(long = "meta", value_name = "KEY=VALUE")]
    meta: Vec<String>,
    #[arg(long)]
    source: Option<String>,
    /// Optional comma-separated embedding overriding the built-in embedder.
    #[arg(long, allow_hyphen_values = true)]
    embedding: Option<String>,
}

#[derive(Args)]
struct ImportFileArgs {
    #[arg(long)]
    db: PathBuf,
    #[arg(long)]
    tenant: String,
    #[arg(long)]
    collection: String,
    #[arg(long)]
    id: Option<String>,
    #[arg(long)]
    path: PathBuf,
    #[arg(long = "meta", value_name = "KEY=VALUE")]
    meta: Vec<String>,
    #[arg(long)]
    source: Option<String>,
}

#[derive(Args)]
struct RecallArgs {
    #[arg(long)]
    db: PathBuf,
    #[arg(long)]
    tenant: String,
    /// Query text (optional when --embedding is given in vector mode).
    #[arg(long, default_value = "")]
    query: String,
    /// Optional comma-separated query embedding (e.g. from an external model).
    /// `allow_hyphen_values` lets negative components (e.g. -0.34) through.
    #[arg(long, allow_hyphen_values = true)]
    embedding: Option<String>,
    #[arg(long)]
    collection: Option<String>,
    #[arg(long)]
    user: Option<String>,
    #[arg(long = "type")]
    memory_type: Option<String>,
    /// Restrict to "chunk", "memory" or "record".
    #[arg(long)]
    kind: Option<String>,
    #[arg(long = "meta", value_name = "KEY=VALUE")]
    meta: Vec<String>,
    /// Ranking mode: hybrid | vector | text.
    #[arg(long, default_value = "hybrid")]
    mode: String,
    #[arg(long = "top-k", default_value_t = 10)]
    top_k: usize,
    /// Emit results as JSON (for programmatic consumers).
    #[arg(long)]
    json: bool,
    /// Query as of this epoch ms. Defaults to current time; expired entries
    /// (valid_until <= now) are excluded by default.
    #[arg(long)]
    as_of: Option<i64>,
    /// Include memories that have been superseded by a newer one. By default
    /// superseded memories are hidden.
    #[arg(long)]
    include_superseded: bool,
}

#[derive(Args)]
struct InspectArgs {
    #[arg(long)]
    db: PathBuf,
    /// Limit inspection to a single tenant.
    #[arg(long)]
    tenant: Option<String>,
    /// Emit output as JSON.
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct ListTenantsArgs {
    #[arg(long)]
    db: PathBuf,
    /// Emit output as JSON.
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct ListCollectionsArgs {
    #[arg(long)]
    db: PathBuf,
    /// Limit to one tenant.
    #[arg(long)]
    tenant: Option<String>,
    /// Emit output as JSON.
    #[arg(long)]
    json: bool,
}

/// Shared args for list-documents / list-memories / list-files.
#[derive(Args)]
struct ListObjectsArgs {
    #[arg(long)]
    db: PathBuf,
    #[arg(long)]
    tenant: String,
    #[arg(long)]
    collection: Option<String>,
    /// Emit output as JSON.
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct ListRecordsArgs {
    #[arg(long)]
    db: PathBuf,
    #[arg(long)]
    tenant: String,
    #[arg(long)]
    collection: Option<String>,
    #[arg(long)]
    table: Option<String>,
    /// Emit output as JSON.
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct AddEdgeArgs {
    #[arg(long)]
    db: PathBuf,
    #[arg(long)]
    tenant: String,
    #[arg(long)]
    id: Option<String>,
    /// Source kind: chunk | memory | record.
    #[arg(long = "from-kind")]
    from_kind: String,
    #[arg(long = "from-id")]
    from_id: String,
    /// Target kind: chunk | memory | record.
    #[arg(long = "to-kind")]
    to_kind: String,
    #[arg(long = "to-id")]
    to_id: String,
    #[arg(long)]
    relation: String,
    #[arg(long = "meta", value_name = "KEY=VALUE")]
    meta: Vec<String>,
    /// Emit output as JSON.
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct ListEdgesArgs {
    #[arg(long)]
    db: PathBuf,
    #[arg(long)]
    tenant: String,
    #[arg(long = "from-id")]
    from_id: Option<String>,
    /// Emit output as JSON.
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct DeleteEdgeArgs {
    #[arg(long)]
    db: PathBuf,
    #[arg(long)]
    tenant: String,
    #[arg(long)]
    id: String,
}

/// Shared args for show-document / show-memory / show-file.
#[derive(Args)]
struct ShowIdArgs {
    #[arg(long)]
    db: PathBuf,
    #[arg(long)]
    tenant: String,
    #[arg(long)]
    collection: String,
    #[arg(long)]
    id: String,
    /// Emit output as JSON.
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct ShowRecordArgs {
    #[arg(long)]
    db: PathBuf,
    #[arg(long)]
    tenant: String,
    #[arg(long)]
    collection: String,
    #[arg(long)]
    table: String,
    #[arg(long)]
    id: String,
    /// Emit output as JSON.
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct EvalQualityArgs {
    /// Path to a retrieval fixture JSON file.
    #[arg(long)]
    fixture: PathBuf,
    /// Optional path to an existing database to evaluate against.
    /// If omitted, memories are seeded into a fresh temp directory.
    #[arg(long)]
    db: Option<PathBuf>,
    /// Number of results to retrieve per scenario.
    #[arg(long = "top-k", default_value_t = 5)]
    top_k: usize,
    /// Emit report as JSON.
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct RateMemoryArgs {
    #[arg(long)]
    db: PathBuf,
    #[arg(long)]
    tenant: String,
    #[arg(long)]
    collection: String,
    #[arg(long)]
    id: String,
    /// Confidence score in [0.0, 1.0].
    #[arg(long)]
    confidence: f32,
}

#[derive(Args)]
struct BuildContextArgs {
    #[arg(long)]
    db: PathBuf,
    #[arg(long)]
    tenant: String,
    /// Natural-language query for recall.
    #[arg(long)]
    query: String,
    /// Hard token ceiling for the assembled context string.
    #[arg(long = "max-tokens", default_value_t = 2048)]
    max_tokens: usize,
    /// How many candidates to recall before budget trimming.
    #[arg(long = "top-k", default_value_t = 20)]
    top_k: usize,
    /// Ranking mode: hybrid | vector | text.
    #[arg(long, default_value = "hybrid")]
    mode: String,
    /// Restrict recall to a collection.
    #[arg(long)]
    collection: Option<String>,
    #[arg(long = "meta", value_name = "KEY=VALUE")]
    meta: Vec<String>,
    /// Include direct graph neighbours of recalled items.
    #[arg(long)]
    include_related: bool,
    /// Maximum number of graph neighbours to consider.
    #[arg(long = "related-limit", default_value_t = 8)]
    related_limit: usize,
    /// Emit output as JSON.
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct AuditArgs {
    #[arg(long)]
    db: PathBuf,
    /// Inclusive start timestamp (epoch ms).
    #[arg(long)]
    from: i64,
    /// Inclusive end timestamp (epoch ms).
    #[arg(long)]
    to: i64,
    #[arg(long)]
    tenant: String,
    /// Emit output as JSON.
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct CompactAuditArgs {
    #[arg(long)]
    db: PathBuf,
    /// Keep at most this many audit records. Overrides config for this call;
    /// 0 disables this constraint (config still applies to the other dimension).
    #[arg(long = "max-records")]
    max_records: Option<usize>,
    /// Keep at most this many bytes in audit.log. Overrides config for this
    /// call; 0 disables this constraint.
    #[arg(long = "max-bytes")]
    max_bytes: Option<u64>,
    /// Emit output as JSON.
    #[arg(long)]
    json: bool,
}

/// Parse arguments from the process and run, returning a process exit code.
pub fn run() -> ExitCode {
    match dispatch(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn dispatch(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::Init(a) => cmd_init(a),
        Command::PutDocument(a) => cmd_put_document(a),
        Command::Remember(a) => cmd_remember(a),
        Command::PutRecord(a) => cmd_put_record(a),
        Command::ImportFile(a) => cmd_import_file(a),
        Command::Recall(a) => cmd_recall(a),
        Command::Forget(a) => cmd_forget(a),
        Command::DeleteDocument(a) => cmd_delete_document(a),
        Command::DeleteRecord(a) => cmd_delete_record(a),
        Command::DeleteFile(a) => cmd_delete_file(a),
        Command::Stats(a) => cmd_stats(a),
        Command::Inspect(a) => cmd_inspect(a),
        Command::Compact(a) => cmd_compact(a),
        Command::ListTenants(a) => cmd_list_tenants(a),
        Command::ListCollections(a) => cmd_list_collections(a),
        Command::ListDocuments(a) => cmd_list_documents(a),
        Command::ListMemories(a) => cmd_list_memories(a),
        Command::ListRecords(a) => cmd_list_records(a),
        Command::ListFiles(a) => cmd_list_files(a),
        Command::AddEdge(a) => cmd_add_edge(a),
        Command::ListEdges(a) => cmd_list_edges(a),
        Command::DeleteEdge(a) => cmd_delete_edge(a),
        Command::ShowDocument(a) => cmd_show_document(a),
        Command::ShowMemory(a) => cmd_show_memory(a),
        Command::ShowRecord(a) => cmd_show_record(a),
        Command::ShowFile(a) => cmd_show_file(a),
        Command::EvalQuality(a) => cmd_eval_quality(a),
        Command::BuildContext(a) => cmd_build_context(a),
        Command::Audit(a) => cmd_audit(a),
        Command::RateMemory(a) => cmd_rate_memory(a),
        Command::CompactAudit(a) => cmd_compact_audit(a),
    }
}

fn open(db: &PathBuf) -> Result<Hippocore, String> {
    Hippocore::open(Config::new(db)).map_err(|e| format!("failed to open database: {e}"))
}

fn parse_meta(pairs: &[String]) -> Result<HashMap<String, String>, String> {
    let mut map = HashMap::new();
    for p in pairs {
        let (k, v) = p
            .split_once('=')
            .ok_or_else(|| format!("invalid metadata (expected key=value): {p:?}"))?;
        if k.is_empty() {
            return Err(format!("metadata key must not be empty: {p:?}"));
        }
        map.insert(k.to_string(), v.to_string());
    }
    Ok(map)
}

fn parse_embedding(raw: &str) -> Result<Vec<f32>, String> {
    raw.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| {
            s.parse::<f32>()
                .map_err(|_| format!("invalid embedding component: {s:?}"))
        })
        .collect()
}

fn parse_item_kind(raw: &str) -> Result<ItemKind, String> {
    match raw {
        "chunk" | "document" | "document_chunk" => Ok(ItemKind::DocumentChunk),
        "memory" => Ok(ItemKind::Memory),
        "record" => Ok(ItemKind::Record),
        other => Err(format!(
            "invalid kind: {other:?} (expected chunk|memory|record)"
        )),
    }
}

fn cmd_init(a: DbArg) -> Result<(), String> {
    let db = open(&a.db)?;
    db.close().map_err(|e| format!("close failed: {e}"))?;
    println!("initialized data directory at {}", a.db.display());
    Ok(())
}

fn cmd_put_document(a: PutDocumentArgs) -> Result<(), String> {
    let metadata = parse_meta(&a.meta)?;
    let mut db = open(&a.db)?;
    // Ergonomic: ensure tenant + collection exist.
    db.create_tenant(&a.tenant, &a.tenant)
        .map_err(|e| format!("{e}"))?;
    db.create_collection(&a.tenant, &a.collection, "")
        .map_err(|e| format!("{e}"))?;

    let mut req = StoreDocumentRequest::new(&a.tenant, &a.collection, a.text);
    req.id = a.id;
    req.metadata = metadata;
    req.source = a.source.map(Source::label);
    req.valid_from = a.valid_from;
    req.valid_until = a.valid_until;
    let doc = db.store_document(req).map_err(|e| format!("{e}"))?;
    db.close().map_err(|e| format!("close failed: {e}"))?;
    println!(
        "stored document {}/{}/{} (version {})",
        doc.tenant_id, doc.collection, doc.id, doc.version
    );
    Ok(())
}

fn cmd_remember(a: RememberArgs) -> Result<(), String> {
    let memory_type = MemoryType::parse(&a.memory_type).map_err(|e| format!("{e}"))?;
    let metadata = parse_meta(&a.meta)?;
    let embedding = match &a.embedding {
        Some(raw) => Some(parse_embedding(raw)?),
        None => None,
    };
    let mut db = open(&a.db)?;
    db.create_tenant(&a.tenant, &a.tenant)
        .map_err(|e| format!("{e}"))?;
    db.create_collection(&a.tenant, &a.collection, "")
        .map_err(|e| format!("{e}"))?;

    let mut req = RememberRequest::new(&a.tenant, &a.collection, memory_type, a.text);
    req.id = a.id;
    req.user_id = a.user;
    req.metadata = metadata;
    req.source = a.source.map(Source::label);
    req.embedding = embedding;
    req.valid_from = a.valid_from;
    req.valid_until = a.valid_until;
    req.supersedes = a.supersedes;
    req.contradicts = a.contradicts;
    req.confidence = a.confidence;
    let mem = db.remember(req).map_err(|e| format!("{e}"))?;
    db.close().map_err(|e| format!("close failed: {e}"))?;
    println!(
        "stored memory {}/{}/{} ({})",
        mem.tenant_id,
        mem.collection,
        mem.id,
        mem.memory_type.as_str()
    );
    Ok(())
}

fn cmd_put_record(a: PutRecordArgs) -> Result<(), String> {
    let metadata = parse_meta(&a.meta)?;
    let embedding = match &a.embedding {
        Some(raw) => Some(parse_embedding(raw)?),
        None => None,
    };
    let payload: serde_json::Value =
        serde_json::from_str(&a.json).map_err(|e| format!("invalid JSON payload: {e}"))?;
    if !payload.is_object() {
        return Err("record JSON payload must be an object".into());
    }

    let mut db = open(&a.db)?;
    db.create_tenant(&a.tenant, &a.tenant)
        .map_err(|e| format!("{e}"))?;
    db.create_collection(&a.tenant, &a.collection, "")
        .map_err(|e| format!("{e}"))?;

    let mut req = PutRecordRequest::new(&a.tenant, &a.collection, &a.table, payload);
    req.id = a.id;
    req.metadata = metadata;
    req.source = a.source.map(Source::label);
    req.embedding = embedding;
    let record = db.put_record(req).map_err(|e| format!("{e}"))?;
    db.close().map_err(|e| format!("close failed: {e}"))?;
    println!(
        "stored record {}/{}/{}/{} (version {})",
        record.tenant_id, record.collection, record.table, record.id, record.version
    );
    Ok(())
}

fn cmd_import_file(a: ImportFileArgs) -> Result<(), String> {
    let metadata = parse_meta(&a.meta)?;
    let mut db = open(&a.db)?;
    db.create_tenant(&a.tenant, &a.tenant)
        .map_err(|e| format!("{e}"))?;
    db.create_collection(&a.tenant, &a.collection, "")
        .map_err(|e| format!("{e}"))?;

    let mut req = ImportFileRequest::new(&a.tenant, &a.collection, a.path);
    req.id = a.id;
    req.metadata = metadata;
    req.source = a.source.map(Source::label);
    let file = db.import_file(req).map_err(|e| format!("{e}"))?;
    db.close().map_err(|e| format!("close failed: {e}"))?;
    println!(
        "imported file {}/{}/{} -> {} ({} bytes, version {})",
        file.tenant_id, file.collection, file.id, file.document_id, file.size_bytes, file.version
    );
    Ok(())
}

fn cmd_recall(a: RecallArgs) -> Result<(), String> {
    let mode = SearchMode::parse(&a.mode)
        .ok_or_else(|| format!("invalid mode: {:?} (expected hybrid|vector|text)", a.mode))?;
    let memory_type = match &a.memory_type {
        Some(s) => Some(MemoryType::parse(s).map_err(|e| format!("{e}"))?),
        None => None,
    };
    let kind = match a.kind.as_deref() {
        None => None,
        Some(raw) => Some(parse_item_kind(raw)?),
    };
    let metadata = parse_meta(&a.meta)?;
    let embedding = match &a.embedding {
        Some(raw) => Some(parse_embedding(raw)?),
        None => None,
    };

    let db = open(&a.db)?;
    let mut req = RecallRequest::new(&a.tenant, &a.query);
    req.embedding = embedding;
    req.collection = a.collection;
    req.user_id = a.user;
    req.memory_type = memory_type;
    req.kind = kind;
    req.metadata = metadata;
    req.mode = mode;
    req.top_k = a.top_k;
    req.as_of = a.as_of;
    req.include_superseded = a.include_superseded;

    let results = db.search(req).map_err(|e| format!("{e}"))?;

    if a.json {
        let out = serde_json::to_string_pretty(&results)
            .map_err(|e| format!("failed to serialize results: {e}"))?;
        println!("{out}");
        return Ok(());
    }

    if results.is_empty() {
        println!("no results");
        return Ok(());
    }
    println!("{} result(s):", results.len());
    for (i, r) in results.iter().enumerate() {
        println!(
            "{:>2}. [{:?}] {}/{}  score={:.3}  ({})",
            i + 1,
            r.kind,
            r.collection,
            r.id,
            r.score,
            r.reason
        );
        println!("    {}", r.text);
    }
    Ok(())
}

fn cmd_forget(a: IdArgs) -> Result<(), String> {
    let mut db = open(&a.db)?;
    db.forget(&a.tenant, &a.collection, &a.id)
        .map_err(|e| format!("{e}"))?;
    db.close().map_err(|e| format!("close failed: {e}"))?;
    println!("forgot memory {}/{}/{}", a.tenant, a.collection, a.id);
    Ok(())
}

fn cmd_delete_document(a: IdArgs) -> Result<(), String> {
    let mut db = open(&a.db)?;
    db.delete_document(&a.tenant, &a.collection, &a.id)
        .map_err(|e| format!("{e}"))?;
    db.close().map_err(|e| format!("close failed: {e}"))?;
    println!("deleted document {}/{}/{}", a.tenant, a.collection, a.id);
    Ok(())
}

fn cmd_delete_record(a: RecordIdArgs) -> Result<(), String> {
    let mut db = open(&a.db)?;
    db.delete_record(&a.tenant, &a.collection, &a.table, &a.id)
        .map_err(|e| format!("{e}"))?;
    db.close().map_err(|e| format!("close failed: {e}"))?;
    println!(
        "deleted record {}/{}/{}/{}",
        a.tenant, a.collection, a.table, a.id
    );
    Ok(())
}

fn cmd_delete_file(a: IdArgs) -> Result<(), String> {
    let mut db = open(&a.db)?;
    db.delete_file(&a.tenant, &a.collection, &a.id)
        .map_err(|e| format!("{e}"))?;
    db.close().map_err(|e| format!("close failed: {e}"))?;
    println!("deleted file {}/{}/{}", a.tenant, a.collection, a.id);
    Ok(())
}

fn cmd_compact(a: DbArg) -> Result<(), String> {
    let mut db = open(&a.db)?;
    let before = db.stats().map_err(|e| format!("{e}"))?;
    db.compact().map_err(|e| format!("compact failed: {e}"))?;
    let after = db.stats().map_err(|e| format!("{e}"))?;
    db.close().map_err(|e| format!("close failed: {e}"))?;
    println!(
        "compacted: wal entries {} -> {}, disk bytes {} -> {}",
        before.wal_entries, after.wal_entries, before.disk_bytes, after.disk_bytes
    );
    Ok(())
}

fn cmd_stats(a: DbArg) -> Result<(), String> {
    let db = open(&a.db)?;
    let stats = db.stats().map_err(|e| format!("{e}"))?;
    println!("{stats}");
    Ok(())
}

fn cmd_inspect(a: InspectArgs) -> Result<(), String> {
    let db = open(&a.db)?;
    let tenants = db.tenants();
    let filtered: Vec<_> = match &a.tenant {
        Some(t) => tenants.into_iter().filter(|x| &x.id == t).collect(),
        None => tenants,
    };

    if a.json {
        let out: Vec<serde_json::Value> = filtered
            .iter()
            .map(|t| {
                let cols = db.collections(&t.id);
                serde_json::json!({
                    "id": t.id,
                    "name": t.name,
                    "collections": cols.iter().map(|c| &c.name).collect::<Vec<_>>()
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&out).map_err(|e| format!("serialization failed: {e}"))?
        );
        return Ok(());
    }

    if filtered.is_empty() {
        println!("no tenants");
        return Ok(());
    }
    for t in &filtered {
        println!("tenant {} ({})", t.id, t.name);
        for c in db.collections(&t.id) {
            println!("  collection {}", c.name);
        }
    }
    Ok(())
}

fn cmd_list_tenants(a: ListTenantsArgs) -> Result<(), String> {
    let db = open(&a.db)?;
    let tenants = db.tenants();
    if a.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&tenants)
                .map_err(|e| format!("serialization failed: {e}"))?
        );
        return Ok(());
    }
    if tenants.is_empty() {
        println!("no tenants");
        return Ok(());
    }
    for t in &tenants {
        println!("{}\t{}", t.id, t.name);
    }
    Ok(())
}

fn cmd_list_collections(a: ListCollectionsArgs) -> Result<(), String> {
    let db = open(&a.db)?;
    let cols = match &a.tenant {
        Some(t) => db.collections(t),
        None => db.all_collections(),
    };
    if a.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&cols)
                .map_err(|e| format!("serialization failed: {e}"))?
        );
        return Ok(());
    }
    if cols.is_empty() {
        println!("no collections");
        return Ok(());
    }
    for c in &cols {
        println!("{}/{}", c.tenant_id, c.name);
    }
    Ok(())
}

fn cmd_list_documents(a: ListObjectsArgs) -> Result<(), String> {
    let db = open(&a.db)?;
    let docs = db.list_documents(&a.tenant, a.collection.as_deref());
    if a.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&docs)
                .map_err(|e| format!("serialization failed: {e}"))?
        );
        return Ok(());
    }
    if docs.is_empty() {
        println!("no documents");
        return Ok(());
    }
    for d in &docs {
        let preview: String = d.text.chars().take(60).collect();
        println!(
            "{}/{}/{}\tv{}\t{}…",
            d.tenant_id, d.collection, d.id, d.version, preview
        );
    }
    Ok(())
}

fn cmd_list_memories(a: ListObjectsArgs) -> Result<(), String> {
    let db = open(&a.db)?;
    let mems = db.list_memories(&a.tenant, a.collection.as_deref());
    if a.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&mems)
                .map_err(|e| format!("serialization failed: {e}"))?
        );
        return Ok(());
    }
    if mems.is_empty() {
        println!("no memories");
        return Ok(());
    }
    for m in &mems {
        let preview: String = m.text.chars().take(60).collect();
        println!(
            "{}/{}/{}\t{}\t{}…",
            m.tenant_id,
            m.collection,
            m.id,
            m.memory_type.as_str(),
            preview
        );
    }
    Ok(())
}

fn cmd_list_records(a: ListRecordsArgs) -> Result<(), String> {
    let db = open(&a.db)?;
    let recs = db.list_records(&a.tenant, a.collection.as_deref(), a.table.as_deref());
    if a.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&recs)
                .map_err(|e| format!("serialization failed: {e}"))?
        );
        return Ok(());
    }
    if recs.is_empty() {
        println!("no records");
        return Ok(());
    }
    for r in &recs {
        println!(
            "{}/{}/{}/{}\tv{}",
            r.tenant_id, r.collection, r.table, r.id, r.version
        );
    }
    Ok(())
}

fn cmd_list_files(a: ListObjectsArgs) -> Result<(), String> {
    let db = open(&a.db)?;
    let files = db.list_files(&a.tenant, a.collection.as_deref());
    if a.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&files)
                .map_err(|e| format!("serialization failed: {e}"))?
        );
        return Ok(());
    }
    if files.is_empty() {
        println!("no files");
        return Ok(());
    }
    for f in &files {
        println!(
            "{}/{}/{}\t{}\t{} bytes",
            f.tenant_id, f.collection, f.id, f.name, f.size_bytes
        );
    }
    Ok(())
}

fn cmd_add_edge(a: AddEdgeArgs) -> Result<(), String> {
    let metadata = parse_meta(&a.meta)?;
    let from_kind = parse_item_kind(&a.from_kind)?;
    let to_kind = parse_item_kind(&a.to_kind)?;
    let mut db = open(&a.db)?;
    let mut req = AddGraphEdgeRequest::new(
        &a.tenant, from_kind, a.from_id, to_kind, a.to_id, a.relation,
    );
    req.id = a.id;
    req.metadata = metadata;
    let edge = db.add_graph_edge(req).map_err(|e| format!("{e}"))?;
    db.close().map_err(|e| format!("close failed: {e}"))?;

    if a.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&edge)
                .map_err(|e| format!("serialization failed: {e}"))?
        );
    } else {
        println!(
            "added edge {} {}:{} -{}-> {}:{}",
            edge.id,
            edge.from_kind.as_str(),
            edge.from_id,
            edge.relation,
            edge.to_kind.as_str(),
            edge.to_id
        );
    }
    Ok(())
}

fn cmd_list_edges(a: ListEdgesArgs) -> Result<(), String> {
    let db = open(&a.db)?;
    let edges = db.list_graph_edges(&a.tenant, a.from_id.as_deref());
    if a.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&edges)
                .map_err(|e| format!("serialization failed: {e}"))?
        );
        return Ok(());
    }
    if edges.is_empty() {
        println!("no graph edges");
        return Ok(());
    }
    for edge in &edges {
        println!(
            "{}\t{}:{} -{}-> {}:{}",
            edge.id,
            edge.from_kind.as_str(),
            edge.from_id,
            edge.relation,
            edge.to_kind.as_str(),
            edge.to_id
        );
    }
    Ok(())
}

fn cmd_delete_edge(a: DeleteEdgeArgs) -> Result<(), String> {
    let mut db = open(&a.db)?;
    db.delete_graph_edge(&a.tenant, &a.id)
        .map_err(|e| format!("{e}"))?;
    db.close().map_err(|e| format!("close failed: {e}"))?;
    println!("deleted edge {}/{}", a.tenant, a.id);
    Ok(())
}

fn cmd_show_document(a: ShowIdArgs) -> Result<(), String> {
    let db = open(&a.db)?;
    let doc = db
        .get_document(&a.tenant, &a.collection, &a.id)
        .ok_or_else(|| format!("document not found: {}/{}/{}", a.tenant, a.collection, a.id))?;
    let chunks = db.get_document_chunks(&a.tenant, &a.collection, &a.id);

    if a.json {
        let out = serde_json::json!({ "document": doc, "chunks": chunks });
        println!(
            "{}",
            serde_json::to_string_pretty(&out).map_err(|e| format!("serialization failed: {e}"))?
        );
        return Ok(());
    }

    println!(
        "document {}/{}/{} (version {})",
        doc.tenant_id, doc.collection, doc.id, doc.version
    );
    println!("  text:       {}", doc.text);
    println!("  chunks:     {}", chunks.len());
    if !doc.metadata.is_empty() {
        let meta: Vec<String> = doc
            .metadata
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect();
        println!("  metadata:   {}", meta.join(", "));
    }
    if let Some(s) = &doc.source {
        println!("  source:     {}", s.label);
    }
    println!("  created_at: {}", doc.created_at);
    println!("  updated_at: {}", doc.updated_at);
    if !chunks.is_empty() {
        println!();
        for c in &chunks {
            println!("  chunk {} ({}):", c.ordinal, c.id);
            println!("    {}", c.text);
        }
    }
    Ok(())
}

fn cmd_show_memory(a: ShowIdArgs) -> Result<(), String> {
    let db = open(&a.db)?;
    let mem = db
        .get_memory(&a.tenant, &a.collection, &a.id)
        .ok_or_else(|| format!("memory not found: {}/{}/{}", a.tenant, a.collection, a.id))?;

    if a.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&mem).map_err(|e| format!("serialization failed: {e}"))?
        );
        return Ok(());
    }

    println!(
        "memory {}/{}/{} ({})",
        mem.tenant_id,
        mem.collection,
        mem.id,
        mem.memory_type.as_str()
    );
    println!("  text:       {}", mem.text);
    if let Some(u) = &mem.user_id {
        println!("  user:       {u}");
    }
    if !mem.metadata.is_empty() {
        let meta: Vec<String> = mem
            .metadata
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect();
        println!("  metadata:   {}", meta.join(", "));
    }
    if let Some(s) = &mem.source {
        println!("  source:     {}", s.label);
    }
    println!("  created_at: {}", mem.created_at);
    Ok(())
}

fn cmd_show_record(a: ShowRecordArgs) -> Result<(), String> {
    let db = open(&a.db)?;
    let rec = db
        .get_record(&a.tenant, &a.collection, &a.table, &a.id)
        .ok_or_else(|| {
            format!(
                "record not found: {}/{}/{}/{}",
                a.tenant, a.collection, a.table, a.id
            )
        })?;

    if a.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&rec).map_err(|e| format!("serialization failed: {e}"))?
        );
        return Ok(());
    }

    println!(
        "record {}/{}/{}/{} (version {})",
        rec.tenant_id, rec.collection, rec.table, rec.id, rec.version
    );
    println!(
        "  payload:    {}",
        serde_json::to_string(&rec.payload).unwrap_or_default()
    );
    println!("  projection: {}", rec.projection);
    if !rec.metadata.is_empty() {
        let meta: Vec<String> = rec
            .metadata
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect();
        println!("  metadata:   {}", meta.join(", "));
    }
    if let Some(s) = &rec.source {
        println!("  source:     {}", s.label);
    }
    println!("  created_at: {}", rec.created_at);
    println!("  updated_at: {}", rec.updated_at);
    Ok(())
}

fn cmd_show_file(a: ShowIdArgs) -> Result<(), String> {
    let db = open(&a.db)?;
    let file = db
        .get_file(&a.tenant, &a.collection, &a.id)
        .ok_or_else(|| format!("file not found: {}/{}/{}", a.tenant, a.collection, a.id))?;

    if a.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&file)
                .map_err(|e| format!("serialization failed: {e}"))?
        );
        return Ok(());
    }

    println!(
        "file {}/{}/{} ({}, {} bytes, version {})",
        file.tenant_id, file.collection, file.id, file.media_type, file.size_bytes, file.version
    );
    println!("  name:       {}", file.name);
    println!("  path:       {}", file.path);
    println!("  checksum:   {}", file.checksum);
    println!("  document:   {}", file.document_id);
    if !file.metadata.is_empty() {
        let meta: Vec<String> = file
            .metadata
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect();
        println!("  metadata:   {}", meta.join(", "));
    }
    if let Some(s) = &file.source {
        println!("  source:     {}", s.label);
    }
    println!("  created_at: {}", file.created_at);
    println!("  updated_at: {}", file.updated_at);
    Ok(())
}

// ---------------------------------------------------------------------------
// eval-quality
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct EvalFixture {
    version: u32,
    memories: Vec<EvalMemory>,
    cases: Vec<EvalCase>,
    thresholds: EvalThresholds,
}

#[derive(Debug, Deserialize)]
struct EvalMemory {
    id: String,
    text: String,
    #[serde(default)]
    metadata: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct EvalCase {
    name: String,
    query: String,
    #[serde(default)]
    relevant_ids: Vec<String>,
    #[serde(default)]
    expected_first_ids: Vec<String>,
    #[serde(default)]
    forbidden_first_ids: Vec<String>,
    #[serde(default)]
    answer_must_include: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct EvalThresholds {
    min_hit_at_1: f32,
    min_hit_at_k: f32,
    min_mrr: f32,
}

#[derive(Debug)]
struct CaseResult {
    name: String,
    hit_at_1: bool,
    hit_at_k: bool,
    mrr_contribution: f32,
    failures: Vec<String>,
}

fn cmd_eval_quality(a: EvalQualityArgs) -> Result<(), String> {
    let raw =
        std::fs::read_to_string(&a.fixture).map_err(|e| format!("cannot read fixture: {e}"))?;
    let fixture: EvalFixture =
        serde_json::from_str(&raw).map_err(|e| format!("invalid fixture JSON: {e}"))?;
    if fixture.version != 1 {
        return Err(format!(
            "unsupported fixture version {} (expected 1)",
            fixture.version
        ));
    }

    // Open or create a database to evaluate against.
    let _tmp_guard: Option<std::path::PathBuf>;
    let db_path = match &a.db {
        Some(p) => {
            _tmp_guard = None;
            p.clone()
        }
        None => {
            let ts = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0);
            let pid = std::process::id();
            let p = std::env::temp_dir().join(format!("hippocore-eval-{ts}-{pid}"));
            std::fs::create_dir_all(&p).map_err(|e| format!("cannot create temp dir: {e}"))?;
            _tmp_guard = Some(p.clone());
            p
        }
    };

    let mut cfg = Config::new(&db_path);
    cfg.sync_writes = false;
    let mut db = Hippocore::open(cfg).map_err(|e| format!("failed to open database: {e}"))?;

    const TENANT: &str = "eval";
    const COLLECTION: &str = "fixture";
    db.create_tenant(TENANT, "Retrieval Eval")
        .map_err(|e| format!("{e}"))?;
    db.create_collection(TENANT, COLLECTION, "Eval fixture collection")
        .map_err(|e| format!("{e}"))?;

    for mem in &fixture.memories {
        let mut req = RememberRequest::new(
            TENANT,
            COLLECTION,
            crate::model::MemoryType::Semantic,
            mem.text.clone(),
        );
        req.id = Some(mem.id.clone());
        req.metadata = mem.metadata.clone();
        db.remember(req).map_err(|e| format!("{e}"))?;
    }

    let top_k = a.top_k;
    let mut case_results: Vec<CaseResult> = Vec::new();

    for case in &fixture.cases {
        let mut req = RecallRequest::new(TENANT, &case.query);
        req.top_k = top_k;
        let hits = db.recall(req).map_err(|e| format!("{e}"))?;
        let ids: Vec<&str> = hits.iter().map(|h| h.id.as_str()).collect();
        let first = ids.first().copied();

        let rank = ids
            .iter()
            .position(|id| case.relevant_ids.iter().any(|r| r == id))
            .map(|i| i + 1);

        let hit_at_1 = rank == Some(1);
        let hit_at_k = rank.is_some_and(|r| r <= top_k);
        let mrr_contribution = rank.map(|r| 1.0 / r as f32).unwrap_or(0.0);

        let mut failures = Vec::new();

        if !case.expected_first_ids.is_empty()
            && !first.is_some_and(|id| case.expected_first_ids.iter().any(|e| e == id))
        {
            failures.push(format!(
                "expected first result in {:?}, got {first:?}",
                case.expected_first_ids
            ));
        }
        if first.is_some_and(|id| case.forbidden_first_ids.iter().any(|f| f == id)) {
            failures.push(format!("forbidden result {first:?} at top"));
        }
        let retrieved_text = hits
            .iter()
            .map(|h| h.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        for term in &case.answer_must_include {
            if !retrieved_text.to_lowercase().contains(&term.to_lowercase()) {
                failures.push(format!("retrieved context missing required term {term:?}"));
            }
        }

        case_results.push(CaseResult {
            name: case.name.clone(),
            hit_at_1,
            hit_at_k,
            mrr_contribution,
            failures,
        });
    }

    // Cleanup temp dir (best-effort).
    if let Some(ref p) = _tmp_guard {
        let _ = std::fs::remove_dir_all(p);
    }

    let n = case_results.len() as f32;
    let agg_hit_at_1 = case_results.iter().filter(|r| r.hit_at_1).count() as f32 / n;
    let agg_hit_at_k = case_results.iter().filter(|r| r.hit_at_k).count() as f32 / n;
    let agg_mrr = case_results.iter().map(|r| r.mrr_contribution).sum::<f32>() / n;

    let t = &fixture.thresholds;
    let mut threshold_failures: Vec<String> = Vec::new();
    if agg_hit_at_1 < t.min_hit_at_1 {
        threshold_failures.push(format!(
            "hit@1 {agg_hit_at_1:.3} < threshold {:.3}",
            t.min_hit_at_1
        ));
    }
    if agg_hit_at_k < t.min_hit_at_k {
        threshold_failures.push(format!(
            "hit@{top_k} {agg_hit_at_k:.3} < threshold {:.3}",
            t.min_hit_at_k
        ));
    }
    if agg_mrr < t.min_mrr {
        threshold_failures.push(format!("mrr {agg_mrr:.3} < threshold {:.3}", t.min_mrr));
    }

    let overall_pass =
        threshold_failures.is_empty() && case_results.iter().all(|r| r.failures.is_empty());

    if a.json {
        let scenarios: Vec<serde_json::Value> = case_results
            .iter()
            .map(|r| {
                serde_json::json!({
                    "name": r.name,
                    "hit_at_1": r.hit_at_1,
                    "hit_at_k": r.hit_at_k,
                    "mrr": r.mrr_contribution,
                    "pass": r.failures.is_empty(),
                    "failures": r.failures
                })
            })
            .collect();
        let out = serde_json::json!({
            "scenarios": scenarios,
            "aggregate": {
                "hit_at_1": agg_hit_at_1,
                "hit_at_k": agg_hit_at_k,
                "mrr": agg_mrr
            },
            "thresholds": {
                "min_hit_at_1": t.min_hit_at_1,
                "min_hit_at_k": t.min_hit_at_k,
                "min_mrr": t.min_mrr
            },
            "threshold_failures": threshold_failures,
            "pass": overall_pass
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&out).map_err(|e| format!("serialization failed: {e}"))?
        );
    } else {
        println!("Eval quality: {} scenario(s)\n", case_results.len());
        for r in &case_results {
            let status = if r.failures.is_empty() {
                "PASS"
            } else {
                "FAIL"
            };
            println!(
                "  {}: hit@1={} hit@{top_k}={} mrr={:.3}  {status}",
                r.name, r.hit_at_1 as u8, r.hit_at_k as u8, r.mrr_contribution
            );
            for f in &r.failures {
                println!("    ✗ {f}");
            }
        }
        println!(
            "\nAggregate: hit@1={agg_hit_at_1:.3}  hit@{top_k}={agg_hit_at_k:.3}  mrr={agg_mrr:.3}"
        );
        println!(
            "Thresholds: hit@1>={:.3}  hit@{top_k}>={:.3}  mrr>={:.3}",
            t.min_hit_at_1, t.min_hit_at_k, t.min_mrr
        );
        if threshold_failures.is_empty() {
            println!("Result: PASS");
        } else {
            println!("Result: FAIL");
            for f in &threshold_failures {
                println!("  ✗ {f}");
            }
        }
    }

    if overall_pass {
        Ok(())
    } else {
        Err("retrieval quality thresholds not met".into())
    }
}

fn cmd_build_context(a: BuildContextArgs) -> Result<(), String> {
    let metadata = parse_meta(&a.meta)?;
    let mode = SearchMode::parse(&a.mode)
        .ok_or_else(|| format!("unknown mode {:?} (expected hybrid|vector|text)", a.mode))?;

    let db = open(&a.db)?;

    let mut req = BuildContextRequest::new(&a.tenant, &a.query, a.max_tokens);
    req.top_k_candidates = a.top_k;
    req.mode = mode;
    req.collection = a.collection;
    req.metadata_filter = metadata;
    req.include_related = a.include_related;
    req.related_limit = a.related_limit;

    let block = db
        .build_context(req)
        .map_err(|e| format!("build_context failed: {e}"))?;

    if a.json {
        let items: Vec<serde_json::Value> = block
            .items_included
            .iter()
            .map(|item| {
                serde_json::json!({
                    "id": item.id,
                    "kind": format!("{:?}", item.kind).to_lowercase(),
                    "score": item.score,
                    "confidence": item.confidence,
                    "token_count": item.token_count,
                    "snippet": item.snippet,
                    "related_item_ids": item.related_item_ids,
                    "inclusion_source": item.inclusion_source,
                })
            })
            .collect();
        let out = serde_json::json!({
            "text": block.text,
            "token_count": block.token_count,
            "items_included": items,
            "items_dropped": block.items_dropped,
        });
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
    } else {
        println!("{}", block.text);
        println!();
        println!(
            "--- {} items included, {} dropped, ~{} tokens ---",
            block.items_included.len(),
            block.items_dropped,
            block.token_count
        );
    }

    Ok(())
}

fn cmd_audit(a: AuditArgs) -> Result<(), String> {
    let db = open(&a.db)?;
    let records = db
        .query_audit(a.from, a.to, &a.tenant)
        .map_err(|e| format!("{e}"))?;

    if a.json {
        let out = serde_json::to_string_pretty(&records)
            .map_err(|e| format!("failed to serialize audit records: {e}"))?;
        println!("{out}");
        return Ok(());
    }

    if records.is_empty() {
        println!("no audit records");
        return Ok(());
    }

    println!("{} audit record(s):", records.len());
    for record in &records {
        println!(
            "{} tenant={} mode={} tokens={} latency_ms={} dropped={} query={:?}",
            record.timestamp_ms,
            record.tenant_id,
            record.mode,
            record.token_count,
            record.latency_ms,
            record.items_dropped,
            record.query
        );
        for item in &record.items {
            let included = if item.included { "included" } else { "dropped" };
            println!(
                "  [{:?}] {}/{} score={:.3} tokens={} {}",
                item.kind, item.collection, item.id, item.score, item.token_count, included
            );
        }
    }
    Ok(())
}

fn cmd_rate_memory(a: RateMemoryArgs) -> Result<(), String> {
    let mut db = open(&a.db)?;
    let mem = db
        .rate_memory(&a.tenant, &a.collection, &a.id, a.confidence)
        .map_err(|e| format!("{e}"))?;
    db.close().map_err(|e| format!("close failed: {e}"))?;
    println!(
        "rated memory {}/{}/{}: confidence={:.3}",
        mem.tenant_id, mem.collection, mem.id, a.confidence
    );
    Ok(())
}

fn cmd_compact_audit(a: CompactAuditArgs) -> Result<(), String> {
    let db = open(&a.db)?;
    let summary = db
        .compact_audit(a.max_records, a.max_bytes)
        .map_err(|e| format!("compact-audit failed: {e}"))?;

    if a.json {
        let out = serde_json::json!({
            "records_kept": summary.records_kept,
            "records_removed": summary.records_removed,
            "bytes_before": summary.bytes_before,
            "bytes_after": summary.bytes_after,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&out).map_err(|e| format!("serialization failed: {e}"))?
        );
    } else {
        println!(
            "audit compacted: {} kept, {} removed ({} → {} bytes)",
            summary.records_kept,
            summary.records_removed,
            summary.bytes_before,
            summary.bytes_after
        );
    }
    Ok(())
}
