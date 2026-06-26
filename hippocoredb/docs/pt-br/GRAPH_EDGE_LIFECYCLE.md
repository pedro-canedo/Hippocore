# Graph Edge Lifecycle Tests v0.1

## Visão geral

Uma suite de testes dedicada verificando o ciclo de vida completo de arestas de
grafo: adicionar, listar, armazenamento do tipo de relação, deletar, durabilidade
após compact + reabrir e isolamento de tenant. Nenhum código de produção foi
alterado; esta feature é puramente testes aditivos.

## Arquivo de testes

`crates/hippocore/tests/graph_edge_lifecycle.rs`

## O que é testado

| Teste | Cenário |
|---|---|
| `added_edge_appears_in_list` | `add_graph_edge` → `list_graph_edges` retorna a aresta |
| `relation_type_is_stored` | String de relação persiste no armazenamento |
| `deleted_edge_absent_from_list` | `delete_graph_edge` remove aresta de `list_graph_edges` |
| `edges_survive_compact_reopen` | Aresta presente após `compact()` + reabertura a frio |
| `graph_edges_are_tenant_isolated` | `list_graph_edges("beta")` retorna vazio quando só "alpha" tem arestas |

## Resumo da API de arestas

- `add_graph_edge(AddGraphEdgeRequest)` — conecta dois itens (qualquer `ItemKind`)
  com uma string `relation`.
- `list_graph_edges(tenant_id, from_id)` — retorna arestas de um tenant,
  opcionalmente filtradas por `from_id`. Retorna `Vec<GraphEdge>` diretamente
  (não um `Result`).
- `delete_graph_edge(tenant_id, edge_id)` — remove a aresta duralmente.
