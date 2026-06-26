# Próxima Feature

## Nome

**Document Chunking Tests v0.1** — testes determinísticos verificando que
`store_document` produz chunks corretos e que o recall em nível de chunk
funciona.

## Por que importa

`store_document` divide automaticamente o texto do documento em chunks e
embedding de cada chunk independentemente. Se o chunker falhar, o índice vetorial
recebe zero entradas e o recall silenciosamente não retorna nada. Nenhuma suite
de testes dedicada bloqueia esse caminho.

## Comportamento

Sem novo código de produção. A suite de testes verificará:

1. Armazenar um documento com texto longo produz ao menos 1 chunk (via
   `get_document_chunks`).
2. Recall com uma query substring retorna o id do documento pai.
3. Múltiplos documentos armazenados na mesma coleção produzem seus próprios
   sets de chunks — sem contaminação cruzada.
4. Chunks sobrevivem `compact()` + reabertura a frio.
5. Deletar um documento remove seus chunks de `get_document_chunks`.

## Arquivos

- `crates/hippocore/tests/document_chunking.rs` — novo arquivo de testes.
- `docs/en/DOCUMENT_CHUNKING.md` e `docs/pt-br/DOCUMENT_CHUNKING.md`.
- `docs/en/STATUS.md` e `docs/pt-br/STATUS.md`.

## Critérios de aceite

- Mínimo de 5 testes de integração determinísticos usando `TempDir`.
- Quality gate:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Fora de escopo

- Configuração customizada do chunker.
- Modo server.
- Mudanças no código de produção.
