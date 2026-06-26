# Próxima Feature

## Nome

**Graph-Aware Ranking v0.1** — aumentar scores de recall usando densidade de
arestas no grafo.

## Por que importa

O Graph-Aware Context v0.1 já expande o conjunto de candidatos via vizinhos
diretos no grafo, e o Audit Retention v0.1 limita o crescimento do audit log.
A lacuna remanescente na qualidade de retrieval é que itens conectados por
muitas arestas a outros itens de alta pontuação recuperados ainda não são
preferidos em relação a itens isolados de alta pontuação. Um passo de ranking
ciente de grafo pode elevar contexto genuinamente conectado acima de coincidências
léxicas.

## Comportamento

Após o passo de recall híbrido produzir uma lista de candidatos ranqueada,
aplica um bônus de conectividade de grafo:

- Para cada candidato, conta quantos de seus vizinhos diretos também estão no
  conjunto de candidatos.
- Escala o bônus por um `graph_rank_weight` configurável (padrão `0.1`).
- `effective_score = recall_score * (1 - graph_rank_weight) + connectivity_bonus * graph_rank_weight`.
- Itens sem vizinhos no conjunto de candidatos não são afetados.

Isso mantém o ranking genérico (sem boost de entidade ou domínio hard-coded) e
respeita o re-ranking por confiança/contradição já existente em `build_context`.

## Arquivos

- `crates/hippocore/src/lib.rs` — re-ranking ciente de grafo em `build_context`
  após `run_query`.
- `crates/hippocore/src/config.rs` — novo campo `graph_rank_weight: f32`
  (padrão `0.1`).
- `crates/hippocore/tests/database.rs` — testes determinísticos.
- `docs/en/GRAPH_AWARE_RANKING.md` e `docs/pt-br/GRAPH_AWARE_RANKING.md`.
- `docs/en/STATUS.md` e `docs/pt-br/STATUS.md`.

## Critérios de aceite

- `recall_score` não muda quando `graph_rank_weight = 0.0`.
- Um item com mais vizinhos no conjunto de candidatos é ranqueado acima de um
  item de score igual sem nenhum vizinho.
- `graph_rank_weight = 0.0` na config mantém a ordenação atual exatamente.
- Definir `graph_rank_weight` fora do range `[0.0, 1.0]` retorna erro tipado.
- Mínimo de 3 testes de integração determinísticos.
- Quality gate:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Fora de escopo

- Travessia multi-hop.
- Clustering de grafo ou detecção de comunidade.
- Modo server.
- Ranking por ML/learning-to-rank.
