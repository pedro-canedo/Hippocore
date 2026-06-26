# Próxima Feature

## Nome da feature

**Matched Terms v0.1**

## Por que importa

Ao depurar qualidade de recall, operadores precisam saber quais tokens da
query de fato apareceram em cada resultado retornado. O campo `reason` expoe
decomposicao de score mas nao cobertura textual. Adicionar `matched_terms`
permite que o LLM cite quais termos motivaram o match e que desenvolvedores
identifiquem se baixo recall se deve a incompatibilidade vocabular ou a
distancia vetorial.

## Comportamento

- Adicionar `matched_terms: Vec<String>` em `RecallResult` (serde default `[]`).
- Populado em `query.rs`: intersecao dos tokens da query com os tokens do
  texto do resultado. Apenas termos presentes nos dois conjuntos sao listados;
  duplicatas removidas; vazio quando modo for vector puro (sem tokens de query).
- O campo e puramente informativo; nunca afeta ranking ou filtragem.
- Callers existentes que omitem o campo na deserializacao nao sao afetados
  por causa de `#[serde(default)]`.

## Arquivos provaveis

- `crates/hippocore/src/model.rs` (RecallResult)
- `crates/hippocore/src/query.rs` (build_result / execute)
- `docs/en/STATUS.md`
- `docs/pt-br/STATUS.md`
- `CHANGELOG.md`

## Criterios de aceitacao

- `RecallResult` tem `matched_terms: Vec<String>` com default `[]`.
- Para recall hibrido ou textual, termos presentes em query e texto do
  resultado sao listados (minusculo, deduplicados).
- Para recall puramente vetorial, `matched_terms` e vazio.
- Resultados serializados que omitem o campo deserializam sem erro.
- `cargo fmt --all --check`, `cargo test --workspace`, e
  `cargo clippy --workspace --all-targets -- -D warnings` passam.

## Fora de escopo

- Offsets ou posicoes de caracteres.
- Pesos de termos ou scores IDF na lista.
- Passar matched_terms como metadado estruturado para o LLM (escolha do caller).
- Qualquer mudanca em storage ou index.
