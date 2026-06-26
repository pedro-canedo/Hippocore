# Ingestão Multimodal — Phase 12

## Visão geral

A Phase 12 estende `StoreDocumentRequest` com um campo `content_type` e um campo
de payload binário `raw`, permitindo ao banco de dados ingerir conteúdo além de
texto simples. Um novo módulo `ingest` roteia documentos pelo extrator apropriado
antes do caminho normal de chunking/embedding.

## Novos campos em `StoreDocumentRequest`

| Campo | Tipo | Descrição |
|---|---|---|
| `content_type` | `Option<String>` | Hint de MIME type. `None` = texto simples. |
| `raw` | `Option<Vec<u8>>` | Payload binário (obrigatório para PDF). |

## Novos construtores de conveniência

### `StoreDocumentRequest::from_pdf(tenant, collection, bytes)`

Constrói uma requisição com `content_type = "application/pdf"` e `raw = bytes`.
O campo `text` fica vazio — o texto é extraído do PDF durante a ingestão.

### `StoreDocumentRequest::from_code(tenant, collection, text, language)`

Constrói uma requisição com `content_type = "text/x-<language>"`. Usa o chunker
de código heurístico em vez do chunker padrão por tokens.

## Tipos de conteúdo suportados

### `application/pdf`

Requer bytes em `raw`. O texto é extraído via `pdf-extract` (Rust puro, sem
dependências C). As páginas são extraídas como uma string concatenada única.

Se a extração falhar (PDF malformado), um `HippocoreError::Validation` é
retornado — nunca um panic.

### `text/x-<linguagem>` e `application/x-<linguagem>`

Chaves de linguagem suportadas: `rust`, `python`, `javascript`, `typescript`,
`js`, `ts`, `go`, `java`, `kotlin`.

O chunker heurístico divide em linhas não-indentadas que começam com palavras-
chave comuns de declaração (ex: `fn `, `def `, `class `, `func `). Segmentos
são mesclados até preencher o budget de tokens; segmentos muito grandes são
divididos pelo chunker padrão por tokens.

Chaves de linguagem não reconhecidas recaem no chunker padrão por tokens.

## Implementação

- `crates/hippocore/src/ingest.rs` — novo módulo:
  - `extract_pdf(bytes: &[u8]) -> Result<String>`
  - `chunk_code(text, language, chunk_tokens) -> Vec<String>`
  - `is_pdf(content_type) -> bool`
  - `code_language(content_type) -> Option<String>`
  - `normalise_mime(content_type) -> String`
- `Hippocore::store_document` despacha por `content_type` antes de construir os chunks.
- Novo método privado `build_chunks_code` usa `ingest::chunk_code`.

## Arquivo de testes

`crates/hippocore/tests/multimodal_ingest.rs` — 8 testes de integração:
- `plain_text_document_produces_chunks`
- `rust_code_document_splits_at_fn_boundaries`
- `python_code_document_splits_at_def_boundaries`
- `unknown_language_falls_back_to_token_chunker`
- `code_document_recalled_by_function_name`
- `pdf_text_is_extracted_and_chunked`
- `missing_raw_bytes_for_pdf_returns_error_not_panic`
- `content_type_stored_in_document_text_field`

Mais 6 testes unitários em `ingest.rs` para roteamento MIME e lógica de
fronteiras do chunker de código.
