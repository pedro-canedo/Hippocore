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
//!     };
//!     hippocore_server::serve(cfg).await.unwrap();
//! }
//! ```

mod auth;
mod handlers;
mod types;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::{
    http::StatusCode,
    middleware,
    routing::{delete, get, post},
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
}

/// Shared application state across all request handlers.
#[derive(Clone)]
pub struct AppState {
    pub(crate) db: Arc<Mutex<Hippocore>>,
    pub(crate) api_key: String,
}

impl AppState {
    /// Create a new shared state from an open database and API key.
    pub fn new(db: Hippocore, api_key: impl Into<String>) -> Self {
        Self {
            db: Arc::new(Mutex::new(db)),
            api_key: api_key.into(),
        }
    }
}

/// Open the database and run the HTTP server until a shutdown signal.
pub async fn serve(cfg: ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    let db = Hippocore::open(Config::new(&cfg.data_dir))?;
    let state = AppState {
        db: Arc::new(Mutex::new(db)),
        api_key: cfg.api_key.clone(),
    };

    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", cfg.port)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

/// Build the axum `Router` — extracted so tests can call it without binding a port.
pub fn build_router(state: AppState) -> Router {
    // Routes that require authentication.
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
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_api_key,
        ));

    Router::new()
        .route("/health", get(health))
        .merge(protected)
        .with_state(state)
}

async fn health() -> StatusCode {
    StatusCode::OK
}
