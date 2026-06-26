# Hippocore DB — Direção da Interface Administrativa

## Princípio de produto

Hippocore DB deve ser operável por máquinas e por pessoas.

SDKs e agentes precisam de acesso programático ao contexto. Pessoas precisam
inspecionar, inserir, corrigir, apagar, importar, avaliar e auditar os dados que
viram contexto. Um banco de contexto não é confiável se o usuário não consegue
ver e gerenciar o que ele lembra.

## Camadas de acesso

### 1. Core embedded

O core Rust continua sendo a fonte da verdade:

- diretório local de dados;
- WAL/recovery/compaction;
- armazenamento de documentos, memórias, records e arquivos;
- indexação e retrieval;
- API Rust.

Essa camada não deve depender de UI, cloud ou processo servidor.

### 2. Administração via CLI

A CLI é a primeira superfície humana de administração. Ela deve crescer antes de
uma GUI completa:

```bash
hippocore list-tenants
hippocore list-collections
hippocore list-memories
hippocore list-documents
hippocore inspect --json
hippocore recall --debug
hippocore import-file
hippocore export
hippocore eval-retrieval
```

A CLI deve permitir debug local e exemplos sem exigir um servidor.

### 3. Hippocore Studio

A futura UI administrativa deve ser local-first, parecida em espírito com
pgAdmin ou DBeaver, mas desenhada para memória/contexto em vez de SQL.

Nome de trabalho:

```txt
Hippocore Studio
```

Capacidades esperadas:

- navegar tenants, collections, documentos, memórias, records e arquivos;
- inserir/editar/apagar memórias e records estruturados;
- importar arquivos e inspecionar o contexto derivado;
- ver metadata, source, timestamps, version e campos futuros de confidence;
- testar queries de recall e ver top-k;
- mostrar vector score, text score, final score, tags e razão do ranking;
- rodar fixtures de qualidade de retrieval;
- ver stats, WAL, uso de disco e status de compaction;
- exportar/importar dados para fluxos locais.

### 4. Server/API mode

Server mode será útil depois, mas não deve ser pré-requisito para o banco
embedded.

Comando futuro:

```bash
hippocore serve --db ./data
```

Isso poderá servir Studio, SDKs e integrações remotas quando o core estiver mais
maduro.

## Compatibilidade com DBeaver/pgAdmin

Compatibilidade direta com DBeaver ou pgAdmin não é o alvo inicial.

Essas ferramentas assumem SQL/JDBC/PostgreSQL wire protocol. Hippocore não é
SQL-first; seu modelo principal é memória, documentos, records, arquivos,
metadata, contexto e auditoria de retrieval.

Opções de longo prazo:

- subconjunto SQL para records estruturados;
- ponte ODBC/JDBC;
- compatibilidade com protocolo PostgreSQL;
- tabelas virtuais read-only para inspeção.

Isso deve vir depois do modelo de dados, context projection e fluxos de
administração estarem estáveis.

## Recomendação de curto prazo

Construir administração nesta ordem:

1. Melhorar comandos CLI de inspeção/list/import/export.
2. Definir o modelo estável de dados/contexto.
3. Adicionar records estruturados e ingestão de arquivos.
4. Criar Studio local quando houver superfície suficiente para gerenciar.
5. Adicionar server/API mode depois do fluxo embedded estar sólido.
