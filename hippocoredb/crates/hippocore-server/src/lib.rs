//! Hippocore DB HTTP server.
//!
//! Exposes the full core library over a REST/JSON API secured with a
//! bearer API key (`X-Api-Key` header). The server is single-process and
//! shares one [`Hippocore`] instance behind an `Arc<Mutex<_>>`.
//!
//! # Quick start
//!
//! ```no_run
//! use hippocore_server::ServerConfig;
//!
//! #[tokio::main]
//! async fn main() {
//!     let cfg = ServerConfig {
//!         data_dir: "./hippocore-data".into(),
//!         port: 8080,
//!         api_key: "secret".into(),
//!         admin_username: "admin".into(),
//!         admin_password: "change-me".into(),
//!     };
//!     hippocore_server::serve(cfg).await.unwrap();
//! }
//! ```

mod admin;
mod auth;
mod handlers;
mod types;

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::{
    http::StatusCode,
    middleware,
    routing::{delete, get, post, put},
    Router,
};

use hippocore::{Config, Hippocore};

pub use types::ServerError;

/// Runtime configuration for the HTTP server.
#[derive(Clone)]
pub struct ServerConfig {
    /// Path to the Hippocore data directory.
    pub data_dir: PathBuf,
    /// TCP port to listen on.
    pub port: u16,
    /// API key required in the `X-Api-Key` request header.
    pub api_key: String,
    /// Bootstrap username for human admin login.
    pub admin_username: String,
    /// Bootstrap password for human admin login.
    pub admin_password: String,
}

/// Shared application state across all request handlers.
#[derive(Clone)]
pub struct AppState {
    pub(crate) db: Arc<Mutex<Hippocore>>,
    pub(crate) api_key: Arc<Mutex<String>>,
    pub(crate) admin_username: Arc<String>,
    pub(crate) admin_password: Arc<String>,
    pub(crate) admin_sessions: Arc<Mutex<HashSet<String>>>,
    pub(crate) llm_providers: Arc<Mutex<Vec<admin::LlmProviderConfig>>>,
    pub(crate) http_client: reqwest::Client,
    pub(crate) port: u16,
}

impl AppState {
    /// Create a new shared state from an open database and API key.
    pub fn new(db: Hippocore, api_key: impl Into<String>) -> Self {
        Self::with_admin_credentials(db, api_key, "admin", "admin")
    }

    /// Create shared state with explicit bootstrap admin credentials.
    pub fn with_admin_credentials(
        db: Hippocore,
        api_key: impl Into<String>,
        admin_username: impl Into<String>,
        admin_password: impl Into<String>,
    ) -> Self {
        let providers = admin::load_llm_providers(db.config().data_dir.as_path());
        Self {
            db: Arc::new(Mutex::new(db)),
            api_key: Arc::new(Mutex::new(api_key.into())),
            admin_username: Arc::new(admin_username.into()),
            admin_password: Arc::new(admin_password.into()),
            admin_sessions: Arc::new(Mutex::new(HashSet::new())),
            llm_providers: Arc::new(Mutex::new(providers)),
            http_client: reqwest::Client::new(),
            port: 8080,
        }
    }
}

/// Open the database and run the HTTP server until a shutdown signal.
pub async fn serve(cfg: ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    let db = Hippocore::open(Config::new(&cfg.data_dir))?;
    let providers = admin::load_llm_providers(&cfg.data_dir);
    let port = cfg.port;
    let state = AppState {
        db: Arc::new(Mutex::new(db)),
        api_key: Arc::new(Mutex::new(cfg.api_key)),
        admin_username: Arc::new(cfg.admin_username),
        admin_password: Arc::new(cfg.admin_password),
        admin_sessions: Arc::new(Mutex::new(HashSet::new())),
        llm_providers: Arc::new(Mutex::new(providers)),
        http_client: reqwest::Client::new(),
        port,
    };

    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

/// Build the axum `Router` — extracted so tests can call it without binding a port.
pub fn build_router(state: AppState) -> Router {
    let public = Router::new()
        .route("/admin", get(admin::page))
        .route("/admin/styles.css", get(admin::styles))
        .route("/admin/app.js", get(admin::app_js))
        .route("/admin/login", post(admin::login))
        .route("/", get(admin::root));

    // Human admin routes require an admin session. The service API key is also
    // accepted here so existing automation can still inspect admin endpoints.
    let admin_protected = Router::new()
        .route("/admin/bootstrap", get(admin::bootstrap))
        .route("/admin/config", get(admin::config))
        .route("/admin/api-key/rotate", post(admin::rotate_api_key))
        .route("/admin/tenants", get(admin::tenants))
        .route("/admin/collections", get(admin::collections))
        .route("/admin/memories", get(admin::memories))
        .route("/admin/documents", get(admin::documents))
        .route("/admin/records", get(admin::records))
        .route("/admin/files", get(admin::files))
        .route("/admin/graph-edges", get(admin::graph_edges))
        .route("/admin/query-records", post(admin::query_records))
        .route("/admin/sql", post(admin::execute_sql))
        .route("/admin/llm-providers", get(admin::llm_providers))
        .route("/admin/llm-providers", post(admin::upsert_llm_provider))
        .route(
            "/admin/llm-providers/validate",
            post(admin::validate_llm_provider),
        )
        .route("/admin/llm-providers/ping", post(admin::ping_llm_provider))
        .route("/admin/api-info", get(admin::api_info))
        .route("/admin/prompts", get(handlers::prompts::list_prompts))
        .route("/admin/prompts", post(handlers::prompts::create_prompt))
        .route("/admin/prompts/{id}", put(handlers::prompts::update_prompt))
        .route(
            "/admin/prompts/{id}",
            delete(handlers::prompts::delete_prompt),
        )
        .route("/admin/tenants", post(handlers::tenants::create_tenant))
        .route(
            "/admin/tenants/{tid}/collections",
            post(handlers::collections::create_collection),
        )
        .route(
            "/admin/tenants/{tid}/memories",
            post(handlers::memories::remember),
        )
        .route(
            "/admin/tenants/{tid}/records",
            post(handlers::records::put_record),
        )
        .route(
            "/admin/tenants/{tid}/documents",
            post(handlers::documents::store_document),
        )
        .route(
            "/admin/tenants/{tid}/files",
            post(handlers::files::upload_file),
        )
        .route(
            "/admin/tenants/{tid}/tables",
            get(handlers::tables::list_tables),
        )
        .route(
            "/admin/tenants/{tid}/tables/{table}",
            delete(handlers::tables::delete_table),
        )
        .route(
            "/admin/tenants/{tid}/recall",
            post(handlers::recall::recall),
        )
        .route(
            "/admin/tenants/{tid}/context",
            post(handlers::context::build_context),
        )
        .route("/admin/tenants/{tid}/chat", post(handlers::chat::chat))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_admin_session,
        ));

    // Service API routes require the machine-to-machine API key.
    let protected = Router::new()
        .route("/stats", get(handlers::stats::get_stats))
        .route("/tenants", post(handlers::tenants::create_tenant))
        .route(
            "/tenants/{tid}/collections",
            post(handlers::collections::create_collection),
        )
        .route(
            "/tenants/{tid}/memories",
            post(handlers::memories::remember),
        )
        .route(
            "/tenants/{tid}/memories/{id}",
            delete(handlers::memories::forget),
        )
        .route("/tenants/{tid}/recall", post(handlers::recall::recall))
        .route(
            "/tenants/{tid}/context",
            post(handlers::context::build_context),
        )
        .route(
            "/tenants/{tid}/documents",
            post(handlers::documents::store_document),
        )
        .route(
            "/tenants/{tid}/graph/traverse",
            post(handlers::graph::traverse),
        )
        .route("/tenants/{tid}/chat", post(handlers::chat::chat))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_api_key,
        ));

    Router::new()
        .merge(public)
        .route("/health", get(health))
        .merge(admin_protected)
        .merge(protected)
        .with_state(state)
}

async fn health() -> StatusCode {
    StatusCode::OK
}
