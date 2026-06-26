# Metadata Filter Regression Suite v0.1

## Visão geral

Uma suite de testes dedicada que exercita o filtro de metadata com
correspondência exata em todos os tipos de item (memories, chunks de documento,
records), semântica AND multi-chave, tratamento de resultado vazio, filtro
escopado por coleção, não-interferência entre tenants e propagação de filtro em
`build_context`.

Nenhum código de produção foi alterado; esta feature é puramente testes aditivos.

## Arquivo de testes

`crates/hippocore/tests/metadata_filter.rs`

## O que é testado

| Teste | Cenário |
|---|---|
| `metadata_filter_exact_match_memories` | Filtro de uma chave em memories; memory não-correspondente excluída |
| `metadata_filter_multi_key_and_semantics` | Filtro de duas chaves (env=prod AND tier=api); itens com apenas uma chave excluídos |
| `metadata_filter_no_match_returns_empty_not_error` | Filtro sem correspondência retorna `Vec` vazio, não erro |
| `metadata_filter_scoped_to_collection` | Filtro combinado com escopo de `collection`; itens de outra coleção excluídos |
| `metadata_filter_records` | Filtro de uma chave em records estruturados |
| `metadata_filter_does_not_cross_tenant_boundary` | Mesmo metadata em dois tenants; consultar um nunca retorna itens do outro |
| `build_context_metadata_filter_respected` | `BuildContextRequest.metadata_filter` propagado ao passo de recall |

## Fora de escopo

- Novos operadores de filtro (prefixo, range, regex).
- Modo server.
- Mudanças no código de produção.
