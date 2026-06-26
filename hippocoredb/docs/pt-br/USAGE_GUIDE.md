# Hippocore DB — Guia Completo de Uso

Este guia cobre todas as formas de usar o Hippocore DB, quando escolher cada
uma, e exemplos concretos para cada caminho.

---

## Os quatro modos de integração

| Modo | Quando usar | Binário / crate |
|---|---|---|
| **Biblioteca Rust embedded** | Sua aplicação é em Rust | `crates/hippocore` |
| **CLI** | Scripting, dev local, testes, ingestão avulsa | binário `hippocore` |
| **Servidor HTTP** | Qualquer linguagem/runtime, microsserviços, Docker | `hippocore serve` |
| **TUI Interativo** | Explorar dados, depurar recalls, revisão manual | `hippocore studio` |

---

## Modo 1 — Biblioteca Rust embedded

Adicione ao `Cargo.toml`:

```toml
[dependencies]
hippocore = { path = "path/to/hippocore" }   # ou versão do crates.io
```

### Abrir e inicializar

```rust
use hippocore::{Config, Hippocore};

let mut db = Hippocore::open(Config::new("./data"))?;

// Cada tenant é um limite de isolamento. Collections agrupam docs/memórias.
db.create_tenant("acme", "Acme Corp")?;
db.create_collection("acme", "suporte", "base de conhecimento de suporte")?;
```

`create_collection` é **idempotente** — chamar duas vezes para o mesmo par
(tenant, nome) retorna a collection existente sem erro.

### Armazenar uma memória (atômica, já com embedding)

```rust
use hippocore::model::MemoryType;
use hippocore::RememberRequest;

let mem = db.remember(RememberRequest::new(
    "acme", "suporte", MemoryType::Semantic,
    "Oracle ORA-12514 significa que o listener não conhece o serviço.",
))?;
println!("armazenada: {}", mem.id);
```

Tipos de memória:
- `Semantic` — um fato ("o BD é Oracle 19c")
- `Episodic` — um evento ("usuário reportou falha de login às 14:32 UTC")
- `Procedural` — um passo a passo ("para redefinir a senha: …")
- `Note` — nota de forma livre

### Armazenar um documento (chunking + embedding automático)

```rust
use hippocore::StoreDocumentRequest;

let doc = db.store_document(StoreDocumentRequest::new(
    "acme", "suporte",
    "Capítulo 1: O erro ORA-12514 ocorre quando o listener do TNS não \
     encontra o nome do serviço. Verifique tnsnames.ora e listener.log.",
))?;
println!("armazenado: {} ({} chars)", doc.id, doc.text.len());
```

Os documentos são divididos em chunks de janela de tokens com sobreposição
automaticamente. Cada chunk recebe seu próprio embedding e é pesquisável de
forma independente.

### Armazenar um PDF

```rust
let bytes = std::fs::read("manual.pdf")?;
let doc = db.store_document(StoreDocumentRequest::from_pdf(
    "acme", "suporte", bytes,
))?;
```

### Armazenar código-fonte

```rust
let code = std::fs::read_to_string("src/auth.rs")?;
let doc = db.store_document(StoreDocumentRequest::from_code(
    "acme", "codebase", code, "rust",
))?;
```

Linguagens suportadas: `rust`, `python`, `javascript`, `typescript`, `go`,
`java`, `kotlin`. Linguagens não reconhecidas usam o chunker de tokens.

### Armazenar um registro estruturado (JSON, projetado em texto pesquisável)

```rust
use hippocore::PutRecordRequest;
use serde_json::json;

let rec = db.put_record(PutRecordRequest::new(
    "acme", "suporte", "sistemas",
    json!({"engine": "postgresql", "port": 5432, "env": "prod"}),
))?;
println!("versão do registro: {}", rec.version);
```

### Recall (híbrido por padrão)

```rust
use hippocore::RecallRequest;

let hits = db.recall(RecallRequest::new("acme", "oracle connection error"))?;
for h in &hits {
    println!("[{:.3}] {} — {}", h.score, h.id, &h.text[..80.min(h.text.len())]);
}
```

`recall` combina **similaridade vetorial** (cosseno) e **relevância textual**
(BM25) para cada resultado. Para usar apenas um modo:

```rust
use hippocore::query::SearchMode;

let mut req = RecallRequest::new("acme", "oracle");
req.mode = SearchMode::Vector;   // ou SearchMode::Text
req.top_k = 5;
req.collection = Some("suporte".into());
let hits = db.recall(req)?;
```

### Montar contexto LLM (com orçamento de tokens)

```rust
use hippocore::BuildContextRequest;

let ctx = db.build_context(BuildContextRequest::new(
    "acme", "oracle ORA-12514 em staging", 2048,  // máx tokens
))?;

// ctx.text está pronto para ser prefixado ao prompt do LLM.
println!("contexto ({} tokens, {} itens):\n{}", ctx.token_count, ctx.items_included.len(), ctx.text);
```

Com vizinhos do grafo incluídos:

```rust
let mut req = BuildContextRequest::new("acme", "oracle", 2048);
req.include_related = true;   // segue arestas de grafo de 1 hop
let ctx = db.build_context(req)?;
```

### Arestas de grafo

```rust
use hippocore::{AddGraphEdgeRequest, ItemKind};

db.add_graph_edge(AddGraphEdgeRequest {
    tenant_id: "acme".into(),
    from_kind: ItemKind::Memory,
    from_id: mem.id.clone(),
    to_kind: ItemKind::DocumentChunk,
    to_id: "doc-1#0".into(),
    relation: "supports".into(),
    weight: 1.0,
})?;
```

Travessia multi-hop:

```rust
use hippocore::{TraverseGraphRequest, ItemKind};

let nodes = db.traverse_graph(TraverseGraphRequest::new(
    "acme",
    [(ItemKind::Memory, mem.id.clone())],
));
for n in &nodes {
    println!("hop {} — {} {:?} via {}", n.hop, n.id, n.kind, n.via_edge_id);
}
```

### Manutenção

```rust
// Estatísticas
let s = db.stats()?;
println!("{} tenants, {} memórias, {} chunks, {} entradas WAL",
    s.tenants, s.memories, s.chunks, s.wal_entries);

// Compactar WAL em snapshot (automático; também chamável manualmente)
db.compact()?;

// Fechar graciosamente (flush e fecha arquivo WAL)
db.close()?;
```

### Validade temporal

```rust
use hippocore::RememberRequest;

let epoch_ms = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64;

let mut req = RememberRequest::new("acme", "suporte", MemoryType::Semantic,
    "Este fato é válido pelos próximos 30 dias.");
req.valid_from = Some(epoch_ms);
req.valid_until = Some(epoch_ms + 30 * 24 * 3600 * 1000);
db.remember(req)?;
```

Memórias expiradas são automaticamente excluídas do `recall` (baseado no
relógio de parede). Use `req.as_of = Some(timestamp)` para consultar estado
histórico.

---

## Modo 2 — CLI

### Instalação

```bash
git clone <repo> && cd hippocoredb
cargo build --release
# Binário em: target/release/hippocore
# Adicione ao PATH ou crie um alias:
alias hc=./target/release/hippocore
```

### Todos os subcomandos

```bash
# ─── Ciclo de vida do banco ───────────────────────────────────────────────────

hc init --db ./data                        # criar/validar diretório de dados

hc stats --db ./data                       # resumo: tenants, docs, memórias, WAL
hc inspect --db ./data                     # listar tenants e contagens de collections

hc compact --db ./data                     # dobrar WAL em snapshot


# ─── Tenants e collections ────────────────────────────────────────────────────

hc list-tenants --db ./data
hc list-collections --db ./data --tenant acme


# ─── Documentos ───────────────────────────────────────────────────────────────

hc put-document --db ./data \
    --tenant acme --collection suporte \
    --text "Oracle ORA-12514 — verifique tnsnames.ora e listener.log." \
    --meta source=wiki --meta priority=high

hc put-document --db ./data \
    --tenant acme --collection suporte \
    --text "..." --id meu-doc-id           # id explícito (upsert idempotente)

hc list-documents --db ./data --tenant acme
hc show-document  --db ./data --tenant acme --collection suporte --id <id>
hc delete-document --db ./data --tenant acme --collection suporte --id <id>


# ─── Memórias ─────────────────────────────────────────────────────────────────

hc remember --db ./data \
    --tenant acme --collection suporte \
    --type semantic \                         # episodic | semantic | procedural | note
    --text "Um cliente roda Oracle 19c em staging." \
    --meta env=staging \
    --confidence 0.9

hc remember --db ./data --tenant acme --collection suporte \
    --type episodic \
    --text "Pico de falhas de login às 14:32 UTC 2026-06-26." \
    --valid-from 1751000000000 \
    --valid-until 1759000000000

hc list-memories  --db ./data --tenant acme
hc show-memory    --db ./data --tenant acme --collection suporte --id <id>
hc forget         --db ./data --tenant acme --collection suporte --id <id>
hc rate-memory    --db ./data --tenant acme --collection suporte --id <id> \
    --confidence 0.95                         # avaliação humana


# ─── Registros (JSON estruturado) ────────────────────────────────────────────

hc put-record --db ./data \
    --tenant acme --collection suporte --table sistemas --id billing-db \
    --json '{"engine":"postgresql","port":5432,"env":"prod"}'

hc list-records  --db ./data --tenant acme --table sistemas
hc show-record   --db ./data --tenant acme --collection suporte \
    --table sistemas --id billing-db
hc delete-record --db ./data --tenant acme --collection suporte \
    --table sistemas --id billing-db


# ─── Importar arquivo (texto: .txt, .md, .json, .csv) ────────────────────────

hc import-file --db ./data \
    --tenant acme --collection suporte \
    --id runbook --path ./runbook.md \
    --meta source=confluence

hc list-files   --db ./data --tenant acme
hc show-file    --db ./data --tenant acme --collection suporte --id runbook
hc delete-file  --db ./data --tenant acme --collection suporte --id runbook


# ─── Recall ───────────────────────────────────────────────────────────────────

hc recall --db ./data --tenant acme \
    --query "oracle connection refused" --top-k 5

hc recall --db ./data --tenant acme --query "oracle" \
    --mode vector                              # vector | text | hybrid (padrão)

hc recall --db ./data --tenant acme --query "oracle" \
    --collection suporte                       # escopar a uma collection

hc recall --db ./data --tenant acme --query "oracle" \
    --json                                     # saída legível por máquina

# Com embeddings externos (ex.: Ollama / OpenAI):
hc recall --db ./data --tenant acme --query "oracle" \
    --embedding "0.12,-0.34,0.56,..."


# ─── Montar contexto LLM ─────────────────────────────────────────────────────

hc build-context --db ./data --tenant acme \
    --query "oracle ORA-12514 em staging" \
    --max-tokens 2048

hc build-context --db ./data --tenant acme \
    --query "oracle" --max-tokens 1024 --json   # retorna JSON ContextBlock


# ─── Arestas de grafo ────────────────────────────────────────────────────────

hc add-edge --db ./data --tenant acme \
    --from-kind Memory --from-id <mem-id> \
    --to-kind DocumentChunk --to-id <doc-id>#0 \
    --relation supports

hc list-edges  --db ./data --tenant acme
hc delete-edge --db ./data --tenant acme --id <edge-id>


# ─── Auditoria ────────────────────────────────────────────────────────────────

hc audit --db ./data --tenant acme --from 0 --to 9223372036854775807
                                             # replay de todos os registros
hc audit --db ./data --tenant acme \
    --from 1750000000000 --to 1760000000000   # janela de tempo

hc compact-audit --db ./data --max-records 500
hc compact-audit --db ./data --max-bytes 1048576


# ─── Qualidade de recuperação ─────────────────────────────────────────────────

hc eval-quality --db ./data --tenant acme     # fixtures de retrieval, imprime métricas
```

### Saída legível por máquina

Todos os comandos de leitura aceitam `--json` para emitir JSON estruturado no
stdout, adequado para piping para `jq` ou outras ferramentas:

```bash
hc recall --db ./data --tenant acme --query "oracle" --top-k 3 --json | \
    jq '.[].text'
```

---

## Modo 3 — Servidor HTTP

### Iniciar o servidor

```bash
# Via variável de ambiente (recomendado):
HIPPOCORE_API_KEY=changeme hippocore serve --port 8080 --db ./data

# Via flag:
hippocore serve --port 8080 --db ./data --api-key changeme
```

O servidor loga o endereço no stderr. Sem auth em `/health`; todos os outros
endpoints exigem o header `X-Api-Key: changeme`.

### Verificação de saúde

```bash
curl http://localhost:8080/health
# 200 OK
```

### Referência completa de endpoints

```bash
BASE=http://localhost:8080
KEY=changeme
AUTH="-H 'X-Api-Key: $KEY'"

# ─── Stats ────────────────────────────────────────────────────────────────────
curl $AUTH $BASE/stats

# ─── Tenant ───────────────────────────────────────────────────────────────────
curl -X POST $AUTH -H 'Content-Type: application/json' \
    -d '{"id":"acme","name":"Acme Corp"}' \
    $BASE/tenants

# ─── Collection ───────────────────────────────────────────────────────────────
curl -X POST $AUTH -H 'Content-Type: application/json' \
    -d '{"name":"suporte","description":"Base de Conhecimento"}' \
    $BASE/tenants/acme/collections

# ─── Memória ──────────────────────────────────────────────────────────────────
curl -X POST $AUTH -H 'Content-Type: application/json' \
    -d '{"collection":"suporte","text":"Oracle ORA-12514","memory_type":"semantic"}' \
    $BASE/tenants/acme/memories

curl -X DELETE $AUTH $BASE/tenants/acme/memories/<id>

# ─── Documento ────────────────────────────────────────────────────────────────
curl -X POST $AUTH -H 'Content-Type: application/json' \
    -d '{"collection":"suporte","text":"Texto completo do documento..."}' \
    $BASE/tenants/acme/documents

# ─── Recall ───────────────────────────────────────────────────────────────────
curl -X POST $AUTH -H 'Content-Type: application/json' \
    -d '{"query":"oracle connection","top_k":5}' \
    $BASE/tenants/acme/recall

# Resposta: [{"id":"...","score":0.87,"text":"...","kind":"Memory","document_id":null}]

# ─── Montar contexto ─────────────────────────────────────────────────────────
curl -X POST $AUTH -H 'Content-Type: application/json' \
    -d '{"query":"oracle ORA-12514","max_tokens":2048,"include_related":false}' \
    $BASE/tenants/acme/context

# Resposta: {"text":"[Memory:mem-123]\nOracle ORA...","token_count":142,"items_included":3,"items_dropped":0}

# ─── Travessia de grafo ──────────────────────────────────────────────────────
curl -X POST $AUTH -H 'Content-Type: application/json' \
    -d '{"seeds":[{"kind":"Memory","id":"mem-123"}],"max_hops":2,"max_nodes":20}' \
    $BASE/tenants/acme/graph/traverse
```

### Usando em Python

```python
import requests

BASE = "http://localhost:8080"
HEADERS = {"X-Api-Key": "changeme", "Content-Type": "application/json"}

# Setup
requests.post(f"{BASE}/tenants", json={"id": "acme", "name": "Acme"}, headers=HEADERS)
requests.post(f"{BASE}/tenants/acme/collections",
              json={"name": "suporte"}, headers=HEADERS)

# Armazenar
requests.post(f"{BASE}/tenants/acme/memories",
              json={"collection": "suporte", "text": "Oracle error ORA-12514"},
              headers=HEADERS)

# Recall
hits = requests.post(f"{BASE}/tenants/acme/recall",
                     json={"query": "oracle connection", "top_k": 5},
                     headers=HEADERS).json()
for h in hits:
    print(f"[{h['score']:.3f}] {h['text'][:80]}")

# Montar contexto para LLM
ctx = requests.post(f"{BASE}/tenants/acme/context",
                    json={"query": "oracle ORA-12514", "max_tokens": 2048},
                    headers=HEADERS).json()
prompt = f"Contexto:\n{ctx['text']}\n\nPergunta: O que é ORA-12514?"
```

### Usando em TypeScript / Node.js

```typescript
const BASE = "http://localhost:8080";
const headers = { "X-Api-Key": "changeme", "Content-Type": "application/json" };

// Armazenar memória
await fetch(`${BASE}/tenants/acme/memories`, {
  method: "POST",
  headers,
  body: JSON.stringify({ collection: "suporte", text: "Oracle ORA-12514" }),
});

// Recall
const hits = await fetch(`${BASE}/tenants/acme/recall`, {
  method: "POST",
  headers,
  body: JSON.stringify({ query: "oracle connection error", top_k: 5 }),
}).then(r => r.json());

// Montar contexto LLM
const ctx = await fetch(`${BASE}/tenants/acme/context`, {
  method: "POST",
  headers,
  body: JSON.stringify({ query: "oracle ORA-12514", max_tokens: 2048 }),
}).then(r => r.json());

const prompt = `Contexto:\n${ctx.text}\n\nPergunta: O que é ORA-12514?`;
```

### Docker

```dockerfile
FROM rust:1.96-slim AS builder
WORKDIR /build
COPY . .
RUN cargo build --release --package hippocore-cli

FROM debian:bookworm-slim
COPY --from=builder /build/target/release/hippocore /usr/local/bin/hippocore
VOLUME /data
ENV HIPPOCORE_API_KEY=changeme
EXPOSE 8080
CMD ["hippocore", "serve", "--port", "8080", "--db", "/data"]
```

```bash
docker build -t hippocore .
docker run -p 8080:8080 -v $(pwd)/data:/data \
    -e HIPPOCORE_API_KEY=supersecret hippocore
```

---

## Modo 4 — TUI Interativo (Studio)

```bash
hippocore studio --db ./data
```

### Navegação

| Tecla | Ação |
|---|---|
| `Tab` / `Shift-Tab` | Alternar entre abas |
| `↑` / `↓` | Navegar na lista |
| `q` | Sair |

### Abas

- **Tenants** — lista todos os tenants com contagens de collections
- **Memories** — navegar memórias com preview de texto; filtrável por tenant
- **Documents** — navegar documentos com contagens de chunks
- **Stats** — estatísticas ao vivo (entradas WAL, bytes em disco, entradas indexadas)

O TUI lê dados na abertura; é somente leitura (sem escrita via UI).
Use-o junto com a CLI ou biblioteca para inspeção e depuração.

---

## Escolhendo o modo certo

```
Sua aplicação é em Rust?
  └── Biblioteca embedded  ← zero overhead de rede, transações in-process

Está fazendo scripting / ingestão avulsa?
  └── CLI  ← sem servidor, saída --json para pipes

Precisa de acesso de qualquer linguagem ou quer rodar atrás de proxy?
  └── Servidor HTTP  ← inicia uma vez, chamado de Python/JS/Go/…

Quer navegar dados visualmente ou depurar um recall?
  └── TUI studio  ← sem necessidade de código
```

---

## Padrão RAG (ponta a ponta)

```
┌──────────────┐  ingestão  ┌──────────────────┐
│ documentos / │──────────► │  Hippocore DB     │
│ memórias /   │            │  (store_document  │
│ PDF / código │            │   remember)       │
└──────────────┘            └────────┬─────────┘
                                     │ recall / build_context
                     ┌───────────────▼──────────────────────┐
                     │ LLM (Ollama / OpenAI / Anthropic)     │
                     │  prompt = ctx.text + pergunta         │
                     └───────────────────────────────────────┘
```

1. **Ingestão**: `store_document` ou `remember` (CLI, biblioteca ou HTTP)
2. **Recall**: `recall(query)` → resultados ranqueados (híbrido vetorial + BM25)
3. **Montar contexto**: `build_context(query, max_tokens)` → string pronta para uso
4. **Gerar**: prefixe `ctx.text` ao prompt do LLM; cite `ctx.items_included`

O embedder embutido é determinístico e offline. Para qualidade de produção,
forneça seus próprios embeddings de um modelo (OpenAI, Ollama, etc.) via
`RememberRequest.embedding` ou `StoreDocumentRequest.chunks`.

---

## Persistência e segurança a falhas

```
<data_dir>/
  wal.log        write-ahead log; uma op por linha: <crc32-hex>\t<json>\n
  snapshot.json  estado completo, escrito atomicamente (tmp + fsync + rename)
  meta.json      versão do formato
```

- Toda escrita adiciona ao `wal.log` com checksum CRC32.
- No `open`, o snapshot é carregado e o WAL é replayed em cima.
- Uma linha WAL truncada ao final (crash durante escrita) é pulada, nunca fatal.
- Erro de CRC32 para o replay naquela linha — nenhum estado corrompido é carregado.
- Compactação automática ocorre quando o WAL excede `auto_compact_after_ops`
  (padrão 1000) ou `auto_compact_after_bytes`. `compact()` também pode ser chamado manualmente.

---

## Isolamento multi-tenant

Todo método de API que lê ou escreve dados requer um `tenant_id` explícito.
O isolamento é aplicado na camada de queries: busca vetorial, busca textual,
travessia de grafo e montagem de contexto filtram por `tenant_id` antes de
pontuar. Não há estado compartilhado entre tenants mesmo dentro do mesmo
diretório de dados.

---

## Configuração

```rust
use hippocore::Config;

let cfg = Config::new("./data")
    .with_embedding_dim(256)            // padrão 128
    .with_chunk_tokens(64)              // padrão 48; tokens por janela de chunk
    .with_auto_compact_after_ops(500)   // padrão 1000
    .with_auto_compact_after_bytes(5 * 1024 * 1024);  // padrão 10 MiB
```

Todas as configurações são por processo; não há arquivo de config em disco.
