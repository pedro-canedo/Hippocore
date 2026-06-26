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
    Config, Hippocore, PutRecordRequest, RecallRequest, RememberRequest, StoreDocumentRequest,
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
    /// Recall context (hybrid by default; choose --mode for vector/text).
    Recall(RecallArgs),
    /// Forget (delete) a memory by id.
    Forget(IdArgs),
    /// Delete a document and all its chunks by id.
    DeleteDocument(IdArgs),
    /// Delete a structured record by table and id.
    DeleteRecord(RecordIdArgs),
    /// Print database statistics.
    Stats(DbArg),
    /// Inspect tenants, collections and counts.
    Inspect(InspectArgs),
    /// Fold the WAL into a fresh snapshot and truncate the log.
    Compact(DbArg),
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
        Command::Recall(a) => cmd_recall(a),
        Command::Forget(a) => cmd_forget(a),
        Command::DeleteDocument(a) => cmd_delete_document(a),
        Command::DeleteRecord(a) => cmd_delete_record(a),
        Command::Stats(a) => cmd_stats(a),
        Command::Inspect(a) => cmd_inspect(a),
        Command::Compact(a) => cmd_compact(a),
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
