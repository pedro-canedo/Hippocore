# Próxima Feature

## Nome

**WAL Recovery Tests v0.1** — testes determinísticos verificando o tratamento
de gravação rasgada no WAL na abertura do banco de dados.

## Por que importa

`CLAUDE.md` e `docs/en/ARCHITECTURE.md` documentam que "uma linha final rasgada
[no WAL] é ignorada, não fatal". Esse invariante é crítico para a segurança dos
dados (uma perda de energia após uma gravação parcial não deve corromper o banco),
mas não há testes de regressão bloqueando isso. Uma refatoração futura da camada
de armazenamento poderia mudar silenciosamente o comportamento de recuperação.

## Comportamento

Sem novo código de produção. A suite de testes verificará:

1. Um banco de dados novo com WAL limpo abre com sucesso.
2. Um banco de dados cujo arquivo WAL foi truncado no meio de um registro abre
   com sucesso (último registro parcial é ignorado, não fatal).
3. Após abrir com WAL rasgado, operações anteriores à entrada rasgada estão
   intactas no estado recuperado.
4. Um WAL com um registro JSON inválido (corrompido) no final abre sem panic
   e ignora a entrada ruim.

## Arquivos

- `crates/hippocore/tests/wal_recovery.rs` — novo arquivo de testes dedicado.
- `docs/en/WAL_RECOVERY.md` e `docs/pt-br/WAL_RECOVERY.md`.
- `docs/en/STATUS.md` e `docs/pt-br/STATUS.md`.

## Critérios de aceite

- Mínimo de 4 testes de integração determinísticos usando `TempDir` e
  manipulação direta do arquivo WAL (truncar / corromper via `std::fs`).
- Quality gate:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Fora de escopo

- Mudanças na compactação do WAL.
- Modo server.
- Mudanças no código de produção.
