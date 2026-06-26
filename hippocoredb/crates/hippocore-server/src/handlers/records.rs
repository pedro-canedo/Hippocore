use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use hippocore::PutRecordRequest;

use crate::{types::ServerError, AppState};

#[derive(Deserialize)]
pub struct PutRecordBody {
    pub collection: String,
    pub table: String,
    pub id: Option<String>,
    pub payload: Value,
}

#[derive(Serialize)]
pub struct RecordResponse {
    pub id: String,
    pub tenant_id: String,
    pub collection: String,
    pub table: String,
}

pub async fn put_record(
    State(state): State<AppState>,
    Path(tid): Path<String>,
    Json(body): Json<PutRecordBody>,
) -> Result<(StatusCode, Json<RecordResponse>), ServerError> {
    let mut req = PutRecordRequest::new(&tid, &body.collection, &body.table, body.payload);
    req.id = body.id;

    let mut db = state.db.lock().unwrap();
    let record = db.put_record(req).map_err(ServerError::from)?;
    Ok((
        StatusCode::CREATED,
        Json(RecordResponse {
            id: record.id,
            tenant_id: record.tenant_id,
            collection: record.collection,
            table: record.table,
        }),
    ))
}
