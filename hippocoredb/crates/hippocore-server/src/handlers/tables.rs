use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{types::ServerError, AppState};

#[derive(Deserialize)]
pub struct TableQuery {
    pub collection: String,
}

#[derive(Serialize)]
pub struct TableInfo {
    pub name: String,
    pub record_count: usize,
}

pub async fn list_tables(
    State(state): State<AppState>,
    Path(tid): Path<String>,
    Query(query): Query<TableQuery>,
) -> Result<Json<Vec<TableInfo>>, ServerError> {
    let db = state.db.lock().unwrap();
    let records = db.list_records(&tid, Some(&query.collection), None);
    let mut counts: HashMap<String, usize> = HashMap::new();
    for r in &records {
        *counts.entry(r.table.clone()).or_insert(0) += 1;
    }
    let mut tables: Vec<TableInfo> = counts
        .into_iter()
        .map(|(name, record_count)| TableInfo { name, record_count })
        .collect();
    tables.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(Json(tables))
}

pub async fn delete_table(
    State(state): State<AppState>,
    Path((tid, table)): Path<(String, String)>,
    Query(query): Query<TableQuery>,
) -> Result<StatusCode, ServerError> {
    let mut db = state.db.lock().unwrap();
    let records = db.list_records(&tid, Some(&query.collection), Some(&table));
    for r in records {
        db.delete_record(&tid, &query.collection, &table, &r.id)
            .map_err(ServerError::from)?;
    }
    Ok(StatusCode::NO_CONTENT)
}
