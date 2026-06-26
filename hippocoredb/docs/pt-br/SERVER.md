# Servidor HTTP — Phase 11

## Visão geral

`hippocore-server` é um servidor HTTP/JSON leve construído com [axum](https://github.com/tokio-rs/axum) 0.8
que expõe toda a biblioteca core do Hippocore DB via REST. Permite integração
de qualquer linguagem ou runtime sem precisar compilar Rust.

## Iniciando o servidor

```bash
# Forma mais simples — data dir padrão ./hippocore-data
HIPPOCORE_API_KEY=minhachave hippocore serve --port 8080

# Todos os flags explícitos
hippocore serve --port 8080 --db ./meusdados --api-key minhachave
```

O mesmo processo também serve um console administrativo web em `/admin`.
Ele usa a API key nas requisições do navegador e pode rotacionar a chave ativa
após o login.

## Autenticação

Todo endpoint protegido exige o header `X-Api-Key` com valor igual à chave
configurada. Requisições sem o header ou com chave errada recebem `401 Unauthorized`.

A chave é definida via `--api-key <key>` (flag CLI) ou variável de ambiente
`HIPPOCORE_API_KEY`. O servidor recusa iniciar sem chave.

## Endpoints

### Públicos (sem auth)

| Método | Path | Descrição |
|---|---|---|
| `GET` | `/health` | Retorna `200 OK` — liveness probe |
| `GET` | `/admin` | Console administrativo web |

### Protegidos (requerem `X-Api-Key`)

| Método | Path | Descrição |
|---|---|---|
| `GET` | `/admin/bootstrap` | Stats, tenants e config para o console |
| `GET` | `/admin/config` | Resumo básico da configuração do servidor |
| `POST` | `/admin/api-key/rotate` | Rotacionar a API key ativa |
| `GET` | `/admin/tenants` | Listar tenants |
| `GET` | `/admin/collections` | Listar collections |
| `GET` | `/admin/memories` | Listar memórias |
| `GET` | `/admin/documents` | Listar documentos |
| `GET` | `/admin/records` | Listar records |
| `GET` | `/admin/files` | Listar arquivos |
| `GET` | `/admin/graph-edges` | Listar arestas de grafo |
| `POST` | `/admin/sql` | Executar query de records estilo SQL MVP (`{"tenant_id":"acme","sql":"select * from systems limit 5"}`) |
| `POST` | `/admin/query-records` | Resposta legada de query restrita de records |
| `POST` | `/admin/tenants/:tid/records` | Criar record JSON-first (`{"collection":"col","table":"data","payload":{...}}`) |
| `POST` | `/admin/tenants/:tid/memories` | Armazenar memória (`{"collection":"col","text":"..."}`) |
| `POST` | `/admin/tenants/:tid/documents` | Armazenar documento (`{"collection":"col","text":"..."}`) |
| `GET` | `/stats` | `DatabaseStats` como JSON |
| `POST` | `/tenants` | Criar tenant (`{"id":"t1","name":"T1"}`) |
| `POST` | `/tenants/:tid/collections` | Criar collection (`{"name":"col","description":"..."}`) |
| `POST` | `/tenants/:tid/memories` | Armazenar memória (`{"collection":"col","text":"...","memory_type":"semantic"}`) |
| `DELETE` | `/tenants/:tid/memories/:id` | Deletar (esquecer) uma memória |
| `POST` | `/tenants/:tid/recall` | Recall híbrido (`{"query":"...","collection":"col","top_k":10}`) |
| `POST` | `/tenants/:tid/context` | Montar contexto LLM (`{"query":"...","max_tokens":1024,"include_related":true}`) |
| `POST` | `/tenants/:tid/documents` | Armazenar documento (`{"collection":"col","text":"...","id":null}`) |
| `POST` | `/tenants/:tid/graph/traverse` | Travessia BFS do grafo (ver abaixo) |

### Corpo da requisição de travessia de grafo

```json
{
  "seeds": [{"kind": "Memory", "id": "mem-123"}],
  "max_hops": 2,
  "max_nodes": 50,
  "relation_filter": null
}
```

`kind` deve ser `"Memory"`, `"DocumentChunk"` ou `"Record"`.

## Arquitetura

- **Crate**: `crates/hippocore-server` (biblioteca + binário opcional).
- **State**: `AppState { db: Arc<Mutex<Hippocore>>, api_key: Arc<Mutex<String>> }`.
- **Middleware de auth**: `axum::middleware::from_fn_with_state`.
- **Tipo de erro**: `ServerError(StatusCode, String)` implementa `IntoResponse`.
- **Testes**: usar `build_router(state)` com `tower::ServiceExt::oneshot` para
  testes em processo (sem porta TCP necessária).

## Embutindo no seu próprio binário

```rust
use hippocore::{Config, Hippocore};
use hippocore_server::{AppState, ServerConfig, serve};

#[tokio::main]
async fn main() {
    let cfg = ServerConfig {
        data_dir: "./data".into(),
        port: 8080,
        api_key: "minhachave".into(),
        admin_username: "admin".into(),
        admin_password: "troque-esta-senha".into(),
    };
    serve(cfg).await.unwrap();
}
```
