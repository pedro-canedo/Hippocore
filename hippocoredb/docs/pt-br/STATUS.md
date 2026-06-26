# Hippocore DB — Status

_Última atualização: 2026-06-26._

## Implementado por último

**Fase 10 — Graph Memory v0.1**:

- Novo modelo público `GraphEdge` e `AddGraphEdgeRequest`.
- Novas APIs duráveis: `add_graph_edge`, `list_graph_edges`,
  `delete_graph_edge` e `graph_neighbors`.
- Novas operações no WAL/snapshot: `PutGraphEdge` e `DeleteGraphEdge`.
- Endpoints de aresta são validados por tenant, tipo e id; endpoints ausentes ou
  ambíguos são rejeitados com erro tipado de validação.
- Deletar memórias, records, documentos ou arquivos importados remove arestas
  que tocam endpoints deletados de forma determinística.
- Novos comandos CLI:
  - `add-edge --tenant <t> --from-kind <kind> --from-id <id> --to-kind <kind>
    --to-id <id> --relation <name> [--json]`
  - `list-edges --tenant <t> [--from-id <id>] [--json]`
  - `delete-edge --tenant <t> --id <edge_id>`
- `ContextItem` e `AuditItem` agora expõem `related_item_ids` para vizinhos
  diretos no momento de `build_context`.
- Documentação adicionada em `docs/en/GRAPH_MEMORY.md` e
  `docs/pt-br/GRAPH_MEMORY.md`.
- 6 novos testes de integração no core e 1 novo smoke test CLI. 112 testes no
  total.

Anterior: **Fase 9 — RAG Audit Engine**:

- Novos tipos públicos de auditoria: `AuditRecord` e `AuditItem`.
- Cada `Hippocore::build_context(req)` anexa um registro JSON-lines em
  `<data_dir>/audit.log`.
- Os registros capturam timestamp, tenant, query, modo, collection opcional,
  orçamento de tokens solicitado, token count final, quantidade dropada, ids dos
  itens recuperados, latência da montagem de contexto, tipos, coleções, scores
  vetorial/textual/final, confiança, tokens por item e se cada item entrou no
  contexto final.
- Nova API `Hippocore::query_audit(from_ms, to_ms, tenant_id)` reproduz registros
  de um tenant em uma janela inclusiva de epoch ms.
- Novo comando CLI:
  `hippocore audit --db <path> --tenant <t> --from <ms> --to <ms> [--json]`.
- `audit --json` emite JSON parseável como `Vec<AuditRecord>`.
- O histórico de auditoria é append-only e separado do WAL de mutações; ele
  sobrevive a reopen sem alterar o recovery do estado.
- Documentação adicionada em `docs/en/RAG_AUDIT_ENGINE.md` e
  `docs/pt-br/RAG_AUDIT_ENGINE.md`.
- 3 novos testes de integração no core e 1 novo smoke test CLI. Os testes já
  existentes de confiança/rating cobrem os critérios de aceite de `rate_memory`.
  105 testes no total.

Anterior: **Fase 6 — TUI interativo (`hippocore studio`)**:

- Novo subcomando `hippocore studio [--db <path>]` que abre uma interface de terminal completa (ratatui).
- Quatro abas navegáveis: **Tenants** (tenants + coleções), **Memories** (busca ao vivo em todos os tenants), **Documents** (todos os documentos armazenados), **Stats** (estatísticas do banco).
- Teclas de atalho: `1`–`4` ou `Tab`/`Shift-Tab` para trocar de aba; `↑↓` / `jk` para rolar; `/` para abrir busca na aba Memories; `Enter` para executar; `Esc` para cancelar; `r` para atualizar dados; `q` ou `Ctrl-C` para sair.
- Busca de memórias chama `recall()` em todos os tenants e exibe `[score] tenant/collection — trecho…`.
- Refresh re-consulta o banco para tenants, documentos e estatísticas.
- `ratatui 0.29` e `crossterm 0.28` adicionados como dependências do workspace e do crate CLI.
- TUI implementado em `crates/hippocore-cli/src/tui.rs`; entry point binário em `main.rs` intercepta `studio` antes de delegar os demais subcomandos para `hippocore::cli`.
- 1 novo smoke test CLI (`cli_studio_exits_without_panic`) — verifica ausência de panic em ambiente não-TTY. 105 testes no total.
- Todos os quality gates passam: `cargo fmt`, `cargo test --workspace`, `cargo clippy -D warnings`.

Anterior: **Fase 4 — Trait `VectorIndex` plugável + HNSW**, **Fase 7 — Resolução por confiança** e **Fase 2 — Group-commit / batch writes**:

### Fase 7: Resolução ciente de fonte e confiança

- Campo `confidence: Option<f32>` adicionado a `Memory`, `RecallResult`, `ContextItem` e `RememberRequest`. Todos são `#[serde(default)]` — entradas existentes no WAL recuperam como `None` (sem avaliação).
- `Memory::validate()` exige `confidence ∈ [0.0, 1.0]`.
- Nova API `Hippocore::rate_memory(tenant, collection, id, confidence)` — avaliação humana; persiste via WAL `PutMemory`.
- `IndexEntry` carrega `confidence` e propaga via `build_result` para `RecallResult`.
- `build_context` aplica reordenação por confiança quando há contradições entre candidatos: `effective_score = recall_score × 0.7 + confidence × 0.3`. Itens sem confiança usam `confidence = 0.5` (neutro).
- `ContextItem` ganha `confidence: Option<f32>` para introspecção pelo chamador.
- CLI: `remember --confidence <f32>` e novo subcomando `rate-memory --tenant --collection --id --confidence`.
- 6 novos testes de integração: armazenamento+recall de confiança, rejeição fora de range, persistência de `rate_memory`, avaliação inválida, erro not-found, ordenação por conflito no `build_context`.

### Fase 4: Trait VectorIndex plugável + HNSW do zero

- Trait `VectorIndex` em `index.rs`: `insert(key, embedding)`, `remove(key)`, `knn(query, k, ef) -> Vec<EntryId>`.
- `BruteForceVectorIndex`: scan cosine exato O(n); backend padrão para corretude.
- `HnswVectorIndex`: encapsula `HnswIndex` do novo módulo `hnsw.rs`; busca ANN sub-linear.
- `HnswIndex` (Rust puro, sem dependências externas): grafo multi-camada, busca por beam com dois heaps (W = min-heap, C = max-heap), soft deletes, amostragem de nível geométrica `level = floor(-ln(U) × mL)`, conexões bidirecionais com poda.
- `Index` ganha fábrica `with_vector_backend(kind: VectorIndexKind)`; inserts/removes propagam para o backend vetorial automaticamente.
- `Index::knn(query, k, ef)` delega para o backend ativo.
- `Config::vector_index: VectorIndexKind` (padrão `BruteForce`) — chamadores selecionam o backend na abertura.
- `query::execute` usa `index.knn()` com `KNN_OVER_FETCH=8×` para corretude de pós-filtro.
- `VectorIndexKind` re-exportado da raiz do crate.
- 4 novos testes unitários HNSW + 3 novos testes de integração.
- Sem novas dependências de crate. 104 testes no total.

### Fase 2: Group-commit / batch writes

- `Storage::append_many(ops: &[Operation])` — serializa todas as ops em um único buffer, um único `write_all`, um único `sync_all` condicional. Um fsync para N ops.
- `Hippocore::remember_many(reqs: Vec<RememberRequest>)` — valida todos os requests primeiro, constrói o vetor de ops, chama `append_many`, aplica estado e índice em loop. Compactação automática no fim.
- `Hippocore::store_documents(reqs: Vec<StoreDocumentRequest>)` — mesmo padrão de group-commit para documentos.
- 3 novos testes de integração: batch de memórias, batch de documentos, durabilidade após restart.
- Sem novas dependências de crate.

Incremento anterior: **Context Compiler** — `build_context(query, user, max_tokens)`:

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
- Arestas duráveis de grafo entre memórias, records e chunks de documento, com
  consulta de vizinhos diretos e gerenciamento via CLI.
- Vector, BM25 text e hybrid recall.
- Tenant isolation.
- WAL com checksum, snapshot atômico, recovery e compaction.
- Embeddings fornecidos pelo usuário para memórias, queries e chunks.
- Avaliação de qualidade em
  `crates/hippocore/tests/fixtures/retrieval_quality_v01.json`.

## Parcial

- Retrieval usa brute-force por padrão; HNSW opcional existe.
- Arestas de grafo são apenas proveniência na v0.1; ranking/travessia ciente de
  grafo ainda não foi implementado.
- Estado vivo fica em memória e índice é reconstruído no open.
- Records não têm schema rico nem índices por campo.
- Blob storage completo, PDF/OCR e dashboard web admin ainda não foram
  implementados.

## Comandos recentes

```bash
cargo fmt --all
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p hippocore
```

## Status de testes

`cargo test --workspace` passa com 112 testes:

- 22 unit (embedder/chunker, cosine/index/BM25, normalização de query + RRF, CRC32 + WAL, 4 novos testes HNSW);
- 73 integração da biblioteca (incl. temporal truth, supersedure, context compiler,
  batch writes, resolução por confiança, HNSW backend, RAG audit e Graph Memory);
- 1 fixture de qualidade de retrieval;
- 15 smoke tests de CLI (incl. `audit --json`, fluxo de graph edge e
  `cli_studio_exits_without_panic`);
- 1 doctest (lib.rs quickstart);
- 1 bench_regression example.

## Próxima feature

**Graph-Aware Context v0.1** — usar vizinhos diretos do grafo como expansão
opcional/proveniência de contexto sem implementar travessia GraphRAG completa.
Veja [NEXT_FEATURE.md](NEXT_FEATURE.md).
