use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// A typed error that converts to an HTTP error response.
pub struct ServerError(pub StatusCode, pub String);

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({"error": self.1}))).into_response()
    }
}

impl From<hippocore::HippocoreError> for ServerError {
    fn from(e: hippocore::HippocoreError) -> Self {
        ServerError(StatusCode::BAD_REQUEST, e.to_string())
    }
}
