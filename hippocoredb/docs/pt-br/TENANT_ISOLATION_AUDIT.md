# Multi-tenant Query Isolation Audit v0.1

## Visão geral

Esta é uma suite de testes de regressão dedicada que prova que nenhum caminho
de query público (`recall`, `search`, `build_context`, `list_graph_edges`) retorna
dados de um tenant diferente do especificado na requisição, mesmo quando tenants
compartilham nomes de coleção idênticos e armazenam conteúdo semanticamente
idêntico.

Nenhum código de produção foi alterado; esta feature é puramente testes
aditivos.

## Arquivo de testes

`crates/hippocore/tests/tenant_isolation.rs`

## O que é testado

| Teste | Caminho | Modo |
|---|---|---|
| `recall_vector_stays_within_tenant` | `recall()` | Vector |
| `recall_text_stays_within_tenant` | `recall()` | Text |
| `recall_hybrid_stays_within_tenant` | `recall()` | Hybrid |
| `build_context_stays_within_tenant` | `build_context()` | — |
| `build_context_with_related_stays_within_tenant` | `build_context()` | com `include_related = true` |
| `list_graph_edges_stays_within_tenant` | `list_graph_edges()` | — |

## Estratégia de teste

Todos os testes usam um helper compartilhado `two_tenant_db` que:
- Cria dois tenants (`alpha`, `beta`) com nomes de coleção idênticos (`shared`).
- Armazena conteúdo idêntico (mesmo texto) em ambos os tenants.
- Retorna o handle `Hippocore` populado.

Cada teste consulta um tenant e asserta que cada item resultado pertence ao
tenant consultado, e nenhum item do outro tenant vaza.

## Invariante confirmado

`tenant_id` na requisição de query é um limite de isolamento rígido para todos
os caminhos de leitura públicos. Ele é aplicado no módulo `query` e verificado
aqui.
