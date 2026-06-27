# Decisões de Arquitetura

Este documento registra decisões atuais do Hippocore DB. Ele deve acompanhar a
versão em inglês em `docs/en/DECISIONS.md`.

## ADR-001: Banco embedded e local-first

- **Decisão**: o core roda embedded em processo local.
- **Motivo**: privacidade, simplicidade, testes determinísticos e soberania de
  dados vêm antes de server/cloud.
- **Trade-off**: integração remota e multiusuário ficam para fases futuras.

## ADR-002: WAL JSON-lines + snapshot atômico

- **Decisão**: persistir mutações em `wal.log` e compactar para
  `snapshot.json`.
- **Motivo**: modelo simples, auditável, recuperável e adequado ao MVP.
- **Trade-off**: snapshot JSON único não é formato final de alta escala.

## ADR-003: Embedder determinístico local

- **Decisão**: usar feature hashing determinístico como embedder padrão.
- **Motivo**: testes offline e reprodutíveis; usuários podem fornecer embeddings
  reais.
- **Trade-off**: não é semanticamente rico como modelos externos.

## ADR-004: Fusão híbrida com min-max

- **Decisão**: combinar cosine e BM25 normalizados em `[0,1]`.
- **Motivo**: simples, determinístico e configurável.
- **Trade-off**: scores são relativos ao conjunto candidato.

## ADR-005: Isolamento por tenant na query layer

- **Decisão**: toda query exige `tenant_id` e filtra antes de retornar dados.
- **Motivo**: evitar vazamento entre tenants é inegociável.

## ADR-006: Records JSON-first antes de SQL

- **Decisão**: dados estruturados começam como `Record` JSON com projeção
  textual.
- **Motivo**: entrega valor de banco/contexto sem implementar SQL cedo demais.
- **Trade-off**: filtros por campo e schemas ricos ficam para depois.

## ADR-008: Log de auditoria RAG separado e append-only

- **Decisão**: persistir registros de auditoria de query em
  `<data_dir>/audit.log`, como um objeto JSON por linha, separado de `wal.log` e
  `snapshot.json`.
- **Contexto**: a Fase 9 precisa rastrear chamadas de `build_context` sem tratar
  uma query como mutação durável do estado materializado do banco.
- **Alternativas consideradas**:
  - adicionar auditoria ao WAL de mutações como uma nova `Operation`;
  - armazenar auditoria dentro de `State` e snapshots.
- **Motivo**: eventos de auditoria são histórico operacional, não dados vivos de
  contexto. Um arquivo JSON-lines dedicado mantém o replay simples,
  inspecionável e local-first, sem inflar snapshot nem rebuild de índice.
- **Trade-off**: `audit.log` não é compactado junto com o snapshot. Isso é
  aceitável na Fase 9; retenção/rotação pode vir depois sem alterar recovery.

## ADR-009: Graph Memory v0.1 usa ids por tenant, não queries de grafo

- **Decisão**: armazenar endpoints de `GraphEdge` como
  `(tenant_id, ItemKind, id)` e rejeitar ids de endpoint ausentes ou ambíguos
  dentro de um tenant.
- **Contexto**: a Fase 10 precisa de relacionamentos duráveis entre memórias,
  chunks de documento e records, mas o MVP evita linguagem de query complexa e
  travessia GraphRAG completa.
- **Alternativas consideradas**:
  - exigir referências qualificadas por collection/table em todo comando CLI;
  - introduzir uma linguagem de referência/query de grafo;
  - armazenar arestas como strings soltas sem validação.
- **Motivo**: endpoints por tenant/kind/id mantêm API e CLI pequenos sem abrir
  mão de correção. Rejeitar ambiguidade é mais seguro do que escolher um item por
  suposição.
- **Trade-off**: usuários com ids duplicados entre collections/tables precisam
  usar ids únicos antes de criar arestas. Uma referência mais rica pode ser
  adicionada depois sem mudar o modelo básico de persistência de `GraphEdge`.

## ADR-010: Contexto ciente de grafo expande apenas vizinhos diretos

- **Decisão**: `build_context` pode incluir opcionalmente vizinhos diretos de
  `GraphEdge` dos itens recuperados via `BuildContextRequest::include_related` e
  `related_limit`. Itens expandidos são marcados como `graph_expanded` no
  contexto e na auditoria; itens recuperados seguem como `recalled`, e itens
  considerados que não couberem no budget aparecem como `not_included` na
  auditoria.
- **Contexto**: Graph Memory v0.1 registra relacionamentos duráveis e expõe ids
  de vizinhos diretos como proveniência. O próximo incremento útil é permitir que
  chamadores usem essas relações na montagem de contexto sem transformar o MVP
  em um motor de query de grafo.
- **Alternativas consideradas**:
  - expandir vizinhos automaticamente em toda chamada de `build_context`;
  - aplicar boost de score de recall com base em arestas;
  - fazer travessia multi-hop ou planejamento estilo GraphRAG.
- **Motivo**: expansão direta opt-in mantém o comportamento default de recall,
  preserva isolamento por tenant, deixa o budget de tokens como árbitro final e
  entrega contexto relacionado útil para RAG sem adicionar um novo sistema de
  ranking.
- **Trade-off**: itens expandidos usam scores neutros porque não foram
  selecionados pelo recall. A expansão direta pode melhorar completude, mas não
  prova relevância além da aresta armazenada; ranking ciente de grafo e
  travessia multi-hop continuam como trabalho futuro.

## ADR-011: Control Plane permanece zero-build enquanto fluxos backend estabilizam

- **Decisão**: o Control Plane em `/admin` continua como HTML/CSS/JavaScript
  estatico embutido e servido por `hippocore-server`, mas passa a ser
  organizado como app shell com paginas por dominio e helpers reutilizaveis de
  UI.
- **Contexto**: a UI admin precisa virar uma superficie clara para
  desenvolvedores, enquanto os contratos backend de SQL/import/catalogo de
  tabelas ainda estao evoluindo.
- **Alternativas consideradas**:
  - migrar imediatamente para Vite/React/TypeScript;
  - manter uma tela unica de debug ate todos os fluxos backend existirem.
- **Motivo**: o shell zero-build mantem deploy Docker/local simples e evita
  complexidade de build frontend antes dos contratos backend estabilizarem. As
  paginas por dominio ainda deixam claro o modelo de produto: dados logicos,
  objetos fisicos/internos, fluxos RAG/contexto e operacoes API/admin ficam
  separados.
- **Trade-off**: o codigo UI escrito a mao escala menos que um app com
  framework. Se o estado do Control Plane crescer alem desta fundacao, um build
  frontend empacotado pode substituir os assets estaticos sem mudar o modelo do
  banco core.

## ADR-012: Respostas HTTP de retrieval preservam evidencias para o usuario

- **Decisao**: adapters de resposta do servidor preservam evidencias de
  retrieval necessarias para explicar resultados, comecando por
  `matched_terms`, em vez de reduzir resultados do core a ids, texto e score.
- **Contexto**: o core ja calcula diagnosticos deterministas de retrieval, mas
  o handler HTTP de recall descartava `matched_terms`, impedindo o Control Plane
  e clientes API de explicar matches lexicais.
- **Motivo**: a promessa do Hippocore inclui contexto fundamentado e auditavel.
  Expor evidencia ja calculada e aditivo, nao aumenta o trabalho de retrieval e
  evita que clientes reimplementem analise de query de forma inconsistente.
- **Trade-off**: objetos de resposta ficam um pouco maiores. Novos campos de
  evidencia continuam aditivos para clientes existentes ignorarem sem quebra.
