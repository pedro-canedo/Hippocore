# Hippocore DB — Status

_Última atualização: 2026-06-26._

## Implementado por último

**MMR (Maximal Marginal Relevance) v0.1**:

- Adicionado `mmr: bool` e `mmr_lambda: f32` em `RecallRequest` (defaults: `false`, `0.5`).
- Quando ativado, resultados sao reranqueados selecionando iterativamente itens
  que maximizam `lambda * relevância - (1-lambda) * max_sim_com_selecionados`.
  Itens sem embeddings sao adicionados ao final.
- `lambda=1.0` equivale a ordenacao por score; `lambda=0.0` a diversidade pura.
- CLI: flags `--mmr` e `--mmr-lambda` em `hippocore recall`.
- HTTP: `{"mmr": true, "mmr_lambda": 0.5}` no body de recall.
- Dois testes unitarios: pick diverso com `lambda=0.5`; ordem por score preservada com `lambda=1.0`.

Anterior: **Matched Terms v0.1**:

- Adicionado `matched_terms: Vec<String>` em `RecallResult` (serde default `[]`).
- Para modos hybrid e text, o campo contem a intersecao dos tokens da query com
  os tokens do texto do resultado (minusculo, deduplicado, ordenado).
- Recall puramente vetorial sempre retorna lista vazia (sem comparacao textual).
- Resultados serializados que omitem o campo deserializam sem erro.
- Dois testes unitarios: intersecao correta; sem sobreposicao retorna vazio.

Anterior: **Document Chunk Deduplication v0.1**:

- Adicionado `dedup_chunks: bool` em `RecallRequest` (padrao `false`).
- Quando ativado, apenas o chunk com maior score por documento pai e retornado.
  Memories, records e outros itens sao sempre mantidos.
- Resultados ja estao ordenados por score quando a deduplicacao ocorre, entao
  a primeira ocorrencia de cada `document_id` e sempre o melhor chunk.
- Exposto como `--dedup-chunks` no CLI e `{"dedup_chunks": true}` no HTTP.
- Dois testes unitarios verificam: dedup mantem o melhor chunk e remove os
  demais; `false` mantem todos os itens sem alteracao.

Anterior: **Recall Min-Score Filter v0.1**:

- Adicionado `min_score: Option<f32>` em `RecallRequest` (padrao `None`).
- Apos confidence weighting e ordenacao, resultados abaixo do limiar sao
  removidos antes da truncagem `top_k`. Limite inferior inclusivo.
- Flag `--min-score` adicionada ao comando `hippocore recall`.
- Endpoint HTTP de recall aceita `{"min_score": 0.5}` no body.
- Tres testes unitarios verificam: limiar remove itens, `None` mantém todos,
  item exatamente no limiar é incluido.

Anterior: **Confidence-Weighted Recall v0.1**:

- Memorias com `confidence` explicito agora influenciam sua posicao de ranking
  em todos os modos de recall (vector, text, hybrid).
- Formula: `final_score = base_score * (1.0 + 0.4 * (confidence - 0.5))`.
  Confianca 1.0 da boost de 20%; confianca 0.0 aplica penalidade de 20%.
- Itens sem confianca definida e itens nao-Memory sao neutros (sem alteracao
  de ranking relativo).
- Mudanca contida em `query.rs`; storage, model e CLI nao foram alterados.
- Tres testes unitarios verificam direcao do weighting, invariancia do caso
  neutro e limites do multiplicador.

Anterior: **Admin CLI v0.1**:

- Adicionados sub-comandos aninhados `hippocore tenants list` e
  `hippocore tenants create`, com criacao explicita de tenant (antes so
  ocorria como side effect de outros comandos).
- Adicionados `hippocore collections list` e `hippocore collections create`.
- Todos os comandos flat existentes continuam funcionando sem alteracao.
- Cinco testes unitarios verificam parse de argumentos e fluxo de
  create/list dos novos sub-comandos.

Anterior: **Admin Data Actions v0.1**:

- Adicionado handler `POST /admin/tenants/:tid/records` usando `put_record`.
- Pagina Ingestion & Recall agora suporta criacao de Memory, Document e Record.
  O tipo Record aceita payload JSON, collection e nome de tabela.
- Import de arquivo via browser permanece Planned e rotulado na UI.
- Dois testes de integracao adicionados: caminho feliz e isolamento de tenant.

Anterior: **Control Plane Foundation v0.1**:

- `/admin` foi reestruturado de uma tela unica estilo debug para um Control
  Plane com sidebar fixa, topbar, selector global de tenant, status de sessao,
  acao de refresh, breadcrumbs, layout responsivo e tema dark moderno.
- Novas paginas por dominio: Dashboard, Data Explorer, SQL Editor, Collections,
  Records, Memories, Documents, Files, Ingestion & Recall, Graph, API Reference,
  Tenants, Service Keys, Observability e Settings.
- Dashboard mostra status do servidor, tenant atual, contagens de objetos,
  tamanho do audit log, diretorio de dados e ultima operacao.
- Data Explorer separa Logical View, Raw JSON e Physical Info, com navegacao de
  tenant/collection e drawer de detalhe.
- SQL Editor usa o endpoint existente `POST /admin/sql` e deixa isolamento por
  tenant explicito.
- Objetos fisicos/internos ficam separados em Records/Memories/Documents/Files/
  Graph e placeholders de Observability para chunks, WAL, snapshot, audit, logs
  e status de indice.
- Testes dos assets admin agora verificam navegacao do Control Plane e a base
  de componentes reutilizaveis.

Anterior: **Phase 12 — Armazenamento Multimodal ✅**:


- `StoreDocumentRequest` ganha `content_type: Option<String>` e
  `raw: Option<Vec<u8>>` (payload binário para PDF).
- Novo módulo `crates/hippocore/src/ingest.rs` com despacho por content-type.
- **PDF** (`application/pdf`): construtor `from_pdf(tenant, col, bytes)`;
  texto extraído via `pdf-extract` (Rust puro, sem deps C). Erros de extração
  retornam `HippocoreError::Validation`, nunca panic.
- **Código** (`text/x-<lang>`): construtor `from_code(tenant, col, text, lang)`;
  chunker heurístico divide em fronteiras de definições top-level. Linguagens:
  rust, python, javascript, typescript, go, java, kotlin. Linguagens não
  reconhecidas recaem no chunker por tokens.
- 8 testes de integração + 6 unitários. **223 testes no total.**
- Documentação em `docs/en/MULTIMODAL_INGEST.md` e `docs/pt-br/`.

Anterior: **Phase 11 — HTTP Server ✅**:

- Novo crate `crates/hippocore-server`: servidor REST/JSON axum 0.8 sobre
  toda a API core. Estado compartilhado `Arc<Mutex<Hippocore>>`.
- `GET /health` (sem auth), `GET /stats`, `POST /tenants`,
  `POST /tenants/:tid/collections`, `POST /tenants/:tid/memories`,
  `DELETE /tenants/:tid/memories/:id`, `POST /tenants/:tid/recall`,
  `POST /tenants/:tid/context`, `POST /tenants/:tid/documents`,
  `POST /tenants/:tid/graph/traverse`.
- Auth via header `X-Api-Key`; requisições não autenticadas retornam 401.
- Subcomando CLI `hippocore serve [--port 8080] [--db ./data] [--api-key KEY]`.
- 7 testes de integração em `crates/hippocore-server/tests/server_integration.rs`.
- **Phase 11 — Server mode 100% completa.**
- Documentação em `docs/en/SERVER.md` e `docs/pt-br/SERVER.md`.
- 8 novos testes (7 integração + 1 doc-test). **209 testes no total.**

Anterior: **GraphRAG Multi-hop Traversal — Phase 10 ✅**:

- Nova API pública: `traverse_graph(TraverseGraphRequest) -> Vec<TraversalNode>`
- Travessia BFS de itens-semente até `max_hops` de profundidade (padrão 2).
- Campos: `tenant_id`, `seed_ids`, `max_hops`, `max_nodes`, `relation_filter`.
- `TraversalNode` retorna `id`, `kind`, `hop`, `via_edge_id`.
- Isolado por tenant: arestas de outros tenants nunca são seguidas.
- 6 novos testes em `crates/hippocore/tests/graph_traversal.rs`.
- **Phase 10 — Graph Memory 100% completa.**
- Documentação em `docs/en/GRAPH_TRAVERSAL.md` e `docs/pt-br/`.
- 6 novos testes. 201 testes no total.

Anterior: **build_context Smoke Tests v0.1**:

- Novo arquivo de testes `crates/hippocore/tests/build_context_smoke.rs` com
  5 testes.
- Cobre: texto não vazio, snippet na string de contexto, budget `max_tokens`
  aplicado, expansão por grafo com `include_related = true`, store vazio
  retorna `text = ""` (sem erro).
- Sem mudanças no código de produção.
- Documentação em `docs/en/BUILD_CONTEXT_SMOKE.md` e
  `docs/pt-br/BUILD_CONTEXT_SMOKE.md`.
- 5 novos testes. 195 testes no total.

Anterior: **Document Chunking Tests v0.1**:

- Novo arquivo de testes `crates/hippocore/tests/document_chunking.rs` com 5
  testes.
- API documentada: `RecallResult.id` é o **id do chunk**; o id do documento
  pai está em `RecallResult.document_id`.
- Cobre: ≥1 chunk produzido, `document_id` no resultado de recall, chunks
  disjuntos entre dois documentos, chunks sobrevivem compact + reabrir, delete
  remove chunks.
- Sem mudanças no código de produção.
- Documentação em `docs/en/DOCUMENT_CHUNKING.md` e
  `docs/pt-br/DOCUMENT_CHUNKING.md`.
- 5 novos testes. 190 testes no total.

Anterior: **Graph Edge Lifecycle Tests v0.1**:

- Novo arquivo de testes `crates/hippocore/tests/graph_edge_lifecycle.rs` com
  5 testes.
- Cobre: aresta adicionada aparece na lista, tipo de relação persiste, aresta
  deletada ausente da lista, arestas sobrevivem compact + reabrir, isolamento
  de tenant.
- Nota documentada: `list_graph_edges` retorna `Vec<GraphEdge>` diretamente
  (não `Result`).
- Sem mudanças no código de produção.
- Documentação em `docs/en/GRAPH_EDGE_LIFECYCLE.md` e
  `docs/pt-br/GRAPH_EDGE_LIFECYCLE.md`.
- 5 novos testes. 185 testes no total.

Anterior: **Record CRUD Tests v0.1**:

- Novo arquivo de testes `crates/hippocore/tests/record_crud.rs` com 6 testes.
- Cobre: armazenar + recuperar por id, aparece no recall, versão incrementa na
  atualização (0 → 1), deleção remove de get e recall, isolamento de tenant em
  lookup por id e recall.
- Invariantes confirmados: versão inicial é 0; `delete_record` é durável.
- Sem mudanças no código de produção.
- Documentação em `docs/en/RECORD_CRUD.md` e `docs/pt-br/RECORD_CRUD.md`.
- 6 novos testes. 180 testes no total.

Anterior: **Compact + Reopen Invariant Tests v0.1**:

- Novo arquivo de testes `crates/hippocore/tests/compact_reopen.rs` com 5 testes.
- Cobre: WAL zerado após compact, coleções / memórias / documentos sobrevivem
  compact + reabertura a frio, múltiplos ciclos compact preservam todo o estado.
- Sem mudanças no código de produção.
- Documentação em `docs/en/COMPACT_REOPEN.md` e `docs/pt-br/COMPACT_REOPEN.md`.
- 5 novos testes. 174 testes no total.

Anterior: **WAL Recovery Tests v0.1**:

- Novo arquivo de testes `crates/hippocore/tests/wal_recovery.rs` com 4 testes.
- Cobre: reabertura a frio com WAL limpo, linha truncada ignorada (sem panic),
  checksum incorreto interrompe replay (entrada ruim não aplicada), estado antes
  do rasgo intacto em `get_memory` e `recall`.
- Formato WAL: `<crc32-hex>\t<json>\n`.
- Sem mudanças no código de produção.
- Documentação em `docs/en/WAL_RECOVERY.md` e `docs/pt-br/WAL_RECOVERY.md`.
- 4 novos testes. 169 testes no total.

Anterior: **Collection CRUD Tests v0.1**:

- Novo arquivo de testes `crates/hippocore/tests/collection_crud.rs` com 6
  testes.
- Descoberta documentada: `create_collection` é **idempotente** (retorna a
  coleção existente em duplicata, sem erro) — seguro usar como upsert.
- Cobre: criação aparece na lista, idempotência, dois tenants compartilham
  nomes, lista isolada por tenant, recall em inexistente retorna vazio,
  round-trip de descrição.
- Sem mudanças no código de produção.
- Documentação em `docs/en/COLLECTION_CRUD.md` e `docs/pt-br/COLLECTION_CRUD.md`.
- 6 novos testes. 165 testes no total.

Anterior: **Valid-Window Recall Tests v0.1**:

- Novo arquivo de testes `crates/hippocore/tests/valid_window.rs` com 5 testes
  usando constantes epoch-ms determinísticas (FAR_PAST=2001, FAR_FUTURE=~ano 2255).
- Cobre: expirado excluído, valid_until futuro incluído, ainda-não-válido excluído,
  backdating `as_of` dentro vs fora da janela, sem janela de validade sempre
  incluído.
- Sem mudanças no código de produção.
- Documentação em `docs/en/VALID_WINDOW_RECALL.md` e
  `docs/pt-br/VALID_WINDOW_RECALL.md`.
- 5 novos testes. 159 testes no total.

Anterior: **Contradiction Advisory Tests v0.1**:

- Novo arquivo de testes `crates/hippocore/tests/contradiction.rs` com 4 testes.
- Confirma: ambas as memórias em contradição aparecem no recall (advisory, não
  filtro); `RecallResult.contradictions` é populado; `build_context` ativa
  re-ranking por confiança quando há contradições (maior confiança vence); sem
  contradições, sem re-ranking e ordenação do recall preservada.
- Sem mudanças no código de produção.
- Documentação em `docs/en/CONTRADICTION_ADVISORY.md` e
  `docs/pt-br/CONTRADICTION_ADVISORY.md`.
- 4 novos testes. 154 testes no total.

Anterior: **Supersession Lifecycle Tests v0.1**:

- Novo arquivo de testes `crates/hippocore/tests/supersession.rs` com 5 testes.
- Cobre: recall padrão oculta memórias substituídas; `include_superseded = true`
  as exibe; linkage é bidirecional; invariantes sobrevivem a compactação WAL +
  reabertura fria; deletar a memória substituta deixa `superseded_by` como
  tombstone durável (memória desatualizada permanece oculta).
- Sem mudanças no código de produção.
- Documentação em `docs/en/SUPERSESSION_LIFECYCLE.md` e
  `docs/pt-br/SUPERSESSION_LIFECYCLE.md`.
- 5 novos testes. 150 testes no total.

Anterior: **Metadata Filter Regression Suite v0.1**:

- Novo arquivo de testes `crates/hippocore/tests/metadata_filter.rs` com 7
  testes.
- Cobre: correspondência exata em memories, semântica AND multi-chave, resultado
  vazio (sem erro), filtro escopado por coleção, filtro de records, não-
  interferência entre tenants e propagação de filtro em `build_context`.
- Sem mudanças no código de produção.
- Documentação em `docs/en/METADATA_FILTER_REGRESSION.md` e
  `docs/pt-br/METADATA_FILTER_REGRESSION.md`.
- 7 novos testes. 145 testes no total.

Anterior: **Multi-tenant Query Isolation Audit v0.1**:

- Novo arquivo de testes dedicado `crates/hippocore/tests/tenant_isolation.rs`.
- 6 asserções de isolamento cobrindo todos os caminhos de leitura públicos:
  `recall()` (Vector, Text, Hybrid), `build_context()` (básico e com
  `include_related = true`) e `list_graph_edges()`.
- Testes usam dois tenants com nomes de coleção idênticos e conteúdo
  semanticamente idêntico, garantindo que a camada de query filtra
  rigidamente por `tenant_id`.
- Sem mudanças no código de produção.
- Documentação em `docs/en/TENANT_ISOLATION_AUDIT.md` e
  `docs/pt-br/TENANT_ISOLATION_AUDIT.md`.
- 6 novos testes. 138 testes no total.

Anterior: **Score Normalization v0.1**:

- `build_context` agora normaliza scores brutos de recall híbrido para
  `[0.0, 1.0]` antes das mesclagens de Temporal Decay e Graph-Aware Ranking
  quando qualquer peso de mesclagem é não-zero. O candidato de maior score
  recebe score normalizado `1.0`; a ordenação é preservada.
- A normalização é completamente ignorada quando ambos os pesos de mesclagem
  são `0.0` (padrão), sem overhead e sem mudança de comportamento.
- Documentação em `docs/en/SCORE_NORMALIZATION.md` e
  `docs/pt-br/SCORE_NORMALIZATION.md`.
- 2 novos testes de integração. 132 testes no total.

Anterior: **Temporal Decay v0.1**:

- `Config` ganha `temporal_weight: f32` (padrão `0.0`) e
  `temporal_decay_days: f32` (padrão `30.0`).
- `build_context` aplica um viés de recência opcional após o recall híbrido:
  `decay = exp(-age_days / decay_days)` mesclado ao score efetivo
  (`recall × (1−w) + decay × w`). Itens Memory usam `created_at`; chunks de
  documentos usam `updated_at` do documento pai; records usam `updated_at`.
  Itens com timestamp não resolvível recebem decaimento neutro de `0.5`.
- O Temporal Decay é executado antes do Graph-Aware Ranking; as duas features
  se compõem.
- `temporal_weight` fora de `[0.0, 1.0]` ou `temporal_decay_days ≤ 0` com
  peso não-zero retorna `HippocoreError::Validation` antes de qualquer I/O.
- Documentação em `docs/en/TEMPORAL_DECAY.md` e `docs/pt-br/TEMPORAL_DECAY.md`.
- 4 novos testes de integração. 130 testes no total.

Anterior: **Graph-Aware Ranking v0.1**:

- `Config` ganha `graph_rank_weight: f32` (default `0.0` = desativado; sem
  mudança de comportamento para bancos existentes).
- `build_context` aplica um bônus de conectividade opcional após o recall
  híbrido: para cada candidato, conta quantos de seus vizinhos diretos no
  grafo também aparecem no conjunto de candidatos, normaliza para `[0.0, 1.0]`
  e mescla ao score efetivo (`recall × (1−w) + connectivity × w`).
- Candidatos são re-ordenados pelo score efetivo após o passo.
- `w = 0.0` (padrão) produz ordenação idêntica ao recall simples.
- Passar `graph_rank_weight` fora de `[0.0, 1.0]` retorna
  `HippocoreError::Validation` antes de qualquer I/O.
- Documentação em `docs/en/GRAPH_AWARE_RANKING.md` e
  `docs/pt-br/GRAPH_AWARE_RANKING.md`.
- 3 novos testes de integração. 126 testes no total.

Anterior: **Audit Retention v0.1**:

- `Config` ganha `audit_max_records: usize` e `audit_max_bytes: u64` (ambos
  com default `0` = ilimitado; sem mudança para bancos existentes).
- Novo struct público `AuditRetentionSummary`: `records_kept`, `records_removed`,
  `bytes_before`, `bytes_after`.
- Novo `Hippocore::compact_audit(max_records, max_bytes)` — lê `audit.log`,
  mantém os registros mais recentes que satisfazem ambas as restrições, escreve
  em `audit.tmp`, fsync, renomeia atomicamente. `Some(0)` desativa aquela
  restrição para a chamada; `None` usa o valor configurado.
- `append_audit_record` agora aplica retenção automaticamente após cada append
  de `build_context` quando `audit_max_records > 0` ou `audit_max_bytes > 0`
  (best-effort).
- `DatabaseStats` ganha `audit_log_bytes: u64` e `audit_records: usize`;
  `Display` os imprime.
- CLI: novo subcomando `compact-audit --db <path> [--max-records <n>]
  [--max-bytes <n>] [--json]`.
- Documentação em `docs/en/AUDIT_RETENTION.md` e
  `docs/pt-br/AUDIT_RETENTION.md`.
- 5 novos testes de integração no core + 1 novo smoke test CLI. 123 testes no
  total.

Anterior: **Graph-Aware Context v0.1**:

- `BuildContextRequest` ganha `include_related: bool` e
  `related_limit: usize`; defaults mantêm o comportamento existente.
- `build_context` pode incluir vizinhos diretos do grafo dos itens recuperados
  quando couberem no budget de tokens.
- Itens expandidos por grafo são isolados por tenant, limitados por
  `related_limit` e deduplicados se já vieram pelo recall.
- Novos rótulos `ContextItemSource`: `recalled`, `graph_expanded`,
  `not_included`.
- `ContextItem` e `AuditItem` agora expõem `inclusion_source`; auditoria marca
  candidatos expandidos por grafo que não couberam como `not_included`.
- CLI `build-context` ganha `--include-related` e `--related-limit <n>`.
- Documentação adicionada em `docs/en/GRAPH_AWARE_CONTEXT.md` e
  `docs/pt-br/GRAPH_AWARE_CONTEXT.md`.
- 5 novos testes de integração no core mais cobertura CLI no fluxo de graph
  edge. 117 testes no total.

Anterior: **Fase 10 — Graph Memory v0.1**:

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
- Arestas de grafo podem expandir contexto por vizinhos diretos; ranking ciente
  de grafo e travessia multi-hop ainda não foram implementados.
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

`cargo test --workspace` passa com 123 testes:

- 22 unit (embedder/chunker, cosine/index/BM25, normalização de query + RRF, CRC32 + WAL, 4 novos testes HNSW);
- 83 integração da biblioteca (incl. temporal truth, supersedure, context compiler,
  batch writes, resolução por confiança, HNSW backend, RAG audit, Graph Memory,
  Graph-Aware Context e Audit Retention default/max-records/max-bytes/
  query-after-compaction/auto-retention-from-config);
- 1 fixture de qualidade de retrieval;
- 16 smoke tests de CLI (incl. `audit --json`, fluxo de graph edge,
  `compact-audit --json` e `cli_studio_exits_without_panic`);
- 1 doctest (lib.rs quickstart);
- 1 bench_regression example.

## Próxima feature

**Phase 11 — HTTP Server** (`crates/hippocore-server`): API REST baseada em
axum expondo toda a biblioteca core via HTTP, com autenticação por API key,
corpos JSON e subcomando CLI `hippocore serve`.
Veja [NEXT_FEATURE.md](NEXT_FEATURE.md).
