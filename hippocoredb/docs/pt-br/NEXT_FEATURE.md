# Próxima Feature

## Nome

**Record CRUD Tests v0.1** — testes determinísticos para o ciclo de vida da
entidade Record.

## Por que importa

`Record` é uma entidade de primeira classe ao lado de `Memory` e `Document`,
mas não tem arquivo de testes dedicado. Records suportam versionamento
(incremento de versão na atualização), deleção e filtro por metadados. Esses
caminhos são exercidos incidentalmente em outros testes mas não estão bloqueados
como regressões explícitas.

## Comportamento

Sem novo código de produção. A suite de testes cobrirá:

1. Armazenar um record e recuperar por id.
2. Recall retorna o record armazenado.
3. Atualizar (re-armazenar mesmo id) incrementa o campo version.
4. Deletar um record; recuperação subsequente retorna `None`.
5. Deletar um record; recall subsequente não o retorna.
6. Records de um tenant não são visíveis para outro tenant.

## Arquivos

- `crates/hippocore/tests/record_crud.rs` — novo arquivo de testes dedicado.
- `docs/en/RECORD_CRUD.md` e `docs/pt-br/RECORD_CRUD.md`.
- `docs/en/STATUS.md` e `docs/pt-br/STATUS.md`.

## Critérios de aceite

- Mínimo de 6 testes de integração determinísticos usando `TempDir`.
- Quality gate:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Fora de escopo

- Operações em lote para records.
- Modo server.
- Mudanças no código de produção.
