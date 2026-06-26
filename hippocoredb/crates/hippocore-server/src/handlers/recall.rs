use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};

use hippocore::RecallRequest;

use crate::{types::ServerError, AppState};

#[derive(Deserialize)]
pub struct RecallBody {
    pub query: String,
    pub collection: Option<String>,
    pub top_k: Option<usize>,
    pub min_score: Option<f32>,
    pub dedup_chunks: Option<bool>,
    pub mmr: Option<bool>,
    pub mmr_lambda: Option<f32>,
}

#[derive(Serialize)]
pub struct RecallResultItem {
    pub id: String,
    pub document_id: Option<String>,
    pub score: f32,
    pub text: String,
    pub kind: String,
}

pub async fn recall(
    State(state): State<AppState>,
    Path(tid): Path<String>,
    Json(body): Json<RecallBody>,
) -> Result<Json<Vec<RecallResultItem>>, ServerError> {
    let mut req = RecallRequest::new(&tid, &body.query);
    req.collection = body.collection;
    if let Some(k) = body.top_k {
        req.top_k = k;
    }
    req.min_score = body.min_score;
    req.dedup_chunks = body.dedup_chunks.unwrap_or(false);
    req.mmr = body.mmr.unwrap_or(false);
    if let Some(lam) = body.mmr_lambda {
        req.mmr_lambda = lam;
    }

    let db = state.db.lock().unwrap();
    let results = db.recall(req).map_err(ServerError::from)?;
    let items = results
        .into_iter()
        .map(|r| RecallResultItem {
            id: r.id,
            document_id: r.document_id,
            score: r.score,
            text: r.text,
            kind: format!("{:?}", r.kind),
        })
        .collect();
    Ok(Json(items))
}
