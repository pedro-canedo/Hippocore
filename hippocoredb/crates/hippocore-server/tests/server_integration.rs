use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tempfile::TempDir;
use tower::ServiceExt;

use hippocore::{Config, Hippocore};
use hippocore_server::{build_router, AppState};

fn test_app(dir: &TempDir) -> axum::Router {
    let db = Hippocore::open(Config::new(dir.path())).unwrap();
    let state = AppState::new(db, "test-key");
    build_router(state)
}

fn json_request(method: &str, uri: &str, body: Value, api_key: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");
    if let Some(key) = api_key {
        builder = builder.header("x-api-key", key);
    }
    builder
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap()
}

// --- health ---

#[tokio::test]
async fn health_returns_200_without_auth() {
    let dir = TempDir::new().unwrap();
    let app = test_app(&dir);
    let req = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn admin_page_is_public() {
    let dir = TempDir::new().unwrap();
    let app = test_app(&dir);
    let req = Request::builder()
        .uri("/admin")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("Hippocore Studio"));
}

// --- auth ---

#[tokio::test]
async fn missing_api_key_returns_401() {
    let dir = TempDir::new().unwrap();
    let app = test_app(&dir);
    let req = Request::builder()
        .method("GET")
        .uri("/stats")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn wrong_api_key_returns_401() {
    let dir = TempDir::new().unwrap();
    let app = test_app(&dir);
    let req = Request::builder()
        .method("GET")
        .uri("/stats")
        .header("x-api-key", "wrong")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn admin_bootstrap_requires_auth() {
    let dir = TempDir::new().unwrap();
    let app = test_app(&dir);
    let req = Request::builder()
        .method("GET")
        .uri("/admin/bootstrap")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn rotating_api_key_updates_auth() {
    let dir = TempDir::new().unwrap();
    let app = test_app(&dir);

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/admin/api-key/rotate",
            json!({"length": 16}),
            Some("test-key"),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let val: Value = serde_json::from_slice(&body).unwrap();
    let new_key = val["api_key"].as_str().unwrap().to_string();
    assert!(new_key.len() >= 32);

    let old = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/stats")
                .header("x-api-key", "test-key")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(old.status(), StatusCode::UNAUTHORIZED);

    let new = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/stats")
                .header("x-api-key", &new_key)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(new.status(), StatusCode::OK);
}

// --- stats ---

#[tokio::test]
async fn stats_returns_json_with_correct_key() {
    let dir = TempDir::new().unwrap();
    let app = test_app(&dir);
    let req = Request::builder()
        .method("GET")
        .uri("/stats")
        .header("x-api-key", "test-key")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let val: Value = serde_json::from_slice(&body).unwrap();
    assert!(val.get("tenants").is_some());
    assert!(val.get("memories").is_some());
}

// --- tenant + collection + memory + recall ---

#[tokio::test]
async fn full_flow_tenant_collection_memory_recall() {
    let dir = TempDir::new().unwrap();
    let app = test_app(&dir);

    // create tenant
    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/tenants",
            json!({"id": "t1", "name": "Tenant 1"}),
            Some("test-key"),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let tenant: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(tenant["id"], "t1");

    // create collection
    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/tenants/t1/collections",
            json!({"name": "notes", "description": "test notes"}),
            Some("test-key"),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // store memory
    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/tenants/t1/memories",
            json!({"collection": "notes", "text": "axum is ergonomic"}),
            Some("test-key"),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let mem: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(mem["tenant_id"], "t1");

    // recall
    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/tenants/t1/recall",
            json!({"query": "ergonomic"}),
            Some("test-key"),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let results: Value = serde_json::from_slice(&body).unwrap();
    let arr = results.as_array().unwrap();
    assert!(!arr.is_empty());
    assert!(arr[0]["text"].as_str().unwrap().contains("axum"));
}

// --- document ---

#[tokio::test]
async fn store_document_returns_chunk_count() {
    let dir = TempDir::new().unwrap();
    let app = test_app(&dir);

    // setup
    for (path, body) in [
        ("/tenants", json!({"id": "t2", "name": "T2"})),
        ("/tenants/t2/collections", json!({"name": "docs"})),
    ] {
        let resp = app
            .clone()
            .oneshot(json_request("POST", path, body, Some("test-key")))
            .await
            .unwrap();
        assert!(resp.status().is_success());
    }

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/tenants/t2/documents",
            json!({"collection": "docs", "text": "Hippocore DB is an AI-native memory database."}),
            Some("test-key"),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let doc: Value = serde_json::from_slice(&body).unwrap();
    assert!(doc["chunk_count"].as_u64().unwrap() >= 1);
}

// --- build_context ---

#[tokio::test]
async fn build_context_returns_text() {
    let dir = TempDir::new().unwrap();
    let app = test_app(&dir);

    for (path, body) in [
        ("/tenants", json!({"id": "t3", "name": "T3"})),
        ("/tenants/t3/collections", json!({"name": "ctx"})),
    ] {
        let resp = app
            .clone()
            .oneshot(json_request("POST", path, body, Some("test-key")))
            .await
            .unwrap();
        assert!(resp.status().is_success());
    }

    app.clone()
        .oneshot(json_request(
            "POST",
            "/tenants/t3/memories",
            json!({"collection": "ctx", "text": "context building works"}),
            Some("test-key"),
        ))
        .await
        .unwrap();

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/tenants/t3/context",
            json!({"query": "context building", "max_tokens": 512}),
            Some("test-key"),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let ctx: Value = serde_json::from_slice(&body).unwrap();
    assert!(ctx.get("text").is_some());
    assert!(ctx["token_count"].as_u64().unwrap() > 0);
}
