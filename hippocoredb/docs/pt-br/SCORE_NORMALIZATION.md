# Score Normalization v0.1

## Visão geral

A Score Normalization é um passo de pré-processamento interno em `build_context`
que limita os scores brutos de recall híbrido ao range `[0.0, 1.0]` antes das
mesclagens de Temporal Decay e Graph-Aware Ranking serem aplicadas. Sem esse
passo, os pesos de mesclagem (`temporal_weight`, `graph_rank_weight`) têm efeitos
assimétricos quando os scores de recall excedem `1.0`, o que pode ocorrer com
certas formas de query/corpus.

Este passo é transparente para o caller e não adiciona novos campos de
configuração.

## Como funciona

Imediatamente após o conjunto de candidatos ser construído (itens recuperados
mais itens opcionalmente expandidos por grafo), e apenas quando pelo menos um
peso de mesclagem é não-zero:

1. Encontra `max_score = max(candidates.score)`.
2. Se `max_score > 0`, divide cada `candidate.score` por `max_score`.

O resultado: o candidato com maior score tem score normalizado exatamente `1.0`;
todos os outros estão em `(0.0, 1.0]`. A ordenação relativa é preservada.

Quando ambos os pesos de mesclagem são `0.0` (padrão) a normalização é
completamente ignorada — `build_context` se comporta de forma idêntica ao
comportamento pré-normalização.

## Invariantes

- A ordenação relativa dentro do conjunto de candidatos é sempre preservada.
- Com um único candidato, o score normalizado é `1.0` (assumindo score bruto
  não-zero).
- A normalização é escopada por query; scores de chamadas diferentes de
  `build_context` não são comparáveis.
- Este passo afeta apenas o caminho de mesclagem interno; scores retornados via
  `ContextItem.score` refletem o score efetivo pós-mesclagem.

## Fora de escopo

- Normalização de scores retornados por `recall()` ou `search()`.
- Calibração de scores entre queries.
- Modo server.
