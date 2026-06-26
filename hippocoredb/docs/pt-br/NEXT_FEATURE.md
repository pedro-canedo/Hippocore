# Próxima Feature

## Nome

**RRF Hybrid Fusion** — substituir a normalização min-max + fusão linear com
alpha do modo híbrido por Reciprocal Rank Fusion (RRF).

## Por que importa

O modo híbrido atual normaliza os scores de vetor e BM25 independentemente
via min-max e combina como `alpha * vector_norm + (1-alpha) * text_norm`.
Essa abordagem tem dois problemas conhecidos:

1. **Sensibilidade à escala**: normalização min-max colapsa uma diferença de 0,9
   entre os ranks 1 e 2 para a mesma largura que uma diferença de 0,001,
   tornando ranks adjacentes indistinguíveis quando um score domina.
2. **alpha manual**: `hybrid_alpha` exige ajuste por caso de uso. Um valor
   padrão fixo de 0,5 é arbitrário.

Reciprocal Rank Fusion é sem parâmetro, baseado em rank e bem estudado:

```
RRF(d) = 1 / (k + rank_vetor(d))  +  1 / (k + rank_texto(d))
```

onde `k = 60` é a constante de suavização padrão, `rank_vetor(d)` é o rank
1-based de `d` na lista ordenada por vetor e `rank_texto(d)` é o rank na lista
BM25 (itens sem match recebem rank penalizado além do conjunto de candidatos).

## Comportamento esperado

- `SearchMode::Hybrid` usa RRF internamente em vez de fusão ponderada com alpha.
- `hybrid_alpha` em `Config` fica sem efeito para o modo híbrido; ainda é aceito
  sem erro para compatibilidade, mas não influencia o score.
- `RecallResult.vector_score` e `text_score` continuam com os scores brutos
  (não normalizados) de cosine e BM25 para transparência.
- `RecallResult.reason` para modo híbrido reporta o score RRF:
  `hybrid/rrf(vector_rank=N, text_rank=M)`.
- `SearchMode::Vector` e `SearchMode::Text` permanecem inalterados.
- Todos os testes existentes devem passar; os thresholds do fixture de qualidade
  devem continuar sendo atendidos ou melhorar.

## Arquivos afetados

- `crates/hippocore/src/query.rs` — substituir lógica de fusão por RRF.
- `crates/hippocore/tests/database.rs` — verificar que os testes de modo híbrido
  ainda passam (asserções de comportamento devem ser estáveis).
- `crates/hippocore/tests/retrieval_quality.rs` — executar fixture; verificar
  thresholds.
- `docs/en/STATUS.md` e `docs/pt-br/STATUS.md`.
- `CHANGELOG.md`, `ROADMAP.md`.

## Critérios de aceite

- `SearchMode::Hybrid` usa RRF.
- A string de reason inclui "rrf" no modo híbrido.
- Os thresholds de qualidade (hit@1, hit@5, MRR) são atendidos.
- Todos os 67+ testes passam.
- `cargo fmt`, `cargo clippy -D warnings` limpos.
- Sem novas dependências.

## Fora de escopo

- Alterar `SearchMode::Vector` ou `SearchMode::Text`.
- Remover `hybrid_alpha` da API pública (manter para compatibilidade, silenciando
  o efeito).
- HNSW, ANN, server mode.

## Follow-up

Após RRF, o próximo passo é o comando CLI `eval-quality` para que operadores
possam executar avaliações de fixture de qualidade contra qualquer banco implantado.
