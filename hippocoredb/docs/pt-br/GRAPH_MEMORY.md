# Graph Memory

Graph Memory v0.1 adiciona arestas duráveis de relacionamento direto entre
itens de contexto armazenados. A entrega é intencionalmente pequena: arestas
melhoram proveniência e auditabilidade, mas ainda não alteram ranking de recall
nem fazem travessia GraphRAG.

## Modelo

`GraphEdge` armazena:

- `id`
- `tenant_id`
- `from_id` / `from_kind`
- `to_id` / `to_kind`
- `relation`
- `metadata`
- `created_at`
- `updated_at`

Os tipos de endpoint suportados são `memory`, `record` e `document_chunk`.

A validação de endpoint é escopada por tenant e usa `(tenant_id, kind, id)`.
Como o CLI evita uma linguagem de query de grafo na v0.1, endpoints que
combinam com mais de um item no tenant são rejeitados como ambíguos.

## API

```rust
let edge = db.add_graph_edge(AddGraphEdgeRequest::new(
    "acme",
    ItemKind::Memory,
    "postgres-policy",
    ItemKind::Memory,
    "python-client",
    "mentions",
))?;

let edges = db.list_graph_edges("acme", Some("postgres-policy"));
let neighbours = db.graph_neighbors("acme", ItemKind::Memory, "postgres-policy");
db.delete_graph_edge("acme", &edge.id)?;
```

As arestas são persistidas pelo mesmo modelo de WAL/snapshot usado por
documentos, memórias, records e arquivos. Reabrir o banco restaura as arestas.

## CLI

```bash
hippocore add-edge \
  --db ./data \
  --tenant acme \
  --from-kind memory \
  --from-id postgres-policy \
  --to-kind memory \
  --to-id python-client \
  --relation mentions \
  --json

hippocore list-edges --db ./data --tenant acme --from-id postgres-policy --json
hippocore delete-edge --db ./data --tenant acme --id edge-123
```

## Contexto e Auditoria

`ContextItem` e `AuditItem` agora incluem `related_item_ids`, os ids dos
vizinhos diretos conhecidos no momento de `build_context`. Isso é apenas
proveniência; a v0.1 não expande nem impulsiona recall com arestas de grafo.

## Semântica de Deleção

Deletar uma memória ou record remove arestas que tocam esse endpoint. Deletar um
documento ou arquivo importado remove arestas que tocam seus chunks derivados.
Deletar uma aresta inexistente é um no-op seguro.

## Fora de Escopo

- Travessia GraphRAG completa.
- Ranking ciente de grafo.
- Extração de entidades.
- Linguagem de query por padrões.
- Server mode.
