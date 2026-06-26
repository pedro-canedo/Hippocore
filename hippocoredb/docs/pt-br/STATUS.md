# Hippocore DB — Status

_Última atualização: 2026-06-26._

## Implementado por último

**Context data model v0.1**:

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

Incremento anterior: RAG Quality Layer v0.1 com normalização de query, tags,
boost/penalty de Oracle/PostgreSQL, fixture de qualidade e otimização do caminho
vector-only.

## Funcionando

- Store/recall para documentos, memórias e records.
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
- File objects e admin UI ainda não foram implementados.

## Comandos recentes

```bash
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
npm run typecheck
cargo bench -p hippocore
```

## Status de testes

`cargo test --workspace` passa com 59 testes:

- 16 unit;
- 35 integração da biblioteca;
- 1 fixture de qualidade;
- 6 smoke tests de CLI;
- 1 doctest.

## Próxima feature

**File ingestion v0.1** — veja [NEXT_FEATURE.md](NEXT_FEATURE.md).
