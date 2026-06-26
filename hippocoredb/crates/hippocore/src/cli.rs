//! Command-line interface logic, shared by the `hippocore` binary.
//!
//! Kept in the library so it can be unit-tested and reused. The binary crate is
//! a thin wrapper that calls [`run`].

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};

use crate::model::{ItemKind, MemoryType, Source};
use crate::query::SearchMode;
use crate::{
    Config, Hippocore, ImportFileRequest, PutRecordRequest, RecallRequest, RememberRequest,
    StoreDocumentRequest,
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
    /// Show a document and its chunks.
    ShowDocument(ShowIdArgs),
    /// Show a memory.
    ShowMemory(ShowIdArgs),
    /// Show a structured record.
    ShowRecord(ShowRecordArgs),
    /// Show an imported file.
    ShowFile(ShowIdArgs),
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
        Command::ShowDocument(a) => cmd_show_document(a),
        Command::ShowMemory(a) => cmd_show_memory(a),
        Command::ShowRecord(a) => cmd_show_record(a),
        Command::ShowFile(a) => cmd_show_file(a),
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
        Some("chunk") | Some("document") => Some(ItemKind::DocumentChunk),
        Some("memory") => Some(ItemKind::Memory),
        Some("record") => Some(ItemKind::Record),
        Some(other) => {
            return Err(format!(
                "invalid kind: {other:?} (expected chunk|memory|record)"
            ))
        }
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
