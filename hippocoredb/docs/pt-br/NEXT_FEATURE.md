# Próxima Feature

## Nome

**Confidence-Weighted Recall v0.1**

## Por que importa

Memorias podem carregar um score de confianca definido pelo chamador ou pelo
comando `rate-memory`, mas o motor de recall hibrido nao o utiliza. Memorias
de alta confianca semanticamente similares à consulta podem ficar abaixo de
matches com menos confianca mas mais ruidosos. Fatorar a confianca no score
final de ranking melhora a qualidade RAG de forma mensuravel sem alterar o
modelo de dados.

## Comportamento

- Apos o score hibrido (fusao vetor + texto) ser calculado, multiplicar pelo
  peso de confianca derivado do campo `confidence` de cada item.
- Itens sem confianca definida (caso comum) recebem peso neutro (`1.0`) — sem
  regressoes para dados existentes.
- Formula: `final_score = hybrid_score * (1.0 + alpha * (confidence - 0.5))`
  com `alpha = 0.4`. Isso da ±20% de boost/penalidade nos extremos.
- Apenas itens `Memory` possuem confidence; `DocumentChunk` e `Record` usam
  peso neutro.
- A mudanca fica inteiramente no modulo `query`; `storage` e `lib.rs` nao sao
  alterados.
- `RecallResult` nao ganha novos campos; confidence e sinal interno de ranking.

## Arquivos provaveis

- `crates/hippocore/src/query.rs`
- `crates/hippocore/tests/retrieval_quality.rs`
- `docs/en/SCORE_NORMALIZATION.md`
- `docs/pt-br/SCORE_NORMALIZATION.md`
- `docs/en/STATUS.md`
- `docs/pt-br/STATUS.md`
- `CHANGELOG.md`

## Criterios de aceite

- Uma memoria com `confidence=0.9` que pontua igual a uma memoria com
  `confidence=0.0` em recall hibrido aparece acima dela nos resultados.
- Uma memoria com `confidence=0.0` que pontua igual a uma sem confianca
  definida aparece abaixo dela.
- Itens sem confianca definda mantem seu ranking relativo original.
- O fixture de qualidade de retrieval continua passando em todos os limiares.
- `cargo fmt --all --check`, `cargo test --workspace` e
  `cargo clippy --workspace --all-targets -- -D warnings` passam.

## Fora do escopo

- Confianca em `DocumentChunk` ou `Record`.
- Expor `confidence` como filtro em `RecallRequest`.
- Tornar `alpha` um parametro configuravel do servidor.
- Alterar a struct `RecallResult`.
- Qualquer alteracao em `storage`, `memory` ou `lib.rs`.
