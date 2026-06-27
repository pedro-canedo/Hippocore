use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tempfile::TempDir;
use tower::ServiceExt;

use hippocore::{Config, Hippocore, PutRecordRequest};
use hippocore_server::{build_router, AppState};

fn test_app(dir: &TempDir) -> axum::Router {
    let db = Hippocore::open(Config::new(dir.path())).unwrap();
    let state = AppState::with_admin_credentials(db, "test-key", "admin", "secret");
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

fn admin_json_request(method: &str, uri: &str, body: Value, session: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .header("x-admin-session", session)
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap()
}

async fn login(app: axum::Router) -> String {
    let resp = app
        .oneshot(json_request(
            "POST",
            "/admin/login",
            json!({"username": "admin", "password": "secret"}),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let val: Value = serde_json::from_slice(&body).unwrap();
    val["session"].as_str().unwrap().to_string()
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
    assert!(html.contains("Hippocore Control Plane"));
    assert!(html.contains("/admin/app.js"));
}

#[tokio::test]
async fn admin_assets_are_public() {
    let dir = TempDir::new().unwrap();
    let app = test_app(&dir);
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/admin/styles.css")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let css = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let css = String::from_utf8(css.to_vec()).unwrap();
    assert!(css.contains(".sidebar"));
    assert!(css.contains(".topbar"));

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/admin/app.js")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let js = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let js = String::from_utf8(js.to_vec()).unwrap();
    for marker in [
        "Dashboard",
        "Data Explorer",
        "SQL Editor",
        "Ingestion & Recall",
        "API Reference",
        "Observability",
        "function AppShell",
        "function DataTable",
        "function JsonViewer",
        "function DetailDrawer",
        "function RecallCards",
        "function ContextResult",
        "knownCollections",
        "matched_terms",
        "/admin/sql",
        "dataExplorer: 'Data Explorer'",
        "dataExplorer: 'Explorador de Dados'",
        "function captureFormState",
        "function messageText",
        "t(GROUP_LABELS[group])",
    ] {
        assert!(js.contains(marker), "missing admin asset marker {marker}");
    }

    let runtime = js.split("const state =").nth(1).unwrap();
    for hardcoded_copy in [
        "PageHeader('Dashboard'",
        "<strong>Create a tenant</strong>",
        "<h3>Quick actions</h3>",
        "PageHeader('Data Explorer'",
        "PageHeader('SQL Editor'",
        "PageHeader('Collections'",
        "PageHeader('Files'",
        "PageHeader('Graph'",
        "PageHeader('API Reference'",
        "PageHeader('Tenants'",
        "PageHeader('Integrations'",
        "PageHeader('System Prompts'",
        "PageHeader('Observability'",
        "PageHeader('Settings'",
        "showToast('Copied to clipboard')",
        "throw new Error('select a tenant first')",
    ] {
        assert!(
            !runtime.contains(hardcoded_copy),
            "hardcoded admin copy returned: {hardcoded_copy}"
        );
    }
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
async fn admin_login_session_can_bootstrap() {
    let dir = TempDir::new().unwrap();
    let app = test_app(&dir);
    let session = login(app.clone()).await;
    let resp = app
        .oneshot(admin_json_request(
            "GET",
            "/admin/bootstrap",
            json!({}),
            &session,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
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
    assert_eq!(arr[0]["matched_terms"], json!(["ergonomic"]));
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

#[tokio::test]
async fn admin_restricted_records_query_returns_tenant_scoped_records() {
    let dir = TempDir::new().unwrap();
    let mut db = Hippocore::open(Config::new(dir.path())).unwrap();
    db.create_tenant("acme", "Acme").unwrap();
    db.create_collection("acme", "data", "").unwrap();
    let mut req = PutRecordRequest::new(
        "acme",
        "data",
        "systems",
        json!({"engine": "postgresql", "language": "python"}),
    );
    req.id = Some("pg-python".to_string());
    db.put_record(req).unwrap();

    let state = AppState::with_admin_credentials(db, "test-key", "admin", "secret");
    let app = build_router(state);
    let session = login(app.clone()).await;

    let resp = app
        .oneshot(admin_json_request(
            "POST",
            "/admin/query-records",
            json!({
                "tenant_id": "acme",
                "sql": "select * from records where payload.engine = 'postgresql' limit 5"
            }),
            &session,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let rows: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(rows.as_array().unwrap().len(), 1);
    assert_eq!(rows[0]["id"], "pg-python");
}

#[tokio::test]
async fn admin_sql_endpoint_returns_result_shape() {
    let dir = TempDir::new().unwrap();
    let mut db = Hippocore::open(Config::new(dir.path())).unwrap();
    db.create_tenant("acme", "Acme").unwrap();
    db.create_collection("acme", "data", "").unwrap();
    let mut req = PutRecordRequest::new(
        "acme",
        "data",
        "systems",
        json!({"engine": "postgresql", "language": "python"}),
    );
    req.id = Some("pg-python".to_string());
    db.put_record(req).unwrap();

    let state = AppState::with_admin_credentials(db, "test-key", "admin", "secret");
    let app = build_router(state);
    let session = login(app.clone()).await;

    let resp = app
        .oneshot(admin_json_request(
            "POST",
            "/admin/sql",
            json!({
                "tenant_id": "acme",
                "sql": "select * from systems where engine = 'postgresql' limit 5"
            }),
            &session,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["command"], "select");
    assert_eq!(result["row_count"], 1);
    assert_eq!(result["rows"][0]["id"], "pg-python");
}

#[tokio::test]
async fn admin_llm_provider_registry_masks_secret() {
    let dir = TempDir::new().unwrap();
    let app = test_app(&dir);
    let session = login(app.clone()).await;

    let resp = app
        .clone()
        .oneshot(admin_json_request(
            "POST",
            "/admin/llm-providers",
            json!({
                "id": "openrouter",
                "kind": "openrouter",
                "base_url": "https://openrouter.ai/api/v1",
                "model": "openai/gpt-4.1-mini",
                "api_key": "secret-provider-key",
                "is_default": true
            }),
            &session,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let provider: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(provider["api_key_set"], true);
    assert!(provider.get("api_key").is_none());

    let resp = app
        .oneshot(admin_json_request(
            "GET",
            "/admin/llm-providers",
            json!({}),
            &session,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let providers: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(providers[0]["id"], "openrouter");
    assert!(providers[0].get("api_key").is_none());
}

#[tokio::test]
async fn admin_put_record_creates_json_record() {
    let dir = TempDir::new().unwrap();
    let app = test_app(&dir);
    let session = login(app.clone()).await;

    // Create tenant and collection first.
    let resp = app
        .clone()
        .oneshot(admin_json_request(
            "POST",
            "/admin/tenants",
            json!({"id": "t1", "name": "Test Tenant"}),
            &session,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let resp = app
        .clone()
        .oneshot(admin_json_request(
            "POST",
            "/admin/tenants/t1/collections",
            json!({"name": "col1"}),
            &session,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Store a JSON record.
    let resp = app
        .clone()
        .oneshot(admin_json_request(
            "POST",
            "/admin/tenants/t1/records",
            json!({
                "collection": "col1",
                "table": "systems",
                "payload": {"engine": "postgresql", "version": 16}
            }),
            &session,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let val: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(val["tenant_id"], "t1");
    assert_eq!(val["collection"], "col1");
    assert_eq!(val["table"], "systems");
    assert!(val["id"].as_str().is_some_and(|s| !s.is_empty()));
}

#[tokio::test]
async fn admin_put_record_requires_tenant_isolation() {
    let dir = TempDir::new().unwrap();
    let app = test_app(&dir);
    let session = login(app.clone()).await;

    // Record for non-existent tenant must fail with 4xx.
    let resp = app
        .clone()
        .oneshot(admin_json_request(
            "POST",
            "/admin/tenants/ghost/records",
            json!({
                "collection": "col",
                "table": "data",
                "payload": {"key": "val"}
            }),
            &session,
        ))
        .await
        .unwrap();
    assert!(
        resp.status().is_client_error(),
        "expected 4xx for unknown tenant, got {}",
        resp.status()
    );
}
