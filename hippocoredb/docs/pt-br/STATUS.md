# Hippocore DB — Status

_Última atualização: 2026-06-26._

## Implementado por último

**Context Compiler** — `build_context(query, user, max_tokens)`:

- Novo tipo `BuildContextRequest`: tenant, query, `max_tokens` (padrão 2048),
  `top_k_candidates` (padrão 20), `mode` (padrão `Hybrid`), `collection` e
  `metadata_filter` opcionais.
- Novo tipo de retorno `ContextBlock`: `text` (string pronta para LLM),
  `token_count` (1 token ≈ 4 bytes), `items_included: Vec<ContextItem>`,
  `items_dropped`.
- `ContextItem` carrega `id`, `kind`, `score`, `token_count` e um `snippet`
  de 120 chars.
- `Hippocore::build_context(req)` recorda até `top_k_candidates` itens,
  ordena por score decrescente, então preenche o budget de tokens greedily —
  itens que excederiam `max_tokens` são contados em `items_dropped`.
- Cada item formatado como `[<kind>:<id>]\n<text>`, blocos separados por `\n\n`.
- CLI: `build-context --tenant <t> --query "..." [--max-tokens 2048]
  [--top-k 20] [--mode hybrid] [--collection c] [--json]`.
- 3 novos testes de integração: enforcement de budget, ordenação por score +
  proveniência, consistência de `token_count`.
- Sem novas dependências de crate. 85 testes no total.

Incremento anterior: **Temporal Truth Layer spec completa** — relações supersedes / contradicts:

- `Memory` ganha `supersedes: Vec<String>`, `contradicts: Vec<String>` e
  `superseded_by: Option<String>` (todos com `#[serde(default)]` para compatibilidade
  retroativa com o WAL existente).
- `RecallResult` ganha `contradictions: Vec<String>` (ids de contradições retornados
  como aviso nos resultados de recall).
- `RememberRequest` ganha os campos `supersedes` e `contradicts`; `RecallRequest`
  ganha `include_superseded: bool` (padrão `false`).
- `remember` valida que cada id em `supersedes`/`contradicts` existe no mesmo tenant
  e marca cada memória substituída com `superseded_by = Some(new_id)` no WAL
  imediatamente (durável).
- `Filter::matches` exclui memórias supersedidas salvo `include_superseded = true`.
- `IndexEntry` carrega `superseded: bool` e `contradicts: Vec<String>` para filtragem
  rápida sem carregar a memória completa.
- CLI: `remember --supersedes <id>...`, `remember --contradicts <id>...`,
  `recall --include-superseded`.
- 5 novos testes de integração: exclusão por supersedure, flag include-superseded,
  contradições no resultado, erro de validação para id desconhecido, durabilidade
  após restart.
- Sem novas dependências de crate. 82 testes no total.

Incremento anterior: **Benchmark regression guard**:

- Adicionado `examples/bench_regression.rs`: guard de latência autônomo para
  recall híbrido, vetorial e textual com 500 memórias (50 iterações medidas após
  5 de warmup).
- `benches/baseline.json` commitado com latências médias atuais (~430–470 µs na
  máquina de desenvolvimento).
- `HIPPO_BENCH_UPDATE=1 cargo run --release --example bench_regression` reescreve
  o baseline; sem essa variável o comando lê o baseline e sai com código não-zero
  se qualquer benchmark regredir além de `HIPPO_BENCH_THRESHOLD` (padrão 20%).
- Completa o item final da Fase 5: benchmark regression guard.
- Sem novas dependências de crate.

Incremento anterior: **Temporal Truth Layer v0.1**:

- Adicionados `valid_from: Option<i64>` e `valid_until: Option<i64>` em `Memory`
  e `Document` (epoch ms; `None` = sem restrição). Campos com `#[serde(default)]`
  para que entradas existentes sejam recuperadas como sempre-válidas.
- `RememberRequest` e `StoreDocumentRequest` ganham os mesmos campos.
- `RecallRequest` ganha `as_of: Option<i64>`. Padrão (`None`) resolve para o
  instante atual na query, excluindo entradas expiradas por padrão.
- Flags de CLI: `recall --as-of <ms>`, `remember --valid-from / --valid-until`,
  `put-document --valid-from / --valid-until`.
- Filtro temporal em `Filter::matches`; chunks de documentos herdam a janela de
  validade do documento pai.
- Compatibilidade retroativa: entradas legadas sem campos temporais decodificam
  com `None/None` (sempre válidas) via serde default — nenhuma migração necessária.
- 5 novos testes de integração: filtro future-valid, expiração, query as-of
  passado, compatibilidade retroativa de entradas legadas, herança de validade
  por chunks.
- Sem novas dependências de crate.
- 76 testes no total.

Incremento anterior: **eval-quality CLI**:

- Adicionado subcomando `eval-quality`: lê um arquivo JSON de fixture de
  retrieval, alimenta memórias em um banco temporário (ou indicado via `--db`),
  executa cada cenário e reporta hit@1, hit@k, MRR por cenário e agregado.
- `--json` emite relatório legível por máquina com campos `scenarios`,
  `aggregate`, `thresholds`, `threshold_failures` e `pass`.
- Sai com código não-zero quando qualquer threshold não é atendido; saída humana
  marca cenários com falha como `FAIL` e imprime violações de threshold.
- `--db <path>` permite avaliação contra um banco existente.
- `--top-k` configura K (padrão 5).
- Usa o mesmo formato de fixture JSON do teste interno `retrieval_quality`.
- Sem novas dependências de crate.
- 2 novos smoke tests CLI (caminho de sucesso + caminho de falha deliberado).
- 71 testes no total.

Incremento anterior: **RRF Hybrid Fusion**:

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

`cargo test --workspace` passa com 85 testes:

- 18 unit (embedder/chunker, cosine/index/BM25, normalização de query + RRF, CRC32 + WAL);
- 52 integração da biblioteca (incl. temporal truth, supersedure, context compiler);
- 1 fixture de qualidade de retrieval;
- 12 smoke tests de CLI;
- 1 doctest (lib.rs quickstart);
- 1 bench_regression example.

## Próxima feature

**RAG Audit Engine** — rastreamento de proveniência, scores de confiança por item
e log de auditoria no momento da query (Fase 9). Veja [NEXT_FEATURE.md](NEXT_FEATURE.md).
