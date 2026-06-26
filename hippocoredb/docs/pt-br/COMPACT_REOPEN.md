# Compact + Reopen Invariant Tests v0.1

## Visão geral

Uma suite de testes dedicada verificando que `compact()` seguido de reabertura
a frio produz exatamente o mesmo estado observável. Nenhum código de produção
foi alterado; esta feature é puramente testes aditivos.

## O que compact faz

`compact()` dobra todas as entradas do WAL em um snapshot atômico
(`snapshot.json`) e trunca o `wal.log` a zero entradas. A próxima abertura
carrega do snapshot em vez de repetir todo o histórico do WAL.

## Arquivo de testes

`crates/hippocore/tests/compact_reopen.rs`

## O que é testado

| Teste | Cenário |
|---|---|
| `wal_length_is_zero_after_compact` | `stats().wal_entries == 0` imediatamente após `compact()` |
| `collections_survive_compact_reopen` | Todas as coleções presentes antes do compact estão presentes após reabertura |
| `memories_survive_compact_reopen` | Memória recuperável por id e aparece em `recall()` após compact + reabrir |
| `documents_survive_compact_reopen` | Documento recuperável por id após compact + reabrir |
| `multiple_compact_cycles_preserve_state` | Três ciclos compact sucessivos não perdem nenhuma das memórias inseridas |

## Fora de escopo

- Mudanças na estratégia de compactação do WAL.
- Modo server.
- Mudanças no código de produção.
