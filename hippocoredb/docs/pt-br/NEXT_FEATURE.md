# Próxima Feature

## Nome

**Graph Edge Lifecycle Tests v0.1** — testes determinísticos para o ciclo de
vida de arestas de grafo.

## Por que importa

As arestas de grafo são usadas em `build_context` para o recurso de ranking
graph-aware. As APIs de aresta (`add_graph_edge`, `list_graph_edges`,
`remove_graph_edge`) são exercidas incidentalmente em `tenant_isolation.rs` e
`database.rs`, mas nenhuma suite dedicada verifica o ciclo de vida completo:
adicionar, listar, tipos de relação, durabilidade após compact + reabrir e
isolamento de tenant.

## Comportamento

Sem novo código de produção. A suite de testes cobrirá:

1. `add_graph_edge` entre duas memórias; `list_graph_edges` a retorna.
2. Adição de aresta com string de `relation` type; a relação persiste.
3. `remove_graph_edge` remove a aresta da lista.
4. Arestas sobrevivem `compact()` + reabertura a frio.
5. `list_graph_edges` de um tenant não retorna arestas de outro tenant.

## Arquivos

- `crates/hippocore/tests/graph_edge_lifecycle.rs` — novo arquivo de testes
  dedicado.
- `docs/en/GRAPH_EDGE_LIFECYCLE.md` e `docs/pt-br/GRAPH_EDGE_LIFECYCLE.md`.
- `docs/en/STATUS.md` e `docs/pt-br/STATUS.md`.

## Critérios de aceite

- Mínimo de 5 testes de integração determinísticos usando `TempDir`.
- Quality gate:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Fora de escopo

- Algoritmos de travessia de grafo.
- Modo server.
- Mudanças no código de produção.
