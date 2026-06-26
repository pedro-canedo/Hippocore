use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{types::ServerError, AppState};

#[derive(Deserialize)]
pub struct CreateCollectionBody {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct CollectionResponse {
    pub name: String,
    pub tenant_id: String,
    pub description: String,
}

pub async fn create_collection(
    State(state): State<AppState>,
    Path(tid): Path<String>,
    Json(body): Json<CreateCollectionBody>,
) -> Result<(StatusCode, Json<CollectionResponse>), ServerError> {
    let mut db = state.db.lock().unwrap();
    let col = db
        .create_collection(&tid, &body.name, body.description.as_deref().unwrap_or(""))
        .map_err(ServerError::from)?;
    Ok((
        StatusCode::CREATED,
        Json(CollectionResponse {
            name: col.name,
            tenant_id: col.tenant_id,
            description: col.description,
        }),
    ))
}
