# Next Feature

## Feature name

**Phase 11 — HTTP Server** (`crates/hippocore-server`)

## Why it matters

Hippocore DB is currently usable only as an embedded Rust library or via the
CLI. Phase 11 makes it accessible to any language or runtime: a lightweight
HTTP server exposes the full core API over REST/JSON. This is the foundation
for SDKs in Python, TypeScript, and other languages.

## Architecture

A new workspace crate `crates/hippocore-server`:
- **Framework**: `axum` 0.7 (pure async, tokio-based, no C deps).
- **Runtime**: `tokio` 1.
- **Auth**: `X-Api-Key` header checked against a key loaded from an env var
  (`HIPPOCORE_API_KEY`) or CLI flag. Unauthenticated requests return 401.
- **State**: `Arc<Mutex<Hippocore>>` shared across handlers.
- **Config**: data dir and port from env vars or CLI flags.

## Endpoints (MVP set)

| Method | Path | Description |
|---|---|---|
| `GET` | `/health` | Liveness check (no auth required) |
| `GET` | `/stats` | `DatabaseStats` as JSON |
| `POST` | `/tenants` | `create_tenant` |
| `POST` | `/tenants/:tid/collections` | `create_collection` |
| `POST` | `/tenants/:tid/memories` | `remember` |
| `POST` | `/tenants/:tid/recall` | `recall` |
| `POST` | `/tenants/:tid/context` | `build_context` |
| `POST` | `/tenants/:tid/documents` | `store_document` |
| `DELETE` | `/tenants/:tid/memories/:id` | `forget` |
| `POST` | `/tenants/:tid/graph/traverse` | `traverse_graph` |

## CLI integration

Add `hippocore serve [--port 8080] [--data-dir ./data] [--api-key KEY]` to the
existing CLI binary via `hippocore-cli`.

## Files

- `crates/hippocore-server/` — new crate (lib + optional binary).
- `Cargo.toml` — add `crates/hippocore-server` to workspace members.
- `crates/hippocore-cli/` — add `serve` subcommand.
- `docs/en/SERVER.md` and `docs/pt-br/SERVER.md`.
- `docs/en/STATUS.md` and `docs/pt-br/STATUS.md`.
- ROADMAP.md Phase 11 ✅.

## Acceptance criteria

- Server starts, `/health` returns 200 without auth.
- All listed endpoints return correct HTTP status codes.
- Requests without `X-Api-Key` return 401.
- At least 6 integration tests using `reqwest` or `axum::test`.
- All quality gates pass.

## Out of scope

- gRPC / WebSocket.
- TLS (handled externally by a reverse proxy).
- Production-grade auth (OAuth, RBAC).
- Clustering / distributed state.
