# Próxima Feature

## Nome

**Compact + Reopen Invariant Tests v0.1** — testes determinísticos verificando
que `compact()` seguido de reabertura a frio produz exatamente o mesmo estado
observável.

## Por que importa

`compact()` dobra todas as entradas do WAL em um snapshot e trunca o WAL a zero.
Se esta operação introduzisse alguma perda silenciosa (ex.: truncar o snapshot
antes do rename atômico completar, ou re-indexar de um estado incompleto), o
banco degradaria silenciosamente sem nenhum teste capturar. O caminho compact +
reabrir é exercido incidentalmente mas não explicitamente bloqueado.

## Comportamento

Sem novo código de produção. A suite de testes verificará:

1. Após `compact()`, `wal_len()` (ou equivalente) reporta zero entradas.
2. Uma reabertura a frio após `compact()` produz a mesma lista `collections()`.
3. Uma reabertura a frio após `compact()` retorna as mesmas memórias em `recall()`.
4. Múltiplos ciclos compact → reabrir não perdem dados.
5. Documentos armazenados antes de `compact()` são recuperáveis após reabertura.

## Arquivos

- `crates/hippocore/tests/compact_reopen.rs` — novo arquivo de testes dedicado.
- `docs/en/COMPACT_REOPEN.md` e `docs/pt-br/COMPACT_REOPEN.md`.
- `docs/en/STATUS.md` e `docs/pt-br/STATUS.md`.

## Critérios de aceite

- Mínimo de 5 testes de integração determinísticos usando `TempDir`.
- Quality gate:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Fora de escopo

- Mudanças na estratégia de compactação do WAL.
- Modo server.
- Mudanças no código de produção.
