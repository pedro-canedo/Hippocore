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
