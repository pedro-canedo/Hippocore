# Próxima Feature

## Nome

**Phase 11 — HTTP Server** (`crates/hippocore-server`)

## Por que importa

Hippocore DB é atualmente utilizável apenas como biblioteca Rust embedded ou via
CLI. A Phase 11 o torna acessível a qualquer linguagem ou runtime: um servidor
HTTP leve expõe toda a API core via REST/JSON. Esta é a base para SDKs em
Python, TypeScript e outras linguagens.

## Arquitetura

Novo crate de workspace `crates/hippocore-server`:
- **Framework**: `axum` 0.7 (async puro, baseado em tokio, sem deps C).
- **Runtime**: `tokio` 1.
- **Auth**: header `X-Api-Key` verificado contra chave carregada de env var
  (`HIPPOCORE_API_KEY`) ou flag CLI. Requisições não autenticadas retornam 401.
- **State**: `Arc<Mutex<Hippocore>>` compartilhado entre handlers.
- **Config**: data dir e porta de env vars ou flags CLI.

## Endpoints (conjunto MVP)

| Método | Path | Descrição |
|---|---|---|
| `GET` | `/health` | Liveness check (sem auth) |
| `GET` | `/stats` | `DatabaseStats` como JSON |
| `POST` | `/tenants` | `create_tenant` |
| `POST` | `/tenants/:tid/collections` | `create_collection` |
| `POST` | `/tenants/:tid/memories` | `remember` |
| `POST` | `/tenants/:tid/recall` | `recall` |
| `POST` | `/tenants/:tid/context` | `build_context` |
| `POST` | `/tenants/:tid/documents` | `store_document` |
| `DELETE` | `/tenants/:tid/memories/:id` | `forget` |
| `POST` | `/tenants/:tid/graph/traverse` | `traverse_graph` |

## Integração CLI

Adicionar `hippocore serve [--port 8080] [--data-dir ./data] [--api-key KEY]`
ao binário CLI existente via `hippocore-cli`.

## Arquivos

- `crates/hippocore-server/` — novo crate (lib + binário opcional).
- `Cargo.toml` — adicionar `crates/hippocore-server` aos members do workspace.
- `crates/hippocore-cli/` — adicionar subcomando `serve`.
- `docs/en/SERVER.md` e `docs/pt-br/SERVER.md`.
- `docs/en/STATUS.md` e `docs/pt-br/STATUS.md`.
- ROADMAP.md Phase 11 ✅.

## Critérios de aceite

- Servidor inicia, `/health` retorna 200 sem auth.
- Todos os endpoints listados retornam status HTTP corretos.
- Requisições sem `X-Api-Key` retornam 401.
- Pelo menos 6 testes de integração.
- Todos os quality gates passam.

## Fora de escopo

- gRPC / WebSocket.
- TLS (tratado externamente por reverse proxy).
- Auth de produção (OAuth, RBAC).
- Clustering / estado distribuído.
