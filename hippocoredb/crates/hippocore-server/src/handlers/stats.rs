use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::{types::ServerError, AppState};

pub async fn get_stats(State(state): State<AppState>) -> Result<Json<Value>, ServerError> {
    let db = state.db.lock().unwrap();
    let s = db.stats().map_err(ServerError::from)?;
    Ok(Json(json!({
        "tenants": s.tenants,
        "collections": s.collections,
        "documents": s.documents,
        "chunks": s.chunks,
        "memories": s.memories,
        "records": s.records,
        "files": s.files,
        "graph_edges": s.graph_edges,
        "indexed_entries": s.indexed_entries,
        "wal_entries": s.wal_entries,
        "disk_bytes": s.disk_bytes,
        "audit_records": s.audit_records,
        "audit_log_bytes": s.audit_log_bytes,
    })))
}
