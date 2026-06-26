use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};

use hippocore::BuildContextRequest;

use crate::{types::ServerError, AppState};

#[derive(Deserialize)]
pub struct BuildContextBody {
    pub query: String,
    pub max_tokens: usize,
    pub collection: Option<String>,
    pub include_related: Option<bool>,
}

#[derive(Serialize)]
pub struct ContextResponse {
    pub text: String,
    pub token_count: usize,
    pub items_included: usize,
    pub items_dropped: usize,
}

pub async fn build_context(
    State(state): State<AppState>,
    Path(tid): Path<String>,
    Json(body): Json<BuildContextBody>,
) -> Result<Json<ContextResponse>, ServerError> {
    let mut req = BuildContextRequest::new(&tid, &body.query, body.max_tokens);
    req.collection = body.collection;
    req.include_related = body.include_related.unwrap_or(false);

    let db = state.db.lock().unwrap();
    let block = db.build_context(req).map_err(ServerError::from)?;
    Ok(Json(ContextResponse {
        text: block.text,
        token_count: block.token_count,
        items_included: block.items_included.len(),
        items_dropped: block.items_dropped,
    }))
}
