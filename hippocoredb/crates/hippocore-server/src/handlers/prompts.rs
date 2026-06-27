use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{types::ServerError, AppState};
use hippocore::{SystemPrompt, UpsertPromptRequest};

#[derive(Deserialize, Default)]
pub struct PromptQuery {
    pub tenant_id: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct UpsertPromptBody {
    pub id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub content: String,
    pub tenant_id: Option<String>,
}

pub async fn list_prompts(
    State(state): State<AppState>,
    Query(query): Query<PromptQuery>,
) -> Result<Json<Vec<SystemPrompt>>, ServerError> {
    let db = state.db.lock().unwrap();
    Ok(Json(db.list_prompts(query.tenant_id.as_deref())))
}

pub async fn create_prompt(
    State(state): State<AppState>,
    Json(body): Json<UpsertPromptBody>,
) -> Result<(StatusCode, Json<SystemPrompt>), ServerError> {
    let mut db = state.db.lock().unwrap();
    let prompt = db.upsert_prompt(UpsertPromptRequest {
        id: body.id,
        name: body.name,
        description: body.description,
        content: body.content,
        tenant_id: body.tenant_id,
    })?;
    Ok((StatusCode::CREATED, Json(prompt)))
}

pub async fn update_prompt(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpsertPromptBody>,
) -> Result<Json<SystemPrompt>, ServerError> {
    let mut db = state.db.lock().unwrap();
    let prompt = db.upsert_prompt(UpsertPromptRequest {
        id: Some(id),
        name: body.name,
        description: body.description,
        content: body.content,
        tenant_id: body.tenant_id,
    })?;
    Ok(Json(prompt))
}

pub async fn delete_prompt(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ServerError> {
    let mut db = state.db.lock().unwrap();
    db.delete_prompt(&id)?;
    Ok(StatusCode::NO_CONTENT)
}
