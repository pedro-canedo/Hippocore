# Próxima Feature

## Nome

**Phase 13 — Filtragem de Metadados e Busca Facetada**

## Por que importa

A Phase 12 deu ao Hippocore a capacidade de ingerir tipos de conteúdo diversos.
A Phase 13 torna a camada de recuperação mais precisa: callers frequentemente
precisam escopar queries por intervalo de datas, fonte específica, banda de
confiança ou chave de metadados personalizada. Sem filtragem nativa, todo
resultado de recall precisa ser pós-processado.

## Escopo mínimo para a Phase 13

1. **Expressões de filtro de metadados**: estender `RecallRequest` e
   `BuildContextRequest` para aceitar expressões de filtro mais ricas além do
   map de correspondência exata `Metadata` atual — ao mínimo operadores `$gte`,
   `$lte`, `$in` e `$ne` em valores de metadados string e numéricos.
2. **Filtro de intervalo de datas**: helpers de atalho em `RecallRequest` para
   `valid_from >= X` e `valid_until <= Y`.
3. **Filtro de intervalo de confiança**: `min_confidence: Option<f32>` em
   `RecallRequest` para excluir memórias abaixo de um limiar.
4. **Contagens de faceta**: novo método `Hippocore::facets(tenant, collection,
   field)` retornando `Vec<(String, usize)>` — valores distintos e contagens de
   documentos para um campo de metadados.
5. **Testes**: ao menos 6 testes de integração cobrindo cada novo operador de
   filtro e facetas.
6. **Docs**: documentação bilíngue.

## Fora de escopo para a Phase 13

- Linguagem de query completa (DSL estilo SQL).
- Filtragem geoespacial.
- Objetos de metadados aninhados.
- Atualizações de índice write-through para scans de filtros grandes.
