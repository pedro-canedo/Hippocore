# Hippocore DB — Arquitetura

Hippocore é um banco embedded, single-process, organizado em módulos pequenos.

A arquitetura de longo prazo é banco de dados + camada de projeção de contexto:
documentos, memórias, records e futuros arquivos são armazenados como objetos de
banco, depois projetados em itens de contexto indexáveis para SDKs, agentes e
RAG. Veja [DATA_MODEL.md](DATA_MODEL.md) e
[ADMIN_INTERFACE.md](ADMIN_INTERFACE.md).

## Módulos

| Módulo | Responsabilidade |
|--------|------------------|
| `config` | Configuração do data dir, embedding, chunks, hybrid alpha e sync. |
| `errors` | `HippocoreError` + `Result`. |
| `model` | Entidades tipadas: tenant, collection, document, chunk, memory, record, source, result. |
| `storage` | WAL JSON-lines, snapshot atômico, recovery e compaction. |
| `memory` | Embedder determinístico, chunking e tokenização. |
| `index` | Índice em memória: vetor exato + texto BM25. |
| `query` | Filtros, normalização, tags, fusão de score e `RecallResult`. |
| `cli` | Comandos `init`, `put-document`, `remember`, `put-record`, `recall`, etc. |
| `lib.rs` | Motor `Hippocore` e API pública. |

## Fluxo de escrita

```txt
caller → store_document / remember / put_record
       → validar modelo
       → chunks + embeddings            [documentos]
       → projeção de texto + embedding  [records]
       → storage.append(Operation)      [WAL antes da memória]
       → state.apply(Operation)
       → index.insert(IndexEntry)
```

## Fluxo de leitura

```txt
caller → run_query
       → Filter com tenant obrigatório
       → query::execute
           → normalização de query quando aplicável
           → filtros
           → score vetorial e/ou BM25
           → normalização min-max
           → fusão híbrida
           → sort + top-k + RecallResult
```

## Recovery e compaction

No `open`, o storage carrega `snapshot.json` e reaplica `wal.log`.

`compact()` escreve um snapshot novo de forma atômica e trunca o WAL.

## IndexEntry unificado

Chunks de documentos, memórias e records são projetados em `IndexEntry`. O
modelo persistido permanece distinto e fortemente tipado, mas retrieval usa uma
representação comum.

## Entidades

- `Document`: texto persistido como objeto lógico.
- `Chunk`: fatia pesquisável de documento.
- `Memory`: item atômico de memória.
- `Record`: objeto JSON persistido em namespace de tabela, com projeção textual.
- `IndexedEntry`: projeção em memória usada por recall/search.
- `WAL Entry`: operação durável para recovery, não é pesquisável.

## Isolamento

`tenant_id` é obrigatório no filtro e verificado em toda consulta. Nenhum caminho
de query deve retornar dados de outro tenant.

## Invariantes

- `storage` conhece operações e bytes, não scoring.
- `index`/`query` não tocam disco.
- `Embedder` e `cosine_similarity` não entram em pânico em entrada inválida.
- `unsafe` é proibido.
