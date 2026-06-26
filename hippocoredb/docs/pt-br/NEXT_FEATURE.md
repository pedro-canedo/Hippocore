# Próxima Feature

## Nome

**Score Normalization v0.1** — normalizar scores de recall e entradas de
mesclagem para o range consistente `[0.0, 1.0]` antes do Temporal Decay e do
Graph-Aware Ranking.

## Por que importa

`temporal_weight` e `graph_rank_weight` assumem que ambos os operandos da
mesclagem estão em `[0.0, 1.0]`. Os scores de decaimento e conectividade já
são normalizados, mas o score de recall bruto (um híbrido de cosseno + TF-IDF)
pode exceder `1.0` dependendo da forma da query ou do corpus, tornando a
mesclagem assimétrica. Sem normalização, os parâmetros de peso não têm
semântica intuitiva e independente do corpus.

## Comportamento

Antes das mesclagens de Temporal Decay e Graph-Aware Ranking, normalizar o
`recall_score` bruto de cada candidato:

- Coletar todos os scores do conjunto de candidatos.
- `max_score = max(scores)`.
- `normalised = score / max_score` (ou `0.0` quando max é `0`).
- Prosseguir com `normalised` como base em todas as fórmulas de mesclagem.

A normalização é escopada por query (relativa ao conjunto de candidatos atual),
então valores absolutos de score entre queries permanecem incomparáveis — isso
é intencional.

## Arquivos

- `crates/hippocore/src/lib.rs` — passo de normalização em `build_context`
  antes do Temporal Decay e Graph-Aware Ranking.
- `crates/hippocore/tests/database.rs` — testes determinísticos.
- `docs/en/SCORE_NORMALIZATION.md` e `docs/pt-br/SCORE_NORMALIZATION.md`.
- `docs/en/STATUS.md` e `docs/pt-br/STATUS.md`.

## Critérios de aceite

- Todos os scores de candidatos no caminho de mesclagem estão em `[0.0, 1.0]`.
- A normalização não muda a ordenação relativa quando nenhuma mesclagem está
  ativa (todos os pesos `= 0`).
- Quando um único candidato é retornado, ele recebe score `1.0`.
- Mínimo de 2 testes de integração determinísticos.
- Quality gate:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Fora de escopo

- Normalização de scores de recall retornados por `recall()` ou `search()`
  (afeta apenas a mesclagem em `build_context`).
- Persistência ou calibração de scores entre queries.
- Modo server.
