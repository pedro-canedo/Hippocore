# build_context Smoke Tests v0.1

## Visão geral

Uma suite de testes dedicada verificando que `build_context` monta um bloco de
contexto pronto para LLM a partir dos itens recuperados. Nenhum código de
produção foi alterado; esta feature é puramente testes aditivos.

## Arquivo de testes

`crates/hippocore/tests/build_context_smoke.rs`

## O que é testado

| Teste | Cenário |
|---|---|
| `build_context_produces_non_empty_text` | `text` não vazio quando memories relevantes existem |
| `context_text_contains_memory_snippet` | String de contexto contém palavra reconhecível da memory armazenada |
| `max_tokens_limits_items_included` | Budget apertado inclui menos itens que budget amplo |
| `include_related_adds_graph_neighbours` | Item conectado por grafo aparece em `items_included` quando `include_related = true` |
| `empty_store_returns_empty_context_not_error` | Coleção vazia → `text == ""`, `items_included.is_empty()`, sem erro |

## Formato do contexto

Cada item em `ContextBlock.text` é formatado como: `[<kind>:<id>]\n<text>`,
separado por `\n\n`. Contagem de tokens: 1 token ≈ 4 bytes.
