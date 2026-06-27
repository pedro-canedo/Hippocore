# Hippocore DB — Escopo do MVP

## Dentro do escopo implementado

- Biblioteca Rust embedded, CLI `hippocore`, servidor HTTP opcional e Control
  Plane local embutido.
- Tenants, collections, documents, chunks, memories, records, files, arestas de
  grafo, auditoria, sources, metadata e resultados de recall/contexto tipados.
- Persistencia local sem banco externo: WAL append-only com checksum, replay no
  startup, snapshots atomicos, compactacao e erros tipados de recovery.
- Indice vetorial cosseno exato e HNSW opcional, busca BM25, recall hibrido,
  filtros por tenant, embeddings deterministas e embeddings do caller.
- Montagem de contexto, validade temporal, supersession, contradicao advisory,
  confidence weighting, expansao de grafo e auditoria de retrieval.
- Records JSON-first, documents chunked, memories e ingestao de arquivos PDF,
  CSV, JSON e texto.
- Subconjunto SQL-like read-only e deliberadamente pequeno para records.
- Fluxos admin para tenants, collections, tables, ingestao, SQL, recall,
  contexto, providers, service keys, prompts, chat e observabilidade.
- Testes deterministas, exemplos, documentacao e benchmarks basicos.

## Fora do escopo imediato

- Cluster distribuido, Raft/consensus ou replicacao multi-node.
- Autorizacao de nivel de producao e identidade hospedada.
- Servicos cloud de embedding obrigatorios ou indexacao somente por GPU.
- Engine SQL completo, joins, expressoes arbitrarias ou PostgreSQL wire protocol.
- Dependencia obrigatoria de banco externo.
- `unsafe` Rust.

## Simplificações deliberadas

- O estado vivo cabe em memoria; indices sao reconstruidos no open.
- Retrieval exato continua sendo o default offline; HNSW e backend opcional.
- Snapshot continua baseado em JSON.
- Records sao JSON-first e ainda nao possuem schema rico ou indice por campo.
- O Control Plane esta migrando para um app React + Vite + TypeScript servido em
  `/console`; o app legado vanilla-JS zero-build permanece em `/admin` ate a
  migracao concluir. O build TypeScript e commitado e embarcado via
  `include_str!`, entao `cargo build` e a imagem Docker continuam sem precisar
  de toolchain Node. Veja ADR-016.
