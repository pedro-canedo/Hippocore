# Próxima Feature

## Nome

**Phase 12 — Armazenamento Multimodal** (texto, PDF, código, imagens)

## Por que importa

A Phase 11 (servidor HTTP) tornou o Hippocore acessível a qualquer linguagem.
A Phase 12 estende o modelo de documento para suportar tipos de conteúdo mais
ricos: PDFs, arquivos de código-fonte e futuramente imagens. Isso fecha a
lacuna entre o Hippocore e pipelines RAG de produção que processam inputs reais.

## Escopo mínimo para a Phase 12

1. **Ingestão de PDF**: aceitar payload de bytes PDF, extrair texto por página,
   chunkar e embeder cada página. Armazenar número de página nos metadados.
2. **Ingestão de arquivo de código**: aceitar arquivos fonte com hint de
   `language`; usar chunking com awareness de tokens respeitando limites de
   função (heurístico, sem tree-sitter).
3. **Roteamento por MIME type**: `StoreDocumentRequest` ganha campo
   `content_type`; a biblioteca core despacha para o extrator apropriado.
4. **Testes**: pelo menos um teste round-trip de PDF e um de arquivo de código.
5. **Docs**: documentação bilíngue para a nova API `content_type`.

## Fora de escopo para a Phase 12

- Imagem / áudio / vídeo (fases posteriores).
- Aceleração GPU.
- Parsing de AST com tree-sitter.
- Extratores hospedados na nuvem.
