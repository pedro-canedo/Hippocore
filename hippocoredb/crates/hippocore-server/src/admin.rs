use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Json, Redirect},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::{types::ServerError, AppState};

pub async fn root() -> Redirect {
    Redirect::to("/admin")
}

pub async fn page() -> Html<&'static str> {
    Html(ADMIN_HTML)
}

pub async fn styles() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        ADMIN_CSS,
    )
}

pub async fn app_js() -> impl IntoResponse {
    (
        [(
            header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )],
        ADMIN_JS,
    )
}

#[derive(Serialize)]
pub struct AdminConfigResponse {
    pub api_key_set: bool,
    pub api_key_length: usize,
    pub data_dir: String,
    pub admin_user_set: bool,
}

#[derive(Serialize)]
pub struct AdminBootstrapResponse {
    pub config: AdminConfigResponse,
    pub stats: serde_json::Value,
    pub tenants: Vec<hippocore::Tenant>,
}

#[derive(Serialize)]
pub struct RotatedKeyResponse {
    pub api_key: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub session: String,
    pub username: String,
}

#[derive(Deserialize, Default)]
pub struct CollectionQuery {
    pub tenant_id: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct EntityQuery {
    pub tenant_id: Option<String>,
    pub collection: Option<String>,
    pub table: Option<String>,
    pub from_id: Option<String>,
}

#[derive(Deserialize)]
pub struct RotateKeyRequest {
    pub length: Option<usize>,
}

#[derive(Deserialize)]
pub struct QueryRecordsRequest {
    pub tenant_id: String,
    pub sql: String,
}

#[derive(Deserialize)]
pub struct SqlRequest {
    pub tenant_id: String,
    pub sql: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct LlmProviderConfig {
    pub id: String,
    pub kind: String,
    pub base_url: String,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(default)]
    pub is_default: bool,
}

#[derive(Serialize)]
pub struct LlmProviderView {
    pub id: String,
    pub kind: String,
    pub base_url: String,
    pub model: String,
    pub api_key_set: bool,
    pub is_default: bool,
}

#[derive(Serialize)]
pub struct ProviderValidationResponse {
    pub ok: bool,
    pub message: String,
    pub snippet: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ServerError> {
    if body.username != *state.admin_username || body.password != *state.admin_password {
        return Err(ServerError(
            StatusCode::UNAUTHORIZED,
            "invalid admin credentials".into(),
        ));
    }
    let session = generate_api_key(32)?;
    state.admin_sessions.lock().unwrap().insert(session.clone());
    Ok(Json(LoginResponse {
        session,
        username: body.username,
    }))
}

pub async fn config(
    State(state): State<AppState>,
) -> Result<Json<AdminConfigResponse>, ServerError> {
    let key = state.api_key.lock().unwrap();
    let db = state.db.lock().unwrap();
    let data_dir = db.config().data_dir.to_string_lossy().to_string();
    Ok(Json(AdminConfigResponse {
        api_key_set: !key.is_empty(),
        api_key_length: key.len(),
        data_dir,
        admin_user_set: !state.admin_username.is_empty() && !state.admin_password.is_empty(),
    }))
}

pub async fn bootstrap(
    State(state): State<AppState>,
) -> Result<Json<AdminBootstrapResponse>, ServerError> {
    let config = {
        let key = state.api_key.lock().unwrap();
        let db = state.db.lock().unwrap();
        AdminConfigResponse {
            api_key_set: !key.is_empty(),
            api_key_length: key.len(),
            data_dir: db.config().data_dir.to_string_lossy().to_string(),
            admin_user_set: !state.admin_username.is_empty() && !state.admin_password.is_empty(),
        }
    };
    let (stats, tenants) = {
        let db = state.db.lock().unwrap();
        let stats = db.stats().map_err(ServerError::from)?;
        (stats, db.tenants())
    };
    Ok(Json(AdminBootstrapResponse {
        stats: serde_json::json!({
            "tenants": stats.tenants,
            "collections": stats.collections,
            "documents": stats.documents,
            "chunks": stats.chunks,
            "memories": stats.memories,
            "records": stats.records,
            "files": stats.files,
            "graph_edges": stats.graph_edges,
            "indexed_entries": stats.indexed_entries,
            "wal_entries": stats.wal_entries,
            "disk_bytes": stats.disk_bytes,
            "audit_records": stats.audit_records,
            "audit_log_bytes": stats.audit_log_bytes,
        }),
        config,
        tenants,
    }))
}

pub async fn rotate_api_key(
    State(state): State<AppState>,
    Json(body): Json<RotateKeyRequest>,
) -> Result<Json<RotatedKeyResponse>, ServerError> {
    let len = body.length.unwrap_or(32).max(16);
    let api_key = generate_api_key(len)?;
    *state.api_key.lock().unwrap() = api_key.clone();
    Ok(Json(RotatedKeyResponse { api_key }))
}

pub async fn tenants(
    State(state): State<AppState>,
) -> Result<Json<Vec<hippocore::Tenant>>, ServerError> {
    let db = state.db.lock().unwrap();
    Ok(Json(db.tenants()))
}

pub async fn collections(
    State(state): State<AppState>,
    Query(query): Query<CollectionQuery>,
) -> Result<Json<Vec<hippocore::Collection>>, ServerError> {
    let db = state.db.lock().unwrap();
    let items = match query.tenant_id {
        Some(ref tenant_id) => db.collections(tenant_id),
        None => db.all_collections(),
    };
    Ok(Json(items))
}

pub async fn memories(
    State(state): State<AppState>,
    Query(query): Query<EntityQuery>,
) -> Result<Json<Vec<hippocore::Memory>>, ServerError> {
    let tenant_id = query
        .tenant_id
        .ok_or_else(|| ServerError(StatusCode::BAD_REQUEST, "tenant_id required".into()))?;
    let db = state.db.lock().unwrap();
    Ok(Json(
        db.list_memories(&tenant_id, query.collection.as_deref()),
    ))
}

pub async fn documents(
    State(state): State<AppState>,
    Query(query): Query<EntityQuery>,
) -> Result<Json<Vec<hippocore::Document>>, ServerError> {
    let tenant_id = query
        .tenant_id
        .ok_or_else(|| ServerError(StatusCode::BAD_REQUEST, "tenant_id required".into()))?;
    let db = state.db.lock().unwrap();
    Ok(Json(
        db.list_documents(&tenant_id, query.collection.as_deref()),
    ))
}

pub async fn records(
    State(state): State<AppState>,
    Query(query): Query<EntityQuery>,
) -> Result<Json<Vec<hippocore::Record>>, ServerError> {
    let tenant_id = query
        .tenant_id
        .ok_or_else(|| ServerError(StatusCode::BAD_REQUEST, "tenant_id required".into()))?;
    let db = state.db.lock().unwrap();
    Ok(Json(db.list_records(
        &tenant_id,
        query.collection.as_deref(),
        query.table.as_deref(),
    )))
}

pub async fn files(
    State(state): State<AppState>,
    Query(query): Query<EntityQuery>,
) -> Result<Json<Vec<hippocore::FileObject>>, ServerError> {
    let tenant_id = query
        .tenant_id
        .ok_or_else(|| ServerError(StatusCode::BAD_REQUEST, "tenant_id required".into()))?;
    let db = state.db.lock().unwrap();
    Ok(Json(db.list_files(&tenant_id, query.collection.as_deref())))
}

pub async fn graph_edges(
    State(state): State<AppState>,
    Query(query): Query<EntityQuery>,
) -> Result<Json<Vec<hippocore::GraphEdge>>, ServerError> {
    let tenant_id = query
        .tenant_id
        .ok_or_else(|| ServerError(StatusCode::BAD_REQUEST, "tenant_id required".into()))?;
    let db = state.db.lock().unwrap();
    Ok(Json(
        db.list_graph_edges(&tenant_id, query.from_id.as_deref()),
    ))
}

pub async fn query_records(
    State(state): State<AppState>,
    Json(body): Json<QueryRecordsRequest>,
) -> Result<Json<Vec<hippocore::Record>>, ServerError> {
    let db = state.db.lock().unwrap();
    Ok(Json(
        db.query_records_restricted(&body.tenant_id, &body.sql)?,
    ))
}

pub async fn execute_sql(
    State(state): State<AppState>,
    Json(body): Json<SqlRequest>,
) -> Result<Json<hippocore::SqlResult>, ServerError> {
    let db = state.db.lock().unwrap();
    Ok(Json(db.execute_sql(&body.tenant_id, &body.sql)?))
}

pub async fn llm_providers(
    State(state): State<AppState>,
) -> Result<Json<Vec<LlmProviderView>>, ServerError> {
    let providers = state.llm_providers.lock().unwrap();
    Ok(Json(providers.iter().map(provider_view).collect()))
}

pub async fn upsert_llm_provider(
    State(state): State<AppState>,
    Json(mut body): Json<LlmProviderConfig>,
) -> Result<Json<LlmProviderView>, ServerError> {
    validate_provider_config(&body)?;
    if body.api_key.as_deref() == Some("") {
        body.api_key = None;
    }

    let data_dir = {
        let db = state.db.lock().unwrap();
        db.config().data_dir.clone()
    };
    let mut providers = state.llm_providers.lock().unwrap();
    if body.is_default {
        for provider in providers.iter_mut() {
            provider.is_default = false;
        }
    }
    if let Some(existing) = providers.iter_mut().find(|provider| provider.id == body.id) {
        *existing = body.clone();
    } else {
        providers.push(body.clone());
    }
    save_llm_providers(&data_dir, &providers)?;
    Ok(Json(provider_view(&body)))
}

pub async fn validate_llm_provider(
    Json(body): Json<LlmProviderConfig>,
) -> Result<Json<ProviderValidationResponse>, ServerError> {
    validate_provider_config(&body)?;
    Ok(Json(ProviderValidationResponse {
        ok: true,
        message: "provider configuration is valid locally".to_string(),
        snippet: provider_snippet(&body),
    }))
}

pub(crate) fn load_llm_providers(data_dir: &Path) -> Vec<LlmProviderConfig> {
    let path = llm_providers_path(data_dir);
    if let Ok(bytes) = std::fs::read(&path) {
        if let Ok(providers) = serde_json::from_slice::<Vec<LlmProviderConfig>>(&bytes) {
            return providers;
        }
    }

    let mut providers = Vec::new();
    let ollama_url = std::env::var("HIPPOCORE_OLLAMA_URL").or_else(|_| std::env::var("OLLAMA_URL"));
    if let Ok(base_url) = ollama_url {
        providers.push(LlmProviderConfig {
            id: "ollama".to_string(),
            kind: "ollama".to_string(),
            base_url,
            model: std::env::var("HIPPOCORE_OLLAMA_MODEL")
                .or_else(|_| std::env::var("CHAT_MODEL"))
                .unwrap_or_else(|_| "llama3.2".to_string()),
            api_key: None,
            is_default: true,
        });
    }
    if let Ok(api_key) = std::env::var("OPENROUTER_API_KEY") {
        providers.push(LlmProviderConfig {
            id: "openrouter".to_string(),
            kind: "openrouter".to_string(),
            base_url: std::env::var("OPENROUTER_BASE_URL")
                .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string()),
            model: std::env::var("OPENROUTER_MODEL")
                .unwrap_or_else(|_| "openai/gpt-4.1-mini".to_string()),
            api_key: Some(api_key),
            is_default: providers.is_empty(),
        });
    }
    providers
}

fn generate_api_key(len: usize) -> Result<String, ServerError> {
    let byte_len = len.max(16);
    let mut bytes = vec![0u8; byte_len];
    getrandom::fill(&mut bytes).map_err(|e| {
        ServerError(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to generate api key: {e}"),
        )
    })?;
    Ok(bytes.into_iter().map(|b| format!("{:02x}", b)).collect())
}

fn provider_view(provider: &LlmProviderConfig) -> LlmProviderView {
    LlmProviderView {
        id: provider.id.clone(),
        kind: provider.kind.clone(),
        base_url: provider.base_url.clone(),
        model: provider.model.clone(),
        api_key_set: provider.api_key.as_ref().is_some_and(|key| !key.is_empty()),
        is_default: provider.is_default,
    }
}

fn validate_provider_config(provider: &LlmProviderConfig) -> Result<(), ServerError> {
    for (label, value) in [
        ("provider id", &provider.id),
        ("provider kind", &provider.kind),
        ("base url", &provider.base_url),
        ("model", &provider.model),
    ] {
        if value.trim().is_empty() {
            return Err(ServerError(
                StatusCode::BAD_REQUEST,
                format!("{label} is required"),
            ));
        }
    }
    if !(provider.base_url.starts_with("http://") || provider.base_url.starts_with("https://")) {
        return Err(ServerError(
            StatusCode::BAD_REQUEST,
            "base url must start with http:// or https://".into(),
        ));
    }
    Ok(())
}

fn save_llm_providers(data_dir: &Path, providers: &[LlmProviderConfig]) -> Result<(), ServerError> {
    std::fs::create_dir_all(data_dir).map_err(|e| {
        ServerError(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to create data dir: {e}"),
        )
    })?;
    let bytes = serde_json::to_vec_pretty(providers).map_err(|e| {
        ServerError(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to encode providers: {e}"),
        )
    })?;
    std::fs::write(llm_providers_path(data_dir), bytes).map_err(|e| {
        ServerError(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to save providers: {e}"),
        )
    })
}

fn llm_providers_path(data_dir: &Path) -> std::path::PathBuf {
    data_dir.join("llm-providers.json")
}

fn provider_snippet(provider: &LlmProviderConfig) -> String {
    format!(
        "HIPPOCORE_LLM_PROVIDER={}\nHIPPOCORE_LLM_BASE_URL={}\nHIPPOCORE_LLM_MODEL={}\nHIPPOCORE_LLM_API_KEY={}",
        provider.id,
        provider.base_url,
        provider.model,
        if provider.api_key.as_ref().is_some_and(|key| !key.is_empty()) {
            "<stored locally>"
        } else {
            "<set-if-required>"
        }
    )
}

const ADMIN_HTML: &str = include_str!("admin/index.html");
const ADMIN_CSS: &str = include_str!("admin/styles.css");
const ADMIN_JS: &str = include_str!("admin/app.js");
