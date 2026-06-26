# Valid-Window Recall Tests v0.1

## Visão geral

Uma suite de testes dedicada verificando que o filtro de validade temporal
funciona corretamente para memórias com timestamps `valid_from` / `valid_until`.
Memórias expiradas ou ainda-não-válidas devem ser excluídas do recall no
momento atual; queries backdatadas com `as_of` devem recuperar memórias que
eram válidas naquele momento passado.

Nenhum código de produção foi alterado; esta feature é puramente testes aditivos.

## Arquivo de testes

`crates/hippocore/tests/valid_window.rs`

## O que é testado

| Teste | Cenário |
|---|---|
| `expired_memory_excluded_from_recall` | `valid_until` no passado → excluída |
| `future_valid_memory_included_in_recall` | `valid_until` no futuro → incluída |
| `not_yet_valid_memory_excluded_from_recall` | `valid_from` no futuro → excluída |
| `as_of_backdating_retrieves_past_valid_memory` | `as_of` dentro da janela → incluída; no tempo atual → excluída |
| `memory_with_no_validity_window_always_included` | Sem `valid_from`/`valid_until` → sempre incluída |

## Timestamps usados

- `FAR_PAST = 1_000_000_000_000` ms (2001-09-08) — com certeza expirado.
- `FAR_FUTURE = 9_000_000_000_000` ms (~ano 2255) — com certeza não expirado.

Ambas as constantes evitam depender de `SystemTime::now()`, então os testes
permanecem determinísticos independentemente de quando são executados.

## Fora de escopo

- Expiração / evicção automática de memórias expiradas.
- Mudanças no código de produção.
- Modo server.
