use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use hippocore::model::MemoryType;
use hippocore::RememberRequest;

use crate::{types::ServerError, AppState};

#[derive(Deserialize)]
pub struct RememberBody {
    pub collection: String,
    pub text: String,
    pub memory_type: Option<String>,
}

#[derive(Serialize)]
pub struct MemoryResponse {
    pub id: String,
    pub tenant_id: String,
    pub collection: String,
    pub text: String,
}

pub async fn remember(
    State(state): State<AppState>,
    Path(tid): Path<String>,
    Json(body): Json<RememberBody>,
) -> Result<(StatusCode, Json<MemoryResponse>), ServerError> {
    let mtype = match body.memory_type.as_deref() {
        Some("episodic") => MemoryType::Episodic,
        Some("procedural") => MemoryType::Procedural,
        _ => MemoryType::Semantic,
    };

    let mut db = state.db.lock().unwrap();
    let mem = db
        .remember(RememberRequest::new(
            &tid,
            &body.collection,
            mtype,
            &body.text,
        ))
        .map_err(ServerError::from)?;
    Ok((
        StatusCode::CREATED,
        Json(MemoryResponse {
            id: mem.id,
            tenant_id: mem.tenant_id,
            collection: mem.collection,
            text: mem.text,
        }),
    ))
}

pub async fn forget(
    State(state): State<AppState>,
    Path((tid, id)): Path<(String, String)>,
) -> Result<StatusCode, ServerError> {
    // collection is not part of the path; use a sentinel and fall back to
    // iterating all collections by searching the state directly via forget.
    // forget() requires a collection name, so we scan to find it.
    let collection = {
        let db = state.db.lock().unwrap();
        db.list_memories(&tid, None)
            .into_iter()
            .find(|m| m.id == id)
            .map(|m| m.collection)
    };
    let collection = match collection {
        Some(c) => c,
        None => return Ok(StatusCode::NOT_FOUND),
    };

    let mut db = state.db.lock().unwrap();
    db.forget(&tid, &collection, &id)
        .map_err(ServerError::from)?;
    Ok(StatusCode::NO_CONTENT)
}
