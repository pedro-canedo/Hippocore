# Próxima Feature

## Nome

**Admin CLI v0.1**

## Por que importa

O Control Plane e a API REST agora cobrem os fluxos principais de ingestao de
dados. O proximo slice de maior valor e adicionar sub-comandos CLI que espelhem
esses fluxos para que operadores possam automatizar sem precisar rodar um
servidor HTTP. O binario `hippocore-cli` ja tem `stats` e `bench`; faltam
comandos de plano de dados.

## Comportamento

- `hippocore tenants list` — listar todos os tenants no diretorio de dados.
- `hippocore tenants create <id> [--name <n>]` — criar tenant.
- `hippocore collections list --tenant <tid>` — listar collections do tenant.
- `hippocore collections create --tenant <tid> <name>` — criar collection.
- `hippocore memories add --tenant <tid> --collection <col> <text>` — salvar
  memoria.
- `hippocore records put --tenant <tid> --collection <col> --table <tbl>
  [--payload <json>|--file <path>]` — salvar record JSON-first.
- `hippocore recall --tenant <tid> --collection <col> <query>` — recall hibrido.
- Todos os comandos leem do diretorio de dados (flag `--db`, padrao
  `./hippocore-data`).
- Saida e texto simples ou JSON (flag `--json`).

## Arquivos provaveis

- `crates/hippocore-cli/src/main.rs`
- `crates/hippocore/src/cli.rs`
- `crates/hippocore/tests/record_crud.rs`
- `docs/en/STATUS.md`
- `docs/pt-br/STATUS.md`
- `CHANGELOG.md`

## Criterios de aceite

- Todos os sub-comandos listados implementados e respondem a `--help`.
- `hippocore tenants list` lista tenants apos `tenants create`.
- `hippocore records put` cria record legivel via `hippocore sql`.
- `hippocore recall` retorna resultados em texto.
- Isolamento de tenant: comandos falham com erro claro se o tenant nao existe.
- `cargo fmt --all --check`, `cargo test --workspace` e
  `cargo clippy --workspace --all-targets -- -D warnings` passam.

## Fora do escopo

- TUI interativa.
- Completacao de shell.
- Saida em streaming.
- Import de CSV.
- CLI de travessia de grafo.
- Sub-comandos de servidor HTTP (esses ficam em `hippocore serve`).
