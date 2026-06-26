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
