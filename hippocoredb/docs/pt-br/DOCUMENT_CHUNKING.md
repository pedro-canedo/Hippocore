# Document Chunking Tests v0.1

## Visão geral

Uma suite de testes dedicada verificando que `store_document` divide e indexa
corretamente o texto do documento em chunks, e que o ciclo de vida dos chunks
(compact + reabrir, deletar) funciona corretamente. Nenhum código de produção
foi alterado; esta feature é puramente testes aditivos.

## Detalhe importante da API

Em `RecallResult`, `id` é o **id do chunk** (não do documento). O id do
documento pai está em `document_id: Option<String>`. Os testes devem comparar
com `result.document_id` ao procurar um documento via seus chunks.

## Arquivo de testes

`crates/hippocore/tests/document_chunking.rs`

## O que é testado

| Teste | Cenário |
|---|---|
| `stored_document_produces_chunks` | `get_document_chunks` retorna ≥1 chunk após armazenar documento |
| `recall_returns_parent_document_id` | Resultados de `recall()` contêm o `document_id` correto |
| `two_documents_produce_separate_chunks` | Ids de chunks de dois documentos são disjuntos |
| `chunks_survive_compact_reopen` | Chunks presentes após `compact()` + reabertura a frio |
| `deleting_document_removes_chunks` | `delete_document` remove todos os seus chunks de `get_document_chunks` |

## Fora de escopo

- Configuração customizada do chunker.
- Modo server.
- Mudanças no código de produção.
