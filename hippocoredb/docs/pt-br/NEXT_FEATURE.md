# Próxima Feature

## Nome

**Admin CLI v0.1** — facilitar inspeção, listagem e exportação do banco sem
introduzir server mode ou UI ainda.

## Por que importa

Hippocore DB agora armazena documentos, memórias, records estruturados e
arquivos importados. Antes de construir um Studio local, usuários precisam de
uma superfície prática de administração, no espírito dos fluxos básicos de
pgAdmin/DBeaver: ver o que existe, inspecionar objetos, exportar dados e
depurar contexto.

## Comportamento esperado

- Adicionar comandos de listagem para tipos armazenados:
  - `list-tenants`;
  - `list-collections`;
  - `list-documents`;
  - `list-memories`;
  - `list-records`;
  - `list-files`.
- Adicionar `inspect --json` ou saída equivalente legível por máquina.
- Adicionar comandos de detalhe quando fizer sentido, como `show-record`,
  `show-document` e `show-file`.
- Adicionar exportação JSON de objetos ou collections selecionadas.
- Manter tudo local-first e embedded; sem server mode.
- Preservar tenant isolation e visibilidade de metadata.

## Arquivos afetados

- `crates/hippocore/src/lib.rs` — APIs públicas de leitura/listagem quando
  faltarem.
- `crates/hippocore/src/cli.rs` — comandos administrativos e saída JSON.
- `crates/hippocore-cli/tests/cli.rs` — cobertura por subprocesso.
- `docs/en/ADMIN_INTERFACE.md` e `docs/pt-br/ADMIN_INTERFACE.md`.
- `README.md`, `STATUS.md`, `CHANGELOG.md`.

## Critérios de aceite

- Usuário consegue listar tenants, collections e cada tipo de objeto armazenado.
- Usuário consegue inspecionar record/file/document específico o suficiente para
  entender o que foi armazenado e qual projeção de contexto existe.
- Saída JSON é estável o bastante para scripts e futura integração com Studio.
- Nenhum processo servidor é necessário.
- `cargo fmt --all --check`, `cargo test --workspace` e clippy passam.

## Fora de escopo

- Web UI.
- Protocolo PostgreSQL.
- Linguagem SQL.
- Auth/autorização.
- Server mode remoto.

## Follow-up

Depois do Admin CLI v0.1, revisitar um protótipo local do Hippocore Studio e
adicionar um comando pequeno de avaliação de qualidade usando o formato de
fixture existente.
