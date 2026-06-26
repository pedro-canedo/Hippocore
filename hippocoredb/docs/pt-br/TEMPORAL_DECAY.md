# Temporal Decay v0.1

## Visão geral

O Temporal Decay é um passo de re-ranking opcional em `build_context` que
aplica um viés de recência aos scores de recall. Itens criados ou atualizados
recentemente recebem um score de decaimento maior do que itens mais antigos,
evitando que uma memória desatualizada-mas-semanticamente-similar desloque uma
memória mais recente e ligeiramente menos similar.

O recurso é desativado por padrão (`temporal_weight = 0.0`) e não adiciona
overhead quando desativado.

## Como funciona

Após o passo de recall híbrido (e após o re-ranking por confiança quando há
contradições), o passo de decaimento temporal é executado sobre o conjunto de
candidatos:

1. Para cada candidato, obtém o timestamp efetivo:
   - **Memory** → `created_at` (memórias são imutáveis; usa tempo de criação).
   - **Document chunk** → `updated_at` do documento pai.
   - **Record** → `updated_at`.
2. Calcula `age_days = (now_ms - timestamp_ms) / 86_400_000`.
3. Calcula um score de decaimento exponencial:
   ```
   decay = exp(-age_days / temporal_decay_days)
   ```
   Começa próximo de `1.0` para itens novos e cai em direção a `0.0` para
   itens muito antigos, com `temporal_decay_days` controlando a meia-vida.
   Itens cujo timestamp não pode ser resolvido recebem decaimento neutro de `0.5`.
4. Mescla ao score efetivo:
   ```
   effective_score = recall_score × (1 − w) + decay × w
   ```
   onde `w = config.temporal_weight`.
5. Re-ordena por score efetivo decrescente.

Quando `w = 0` (padrão) a fórmula se reduz a `recall_score` e a ordenação
permanece inalterada.

## Configuração

```rust
use hippocore::Config;

let mut cfg = Config::new("./meudb");
// Habilitar temporal decay com viés moderado (meia-vida de 30 dias):
cfg.temporal_weight = 0.2;
cfg.temporal_decay_days = 30.0;

let db = Hippocore::open(cfg)?;
```

| Campo | Tipo | Padrão | Descrição |
|---|---|---|---|
| `temporal_weight` | `f32` | `0.0` | Peso de mistura em `[0.0, 1.0]` para score de recência |
| `temporal_decay_days` | `f32` | `30.0` | Meia-vida exponencial em dias; deve ser `> 0` quando peso `> 0` |

Passar `temporal_weight` fora de `[0.0, 1.0]` ou `temporal_decay_days ≤ 0`
quando `temporal_weight > 0` faz `build_context` retornar
`HippocoreError::Validation` antes de qualquer I/O.

## Invariantes

- `w = 0.0` — ordenação idêntica ao recall híbrido simples.
- Itens com a mesma idade recebem o mesmo score de decaimento.
- Itens cujo timestamp não pode ser resolvido recebem decaimento neutro de
  `0.5`, então não são silenciosamente descartados.
- O passo é executado para todos os tipos de item; chunks de documentos usam
  o timestamp do documento pai para não penalizar chunks de documentos
  atualizados recentemente.
- O Temporal Decay é aplicado antes do Graph-Aware Ranking, então as duas
  features se compõem.

## Fora de escopo

- Reindexação periódica em background por idade.
- Evicção automática de memórias desatualizadas.
- Travessia multi-hop de grafo.
- Modo server.
