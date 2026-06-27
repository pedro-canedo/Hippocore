use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use serde_json::Value;

use hippocore::{PutRecordRequest, StoreDocumentRequest};

use crate::{types::ServerError, AppState};

#[derive(Serialize)]
pub struct UploadResponse {
    /// Document id when the file was stored as a document.
    pub file_id: Option<String>,
    /// Storage kind: `"document"` or `"records"`.
    pub kind: String,
    /// Chunk count for documents, row count for CSV/JSON records.
    pub count: usize,
    /// Filename as received.
    pub name: String,
}

pub async fn upload_file(
    State(state): State<AppState>,
    Path(tid): Path<String>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<UploadResponse>), ServerError> {
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut file_name = String::from("upload");
    let mut collection = String::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ServerError(StatusCode::BAD_REQUEST, e.to_string()))?
    {
        let field_name = field.name().unwrap_or("").to_string();
        // Capture the filename from Content-Disposition before consuming the field.
        let content_disp_name = field.file_name().map(str::to_string).unwrap_or_default();

        let data = field
            .bytes()
            .await
            .map_err(|e| ServerError(StatusCode::BAD_REQUEST, e.to_string()))?;

        match field_name.as_str() {
            "file" => {
                if !content_disp_name.is_empty() {
                    file_name = content_disp_name;
                }
                file_bytes = Some(data.to_vec());
            }
            "collection" => {
                collection = String::from_utf8_lossy(&data).trim().to_string();
            }
            "filename" => {
                let name = String::from_utf8_lossy(&data).trim().to_string();
                if !name.is_empty() {
                    file_name = name;
                }
            }
            _ => {}
        }
    }

    let bytes = file_bytes
        .ok_or_else(|| ServerError(StatusCode::BAD_REQUEST, "missing 'file' field".into()))?;

    if collection.is_empty() {
        return Err(ServerError(
            StatusCode::BAD_REQUEST,
            "missing 'collection' field".into(),
        ));
    }

    let ext = std::path::Path::new(&file_name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let stem = std::path::Path::new(&file_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("upload")
        .to_string();

    let mut db = state.db.lock().unwrap();

    match ext.as_str() {
        "pdf" => {
            let req = StoreDocumentRequest::from_pdf(&tid, &collection, bytes);
            let doc = db.store_document(req).map_err(ServerError::from)?;
            let chunk_count = db.get_document_chunks(&tid, &collection, &doc.id).len();
            Ok((
                StatusCode::CREATED,
                Json(UploadResponse {
                    file_id: Some(doc.id),
                    kind: "document".into(),
                    count: chunk_count,
                    name: file_name,
                }),
            ))
        }

        "txt" | "md" => {
            let text = String::from_utf8(bytes).map_err(|_| {
                ServerError(StatusCode::BAD_REQUEST, "file is not valid UTF-8".into())
            })?;
            let req = StoreDocumentRequest::new(&tid, &collection, text);
            let doc = db.store_document(req).map_err(ServerError::from)?;
            let chunk_count = db.get_document_chunks(&tid, &collection, &doc.id).len();
            Ok((
                StatusCode::CREATED,
                Json(UploadResponse {
                    file_id: Some(doc.id),
                    kind: "document".into(),
                    count: chunk_count,
                    name: file_name,
                }),
            ))
        }

        "csv" => {
            let text = String::from_utf8(bytes).map_err(|_| {
                ServerError(StatusCode::BAD_REQUEST, "CSV is not valid UTF-8".into())
            })?;
            let rows = parse_csv(&text)?;
            let count = rows.len();
            let table = stem;
            for row in rows {
                let req = PutRecordRequest::new(&tid, &collection, &table, row);
                db.put_record(req).map_err(ServerError::from)?;
            }
            Ok((
                StatusCode::CREATED,
                Json(UploadResponse {
                    file_id: None,
                    kind: "records".into(),
                    count,
                    name: file_name,
                }),
            ))
        }

        "json" => {
            let text = String::from_utf8(bytes).map_err(|_| {
                ServerError(StatusCode::BAD_REQUEST, "JSON is not valid UTF-8".into())
            })?;
            let value: Value = serde_json::from_str(&text)
                .map_err(|e| ServerError(StatusCode::BAD_REQUEST, format!("invalid JSON: {e}")))?;

            match value {
                Value::Array(items) => {
                    let table = stem;
                    let count = items.len();
                    for item in items {
                        if !item.is_object() {
                            return Err(ServerError(
                                StatusCode::BAD_REQUEST,
                                "JSON array must contain objects to import as records".into(),
                            ));
                        }
                        let req = PutRecordRequest::new(&tid, &collection, &table, item);
                        db.put_record(req).map_err(ServerError::from)?;
                    }
                    Ok((
                        StatusCode::CREATED,
                        Json(UploadResponse {
                            file_id: None,
                            kind: "records".into(),
                            count,
                            name: file_name,
                        }),
                    ))
                }
                _ => {
                    let req = StoreDocumentRequest::new(&tid, &collection, text);
                    let doc = db.store_document(req).map_err(ServerError::from)?;
                    let chunk_count = db.get_document_chunks(&tid, &collection, &doc.id).len();
                    Ok((
                        StatusCode::CREATED,
                        Json(UploadResponse {
                            file_id: Some(doc.id),
                            kind: "document".into(),
                            count: chunk_count,
                            name: file_name,
                        }),
                    ))
                }
            }
        }

        other => Err(ServerError(
            StatusCode::UNPROCESSABLE_ENTITY,
            format!("unsupported file type '.{other}'; accepted: .pdf .txt .md .csv .json"),
        )),
    }
}

/// Minimal CSV parser — splits on commas, trims whitespace.
/// Handles only simple CSV (no quoted commas, no escaped quotes).
fn parse_csv(text: &str) -> Result<Vec<Value>, ServerError> {
    let mut lines = text.lines().filter(|l| !l.trim().is_empty());
    let headers: Vec<String> = match lines.next() {
        Some(h) => h.split(',').map(|s| s.trim().to_string()).collect(),
        None => {
            return Err(ServerError(
                StatusCode::BAD_REQUEST,
                "CSV has no header row".into(),
            ))
        }
    };
    if headers.is_empty() {
        return Err(ServerError(
            StatusCode::BAD_REQUEST,
            "CSV header row is empty".into(),
        ));
    }
    let rows: Vec<Value> = lines
        .map(|line| {
            let vals: Vec<&str> = line.split(',').collect();
            let obj: serde_json::Map<String, Value> = headers
                .iter()
                .enumerate()
                .map(|(i, h)| {
                    let v = vals.get(i).copied().unwrap_or("").trim().to_string();
                    (h.clone(), Value::String(v))
                })
                .collect();
            Value::Object(obj)
        })
        .collect();
    Ok(rows)
}
