use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::AppState;

/// Axum middleware: rejects requests without a matching `X-Api-Key` header.
pub async fn require_api_key(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let key = req
        .headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let current_key = state.api_key.lock().unwrap().clone();

    if key != current_key {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(next.run(req).await)
}

/// Axum middleware for human admin routes.
///
/// Browser sessions use `X-Admin-Session`. The service API key remains accepted
/// for admin endpoints so automation can still inspect and rotate operational
/// state without a browser login.
pub async fn require_admin_session(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let service_key = req
        .headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let current_key = state.api_key.lock().unwrap().clone();
    if !current_key.is_empty() && service_key == current_key {
        return Ok(next.run(req).await);
    }

    let session = req
        .headers()
        .get("x-admin-session")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if session.is_empty() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let authorized = state.admin_sessions.lock().unwrap().contains(session);
    if !authorized {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(next.run(req).await)
}
