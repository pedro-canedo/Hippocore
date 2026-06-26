# Graph-Aware Context

Graph-Aware Context v0.1 permite que chamadores optem por adicionar vizinhos
diretos do grafo ao montar um bloco de contexto para LLM.

Isso não é GraphRAG. A feature não altera ranking de recall, não faz travessia
multi-hop e não aplica boost de score. Ela considera apenas vizinhos diretos de
`GraphEdge` dos itens recuperados e os inclui quando couberem no budget de
tokens.

## API

```rust
let mut req = BuildContextRequest::new("acme", "postgresql connection", 2048);
req.top_k_candidates = 5;
req.include_related = true;
req.related_limit = 4;

let block = db.build_context(req)?;
```

Defaults:

- `include_related = false`
- `related_limit = 8`

Quando desabilitado, `build_context` mantém o comportamento anterior.

## Proveniência

`ContextItem` agora expõe:

- `related_item_ids`
- `inclusion_source`

`inclusion_source` é `recalled` para hits normais de recall e `graph_expanded`
para itens adicionados por expansão direta do grafo.

`AuditItem` usa os mesmos rótulos e também registra `not_included` para itens
considerados mas não incluídos no contexto final.

## CLI

```bash
hippocore build-context \
  --db ./data \
  --tenant acme \
  --query "postgresql connection" \
  --top-k 5 \
  --include-related \
  --related-limit 4 \
  --json
```

A saída JSON inclui `inclusion_source` para cada item incluído.

## Limites

- Apenas vizinhos diretos são considerados.
- A expansão é isolada por tenant.
- `related_limit` limita candidatos expandidos pelo grafo.
- O mesmo item nunca é duplicado quando já veio pelo recall.
- O budget de tokens continua controlando o contexto final.
