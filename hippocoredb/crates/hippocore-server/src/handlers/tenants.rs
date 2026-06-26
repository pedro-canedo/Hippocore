use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::{types::ServerError, AppState};

#[derive(Deserialize)]
pub struct CreateTenantBody {
    pub id: String,
    pub name: String,
}

#[derive(Serialize)]
pub struct TenantResponse {
    pub id: String,
    pub name: String,
}

pub async fn create_tenant(
    State(state): State<AppState>,
    Json(body): Json<CreateTenantBody>,
) -> Result<(StatusCode, Json<TenantResponse>), ServerError> {
    let mut db = state.db.lock().unwrap();
    let tenant = db
        .create_tenant(&body.id, &body.name)
        .map_err(ServerError::from)?;
    Ok((
        StatusCode::CREATED,
        Json(TenantResponse {
            id: tenant.id,
            name: tenant.name,
        }),
    ))
}
