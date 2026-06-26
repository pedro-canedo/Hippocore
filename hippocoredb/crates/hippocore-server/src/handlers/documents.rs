use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use hippocore::StoreDocumentRequest;

use crate::{types::ServerError, AppState};

#[derive(Deserialize)]
pub struct StoreDocumentBody {
    pub collection: String,
    pub text: String,
    pub id: Option<String>,
}

#[derive(Serialize)]
pub struct DocumentResponse {
    pub id: String,
    pub tenant_id: String,
    pub collection: String,
    pub chunk_count: usize,
}

pub async fn store_document(
    State(state): State<AppState>,
    Path(tid): Path<String>,
    Json(body): Json<StoreDocumentBody>,
) -> Result<(StatusCode, Json<DocumentResponse>), ServerError> {
    let mut req = StoreDocumentRequest::new(&tid, &body.collection, &body.text);
    req.id = body.id;

    let mut db = state.db.lock().unwrap();
    let doc = db.store_document(req).map_err(ServerError::from)?;
    let chunk_count = db.get_document_chunks(&tid, &doc.collection, &doc.id).len();
    Ok((
        StatusCode::CREATED,
        Json(DocumentResponse {
            id: doc.id,
            tenant_id: doc.tenant_id,
            collection: doc.collection,
            chunk_count,
        }),
    ))
}
