# GraphRAG Multi-hop Traversal — conclusão Phase 10

## Visão geral

`traverse_graph` implementa travessia BFS multi-hop de grafo (estilo GraphRAG),
completando a Phase 10. A partir de um conjunto de itens-semente (tipicamente os
melhores hits de recall), a travessia segue links `GraphEdge` em largura até
`max_hops` de profundidade, coletando no máximo `max_nodes` itens descobertos.

## Nova API pública

### `TraverseGraphRequest`

| Campo | Tipo | Padrão | Descrição |
|---|---|---|---|
| `tenant_id` | `String` | — | Escopo tenant (obrigatório) |
| `seed_ids` | `Vec<(ItemKind, String)>` | — | Pontos de partida BFS |
| `max_hops` | `usize` | `2` | Profundidade máxima |
| `max_nodes` | `usize` | `50` | Limite de resultados |
| `relation_filter` | `Option<String>` | `None` | Só seguir arestas com esta `relation` |

### `TraversalNode`

| Campo | Tipo | Descrição |
|---|---|---|
| `id` | `String` | Id do item descoberto |
| `kind` | `ItemKind` | Memory / DocumentChunk / Record |
| `hop` | `usize` | Distância em hops do semente mais próximo |
| `via_edge_id` | `String` | Aresta que introduziu este nó |

### `Hippocore::traverse_graph(req: TraverseGraphRequest) -> Vec<TraversalNode>`

Retorna nós em ordem BFS (todos os nós hop-1 antes de qualquer hop-2).
Itens-semente **não** são incluídos no resultado.

## Arquivo de testes

`crates/hippocore/tests/graph_traversal.rs` — 6 testes cobrindo descoberta
1-hop, distância de hop, fronteira `max_hops=1`, filtro de relação, limite
`max_nodes` e isolamento de tenant.

## Relação com `include_related`

O `include_related` do `build_context` é uma expansão de 1-hop integrada ao
processo de montagem de contexto. `traverse_graph` é uma API autônoma multi-hop
para callers que querem explorar o grafo sem a etapa de montagem com budget de
tokens.
