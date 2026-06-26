# HTTP Server — Phase 11

## Overview

`hippocore-server` is a lightweight HTTP/JSON server built on [axum](https://github.com/tokio-rs/axum) 0.8
that exposes the full Hippocore DB core library over REST. It enables
integration from any language or runtime without linking Rust code.

## Starting the server

```bash
# Simplest form — data dir defaults to ./hippocore-data
HIPPOCORE_API_KEY=mysecret hippocore serve --port 8080

# All flags explicit
hippocore serve --port 8080 --db ./mydata --api-key mysecret
```

The same process also serves a browser-based admin console at `/admin`.
It uses the API key for browser-side requests and can rotate the active key
after login.

## Authentication

Every protected endpoint requires an `X-Api-Key` request header matching the
configured API key. Requests without the header or with a wrong key receive
`401 Unauthorized`.

The key is set via `--api-key <key>` (CLI flag) or the `HIPPOCORE_API_KEY`
environment variable. The server refuses to start if no key is provided.

## Endpoints

### Public (no auth)

| Method | Path | Description |
|---|---|---|
| `GET` | `/health` | Returns `200 OK` — liveness probe |
| `GET` | `/admin` | Browser-based admin console |

### Protected (requires `X-Api-Key`)

| Method | Path | Description |
|---|---|---|
| `GET` | `/admin/bootstrap` | Stats, tenants, and config for the admin console |
| `GET` | `/admin/config` | Basic server configuration summary |
| `POST` | `/admin/api-key/rotate` | Rotate the active API key |
| `GET` | `/admin/tenants` | List tenants |
| `GET` | `/admin/collections` | List collections |
| `GET` | `/admin/memories` | List memories |
| `GET` | `/admin/documents` | List documents |
| `GET` | `/admin/records` | List records |
| `GET` | `/admin/files` | List files |
| `GET` | `/admin/graph-edges` | List graph edges |
| `POST` | `/admin/sql` | Execute MVP SQL-like record query (`{"tenant_id":"acme","sql":"select * from systems limit 5"}`) |
| `POST` | `/admin/query-records` | Legacy restricted records query response |
| `GET` | `/stats` | `DatabaseStats` as JSON |
| `POST` | `/tenants` | Create a tenant (`{"id":"t1","name":"T1"}`) |
| `POST` | `/tenants/:tid/collections` | Create a collection (`{"name":"col","description":"..."}`) |
| `POST` | `/tenants/:tid/memories` | Store a memory (`{"collection":"col","text":"...","memory_type":"semantic"}`) |
| `DELETE` | `/tenants/:tid/memories/:id` | Delete (forget) a memory |
| `POST` | `/tenants/:tid/recall` | Hybrid recall (`{"query":"...","collection":"col","top_k":10}`) |
| `POST` | `/tenants/:tid/context` | Build LLM context (`{"query":"...","max_tokens":1024,"include_related":true}`) |
| `POST` | `/tenants/:tid/documents` | Store a document (`{"collection":"col","text":"...","id":null}`) |
| `POST` | `/tenants/:tid/graph/traverse` | BFS graph traversal (see below) |

### Graph traversal request body

```json
{
  "seeds": [{"kind": "Memory", "id": "mem-123"}],
  "max_hops": 2,
  "max_nodes": 50,
  "relation_filter": null
}
```

`kind` must be one of `"Memory"`, `"DocumentChunk"`, or `"Record"`.

## Architecture

- **Crate**: `crates/hippocore-server` (library + optional binary).
- **State**: `AppState { db: Arc<Mutex<Hippocore>>, api_key: String }`.
- **Auth middleware**: `axum::middleware::from_fn_with_state`.
- **Error type**: `ServerError(StatusCode, String)` implements `IntoResponse`.
- **Testing**: use `build_router(state)` with `tower::ServiceExt::oneshot` for
  in-process tests (no TCP port required).

## Embedding in your own binary

```rust
use hippocore::{Config, Hippocore};
use hippocore_server::{AppState, ServerConfig, serve};

#[tokio::main]
async fn main() {
    let cfg = ServerConfig {
        data_dir: "./data".into(),
        port: 8080,
        api_key: "mysecret".into(),
        admin_username: "admin".into(),
        admin_password: "change-me".into(),
    };
    serve(cfg).await.unwrap();
}
```
