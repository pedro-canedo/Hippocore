# Graph-Aware Ranking v0.1

## Visão geral

O Graph-Aware Ranking é um passo de re-ranking opcional em `build_context` que
aumenta o score efetivo de candidatos recuperados cujos vizinhos diretos no
grafo também aparecem no mesmo conjunto de candidatos. Ele recompensa itens de
contexto densamente conectados a outras memórias relevantes, elevando contexto
genuinamente relacionado acima de coincidências léxicas.

O recurso é desativado por padrão (`graph_rank_weight = 0.0`) e não adiciona
overhead quando desativado.

## Como funciona

Após o passo de recall híbrido produzir uma lista de candidatos ranqueada
(incluindo itens expandidos pelo grafo) e após o re-ranking por
confiança/contradição (quando presente), o passo de grafo é executado:

1. Constrói um conjunto com todos os IDs de candidatos.
2. Para cada candidato `c`, conta quantos IDs em seu `related_item_ids` aparecem
   no conjunto de candidatos. Chama isso de `neighbours_in_set`.
3. Normaliza: `connectivity_score = neighbours_in_set / (total_candidates - 1)`,
   limitado a `[0.0, 1.0]`.
4. Mescla ao score efetivo:

   ```
   effective_score = recall_score × (1 − w) + connectivity_score × w
   ```

   onde `w = config.graph_rank_weight`.

5. Re-ordena por `effective_score` decrescente.

Itens sem arestas de grafo produzem `connectivity_score = 0`, então seu score
efetivo é `recall_score × (1 − w)`. Quando `w = 0`, todos os itens escalam
identicamente e a ordenação permanece inalterada.

## Configuração

```rust
use hippocore::Config;

let mut cfg = Config::new("./meudb");
// Habilitar graph-aware ranking com boost leve:
cfg.graph_rank_weight = 0.1;

let db = Hippocore::open(cfg)?;
```

| Campo | Tipo | Padrão | Descrição |
|---|---|---|---|
| `graph_rank_weight` | `f32` | `0.0` | Peso de mistura em `[0.0, 1.0]` para bônus de conectividade |

Passar um valor fora de `[0.0, 1.0]` faz `build_context` retornar
`HippocoreError::Validation` antes de qualquer I/O.

## Invariantes

- `w = 0.0` — ordenação idêntica ao recall híbrido simples.
- Itens com zero vizinhos no conjunto de candidatos não recebem boost.
- A normalização é relativa ao tamanho atual do conjunto de candidatos; contagem
  absoluta de arestas não é comparada entre queries.
- O passo só é executado quando o conjunto de candidatos tem pelo menos 2 itens.
- Sem travessia de grafo — apenas vizinhos diretos (1-hop) são considerados.

## Fora de escopo

- Conectividade multi-hop ou transitiva.
- Clustering de grafo ou detecção de comunidade.
- Ranking por ML/learning-to-rank.
- Modo server.
