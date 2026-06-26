# Hippocore DB — Status

_Última atualização: 2026-06-26._

## Implementado por último

**RRF Hybrid Fusion**:

- Substituído `alpha * vector_norm + (1-alpha) * text_norm` por Reciprocal Rank
  Fusion (RRF, k=60) em `SearchMode::Hybrid`.
- RRF é sem parâmetro: `score = 1/(60+rank_v) + 1/(60+rank_t)`, onde cada rank
  é 1-based dentro da lista de candidatos ordenada por vetor ou texto. Itens sem
  sinal em uma dimensão recebem rank penalizado além do conjunto de candidatos.
- `RecallResult.reason` para modo híbrido agora reporta
  `hybrid/rrf(vector_rank=N, text_rank=M)` para transparência.
- `RecallResult.vector_score` e `text_score` ainda carregam os scores brutos de
  cosine e BM25 (não os ranks) para depuração.
- `hybrid_alpha` em `Config` mantido na API para compatibilidade retroativa, mas
  sem efeito no scoring híbrido.
- `SearchMode::Vector` e `SearchMode::Text` inalterados.
- `RRF_K` adicionado como constante pública em `query.rs`.
- Helper `clamp01` removido (não mais utilizado).
- 2 novos testes unitários para a constante RRF e ordenação por score.
- 69 testes passando; thresholds do fixture de qualidade mantidos.

Incremento anterior: **Admin CLI v0.1**:

- Adicionados `list-tenants`, `list-collections`, `list-documents`,
  `list-memories`, `list-records` e `list-files` para exploração completa do
  banco.
- Adicionados `show-document` (com listagem de chunks), `show-memory`,
  `show-record` e `show-file` para inspeção detalhada de objetos.
- Todos os comandos list e show suportam `--json` para saída legível por
  máquina, estável para scripts e integração futura com o Studio.
- `inspect` atualizado para suportar `--json`.
- APIs públicas adicionadas: `list_documents`, `list_memories`, `list_records`,
  `list_files`, `all_collections`, `get_document`, `get_memory`, `get_record`,
  `get_file`, `get_document_chunks`.
- `serde_json` adicionado como dev-dependency em `hippocore-cli` (para parsear
  saída JSON nos testes de integração CLI).
- 3 novos smoke tests CLI cobrindo todos os comandos admin e saída `--json`.

Incremento anterior: **File ingestion v0.1**:

- `FileObject`s agora são objetos duráveis de metadata para arquivos locais
  textuais importados.
- APIs `import_file` / `delete_file` e CLI `import-file` / `delete-file`
  suportam `.txt`, `.md`, `.json` e `.csv`.
- Arquivos importados guardam path, nome, media type, checksum CRC32, tamanho em
  bytes, metadata, source, timestamps e version.
- Texto extraído é projetado em um `Document` derivado (`file:<id>`) cujos
  chunks são indexados e recuperáveis com metadata de arquivo.
- Import, recall, filtro por metadata, projeção JSON, restart/delete, extensão
  não suportada e fluxo CLI têm testes.

Incremento anterior: Context data model v0.1:

- `Record`s JSON estruturados são objetos armazenados de primeira classe em um
  namespace lógico de tabela.
- APIs `put_record` / `delete_record` e CLI `put-record` / `delete-record`
  persistem records usando WAL/snapshot existentes.
- Cada record preserva o JSON original e uma projeção textual determinística
  indexada como `ItemKind::Record`.
- Recall/search pode retornar contexto derivado de record com `record_table`,
  source, metadata, scores e reason.
- Metadata filters, tenant isolation, recovery e delete de records têm testes.
- A documentação dentro de `docs/` agora é bilíngue por regra: todo arquivo deve
  existir em `docs/en/` e `docs/pt-br/`.

Incremento anterior: RAG Quality Layer v0.1 com normalização de query, fixture
de qualidade e otimização do caminho vector-only.

## Funcionando

- Store/recall para documentos, memórias, records e arquivos textuais
  importados.
- Vector, BM25 text e hybrid recall.
- Tenant isolation.
- WAL com checksum, snapshot atômico, recovery e compaction.
- Embeddings fornecidos pelo usuário para memórias, queries e chunks.
- Avaliação de qualidade em
  `crates/hippocore/tests/fixtures/retrieval_quality_v01.json`.

## Parcial

- Retrieval ainda é brute-force.
- Estado vivo fica em memória e índice é reconstruído no open.
- Records não têm schema rico nem índices por campo.
- Blob storage completo, PDF/OCR e admin UI ainda não foram implementados.

## Comandos recentes

```bash
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
npm run typecheck
cargo bench -p hippocore
```

## Status de testes

`cargo test --workspace` passa com 69 testes:

- 16 unit;
- 39 integração da biblioteca;
- 1 fixture de qualidade;
- 18 unit (embedder/chunker, cosine/index/BM25, query normalization + RRF, CRC32 + WAL);
- 10 smoke tests de CLI (incl. comandos admin `list-*` e `show-*` com `--json`);
- 1 doctest.

## Próxima feature

**eval-quality CLI** — veja [NEXT_FEATURE.md](NEXT_FEATURE.md).
