# Próxima Feature

## Nome

**Graph Memory v0.1** — arestas duráveis de relacionamento entre itens de
contexto.

## Por que importa

O RAG Audit Engine agora explica quais itens foram recuperados para um bloco de
contexto, mas o Hippocore ainda trata memórias, chunks de documento e records
como um conjunto plano. Agentes frequentemente precisam saber que um item
explica, depende, menciona, contradiz ou se relaciona com outro. Uma camada
pequena de grafo melhora a confiabilidade de contexto sem saltar para GraphRAG
completo.

Graph Memory v0.1 deve adicionar:

- Modelo tipado `GraphEdge` com `tenant_id`, `from_id`, `from_kind`, `to_id`,
  `to_kind`, `relation`, metadata e timestamps.
- APIs duráveis para adicionar, listar e deletar arestas.
- Helpers de travessia por tenant apenas para vizinhos diretos.
- Comandos CLI para adicionar/listar/deletar arestas com saída `--json`.
- Consciência opcional no Context Compiler para incluir ids de vizinhos diretos
  na proveniência/auditoria.

## Critérios de aceite

- `GraphEdge` é persistido pelo modelo atual de WAL/snapshot e sobrevive a
  reopen.
- APIs de aresta validam isolamento por tenant e rejeitam endpoints
  desconhecidos.
- Deletar um item remove ou oculta arestas penduradas de forma determinística.
- Comandos CLI funcionam:
  - `add-edge --tenant <t> --from-kind <kind> --from-id <id> --to-kind <kind>
    --to-id <id> --relation <name>`
  - `list-edges --tenant <t> [--from-id <id>] [--json]`
  - `delete-edge --tenant <t> --id <edge_id>`
- Mínimo de 4 testes determinísticos: caminho feliz add/list, erro de endpoint
  desconhecido, isolamento por tenant, durabilidade após restart.
- Documentação atualizada em inglês e português.
- Quality gate completo:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Fora de escopo

- Travessia ou ranking GraphRAG completo.
- Linguagem de query para padrões de grafo.
- Extração de entidades.
- Server/HTTP mode.
- Armazenamento distribuído de grafo.

## Follow-up

Após Graph Memory v0.1, decidir se o próximo passo de maior valor é ranking de
recall ciente de grafo ou retenção/rotação do log de auditoria.
