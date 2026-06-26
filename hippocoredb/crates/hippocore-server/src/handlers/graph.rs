use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};

use hippocore::{ItemKind, TraverseGraphRequest};

use crate::{types::ServerError, AppState};

#[derive(Deserialize)]
pub struct TraverseBody {
    /// Seed items as `[{"kind": "Memory"|"DocumentChunk"|"Record", "id": "..."}]`.
    pub seeds: Vec<SeedItem>,
    pub max_hops: Option<usize>,
    pub max_nodes: Option<usize>,
    pub relation_filter: Option<String>,
}

#[derive(Deserialize)]
pub struct SeedItem {
    pub kind: String,
    pub id: String,
}

#[derive(Serialize)]
pub struct TraversalNodeResponse {
    pub id: String,
    pub kind: String,
    pub hop: usize,
    pub via_edge_id: String,
}

fn parse_kind(s: &str) -> Result<ItemKind, ServerError> {
    match s {
        "Memory" => Ok(ItemKind::Memory),
        "DocumentChunk" => Ok(ItemKind::DocumentChunk),
        "Record" => Ok(ItemKind::Record),
        other => Err(ServerError(
            axum::http::StatusCode::BAD_REQUEST,
            format!("unknown ItemKind: {other}"),
        )),
    }
}

pub async fn traverse(
    State(state): State<AppState>,
    Path(tid): Path<String>,
    Json(body): Json<TraverseBody>,
) -> Result<Json<Vec<TraversalNodeResponse>>, ServerError> {
    let seed_ids: Result<Vec<(ItemKind, String)>, ServerError> = body
        .seeds
        .into_iter()
        .map(|s| Ok((parse_kind(&s.kind)?, s.id)))
        .collect();

    let mut req = TraverseGraphRequest::new(&tid, seed_ids?);
    if let Some(h) = body.max_hops {
        req.max_hops = h;
    }
    if let Some(n) = body.max_nodes {
        req.max_nodes = n;
    }
    req.relation_filter = body.relation_filter;

    let db = state.db.lock().unwrap();
    let nodes = db.traverse_graph(req);
    let resp = nodes
        .into_iter()
        .map(|n| TraversalNodeResponse {
            id: n.id,
            kind: format!("{:?}", n.kind),
            hop: n.hop,
            via_edge_id: n.via_edge_id,
        })
        .collect();
    Ok(Json(resp))
}
