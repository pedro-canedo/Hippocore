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
| `IndexedEntry` | interno | Projeção pesquisável de chunk, memory ou record. |
| `WAL Entry` | interno | Registro durável de mutação para recovery. |

## Entidades planejadas

| Entidade | Papel |
|----------|-------|
| `File` | Objeto de arquivo com path/nome/media type/checksum/source. |
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

## Direção para ingestão de arquivos

Arquivos devem virar objetos de primeira classe antes de multimodal avançado.

Escopo inicial:

- `.txt`, `.md`, `.json`, `.csv`;
- metadata de arquivo: path/nome/media type/checksum/source;
- projeção para documentos/chunks ou records;
- PDF/OCR/multimodal ficam para fases futuras.

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

## Não objetivos imediatos

- SQL completo;
- protocolo PostgreSQL;
- HNSW;
- server mode como pré-requisito;
- extração multimodal além de arquivos textuais simples.
