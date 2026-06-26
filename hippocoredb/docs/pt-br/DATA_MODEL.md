# Hippocore DB — Modelo de Dados e Projeção de Contexto

## Definição de produto

Hippocore DB é um banco local-first onde informações armazenadas podem virar
contexto nativo para sistemas de IA.

Ele deve ser útil como banco de dados: usuários e aplicações precisam inserir,
gerenciar, inspecionar, apagar e recuperar informações. O diferencial AI-native
é que cada forma de dado suportada também pode ser projetada em contexto
pesquisável, citável e auditável para SDKs, agentes e RAG.

## Ideia central

```txt
dado armazenado → projeção de contexto → indexed entry → recall/search/build_context
```

Contexto não deve ser um pré-processamento externo. Projeção de contexto é
responsabilidade nativa do Hippocore DB.

## Entidades atuais

| Entidade | Status | Papel |
|----------|--------|-------|
| `Tenant` | implementado | Fronteira principal de isolamento. |
| `Collection` | implementado | Grupo lógico dentro de um tenant. |
| `Document` | implementado | Texto armazenado como objeto. |
| `Chunk` | implementado | Fatia pesquisável de documento. |
| `Memory` | implementado | Fato/procedimento/nota/evento atômico. |
| `Record` | implementado | Objeto JSON estruturado em namespace de tabela. |
| `FileObject` | implementado | Metadata de arquivo textual importado ligada a documento derivado. |
| `GraphEdge` | implementado | Relacionamento direto durável entre itens de contexto. |
| `IndexedEntry` | interno | Projeção pesquisável de chunk, memory ou record. |
| `WAL Entry` | interno | Registro durável de mutação para recovery. |

## Entidades planejadas

| Entidade | Papel |
|----------|-------|
| `Table` | Hoje é `Record.table`; schema rico fica para o futuro. |
| `ContextItem` | Unidade pública de contexto independente da origem. |
| `ContextTrace` | Futuro registro de auditoria pergunta → retrieval → contexto → resposta. |

## Projeção de contexto

Todo tipo pesquisável deve definir projeção determinística:

- `kind`;
- `text`;
- `embedding`;
- `metadata`;
- `source`;
- `tenant` / `collection`;
- timestamps / version;
- campos futuros de confidence e validade temporal.

Exemplos:

```txt
Memory
  "PostgreSQL usa porta TCP 5432 por padrão"
  → um ContextItem / IndexedEntry

Document
  "Guia longo..."
  → chunks → vários ContextItems / IndexedEntries

Record
  table: "systems"
  json: {"name":"billing-db","engine":"postgresql","port":5432}
  → "table systems record billing-db name billing-db engine postgresql port 5432"

FileObject
  path: "./notes.md"
  media_type: "text/markdown"
  → Document derivado → chunks → IndexedEntries

GraphEdge
  from: memory "postgres-policy"
  to: memory "python-client"
  relation: "mentions"
  → proveniência/vizinho direto (não vira IndexedEntry)
```

## Records estruturados

O primeiro incremento de dados estruturados é JSON-first:

```rust
db.put_record("tenant", "systems", json!({
    "name": "billing-db",
    "engine": "postgresql",
    "port": 5432
}))
```

Comportamento:

- armazenar JSON original de forma durável;
- preservar metadata para filtros/inspeção;
- criar projeção textual para retrieval;
- retornar contexto derivado de record com tipo/source;
- índices por campo e schemas ricos ficam para depois.

## Ingestão de arquivos

Arquivos são objetos de metadata de primeira classe para entradas locais
textuais. Hippocore armazena o objeto de arquivo de forma durável e projeta o
texto extraído em um documento derivado, cujos chunks viram indexed entries
pesquisáveis.

Escopo implementado:

- ingestão de `.txt`, `.md`, `.json`, `.csv`; ✅
- metadata de arquivo: path, nome, media type, checksum CRC32, tamanho em bytes,
  metadata, source, timestamps e version; ✅
- documento derivado (`file:<id>`) e chunks para recall/search; ✅
- API `import_file` / `delete_file` e CLI `import-file` / `delete-file`. ✅

Os bytes originais ainda não são copiados para uma blob store. O estado durável
guarda metadata e o texto extraído via documento derivado. PDF, OCR e extração
multimodal continuam fora do escopo imediato.

## Graph memory

Arestas de grafo são registros duráveis de relacionamento entre itens de
contexto recuperáveis. Elas não criam indexed entries por si só; em vez disso,
permitem que `build_context` e a auditoria exponham ids de itens diretamente
relacionados como proveniência.

Escopo implementado:

- Modelo `GraphEdge` com tenant, endpoints, relação, metadata e timestamps. ✅
- APIs `add_graph_edge` / `list_graph_edges` / `delete_graph_edge` /
  `graph_neighbors`. ✅
- Comandos CLI `add-edge` / `list-edges` / `delete-edge`. ✅
- Limpeza determinística quando um endpoint é deletado. ✅

Ranking ciente de grafo, travessia multi-hop e linguagem de query de grafo ficam
para o futuro.

## Direção da API de contexto

A API deve evoluir de `recall` para operações explícitas:

```rust
db.search_context(query)
db.build_context(query, max_tokens)
```

Saída esperada:

- itens de contexto ranqueados;
- citações/source;
- scores e razões;
- metadata;
- sinalização de contexto insuficiente;
- futuro trace id de auditoria.
- origem de inclusão (`recalled` ou `graph_expanded`) em itens de contexto.

## Não objetivos imediatos

- SQL completo;
- protocolo PostgreSQL;
- HNSW;
- server mode como pré-requisito;
- extração multimodal além de arquivos textuais simples.
