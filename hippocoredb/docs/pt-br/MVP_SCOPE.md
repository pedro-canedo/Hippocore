# Hippocore DB — Escopo do MVP

## Dentro do escopo implementado

- Biblioteca Rust embedded + CLI `hippocore`.
- Modelo tipado: Tenant, Collection, Document, Chunk, Memory, Record, Source,
  RecallResult, ItemKind.
- Persistência local sem banco externo:
  - WAL JSON-lines append-only;
  - checksums por linha;
  - snapshot atômico;
  - recovery no open;
  - compaction;
  - erros tipados.
- Retrieval:
  - busca vetorial exata por cosine;
  - busca textual BM25;
  - recall híbrido;
  - filtros por tenant, collection, user, tipo, kind e metadata;
  - embedder determinístico local e embeddings fornecidos pelo usuário.
- Dados:
  - documentos chunked;
  - memórias;
  - records JSON-first com projeção de contexto.
- API pública: `open`, `create_tenant`, `create_collection`,
  `store_document`, `remember`, `put_record`, `recall`, `search`, `forget`,
  `delete_document`, `delete_record`, `stats`, `compact`, `close`.
- CLI: init, put-document, remember, put-record, recall, delete/forget, stats,
  inspect, compact.
- Testes determinísticos, exemplo e benchmarks básicos.

## Fora do escopo imediato

- Cluster distribuído.
- Autenticação/autorização de produção.
- Cloud embedders obrigatórios.
- GPU/HNSW/ANN.
- Dashboard web completo.
- SQL/query language completo.
- PostgreSQL wire protocol.
- Banco externo obrigatório.
- `unsafe` Rust.

## Simplificações deliberadas

- Estado vivo em memória.
- Retrieval exato/brute-force.
- Índice reconstruído no open.
- Snapshot JSON ainda é suficiente para o MVP.
- Records não têm schema rico ou índice por campo.
