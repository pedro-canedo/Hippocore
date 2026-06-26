# Supersession Lifecycle Tests v0.1

## Visão geral

Uma suite de testes dedicada cobrindo o ciclo de vida completo da substituição
de memórias: uma nova memória substitui uma mais antiga via o campo `supersedes`,
marcando a memória antiga como substituída para que seja ocultada do recall
padrão mas preservada para o histórico.

Nenhum código de produção foi alterado; esta feature é puramente testes aditivos.

## Arquivo de testes

`crates/hippocore/tests/supersession.rs`

## O que é testado

| Teste | Cenário |
|---|---|
| `superseded_memory_hidden_from_default_recall` | Memória A substituída por B; A ausente do recall, B presente |
| `superseded_memory_visible_with_include_superseded` | `include_superseded = true` retorna A e B |
| `supersession_linkage_is_bidirectional` | `A.superseded_by = id_b` e `id_a ∈ B.supersedes` |
| `supersession_survives_wal_compaction` | Compactação WAL + reabertura fria; todos os invariantes preservados |
| `forgetting_superseding_memory_does_not_restore_superseded` | Deletar B mantém `superseded_by` de A intacto (tombstone durável) |

## Invariantes de design confirmados

- `superseded_by` é um **tombstone durável**: uma vez definido, persiste mesmo
  que a memória substituta seja posteriormente deletada. Isso preserva o histórico
  do evento de substituição e evita a reativação silenciosa de memórias
  desatualizadas.
- O flag `superseded` no índice de retrieval é derivado de
  `superseded_by.is_some()`, então o filtro se aplica corretamente mesmo após
  compactação WAL e recuperação.
