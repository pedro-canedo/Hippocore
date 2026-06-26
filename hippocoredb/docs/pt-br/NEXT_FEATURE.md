# Próxima Feature

## Nome

**Admin Data Actions v0.1**

## Por que importa

O Control Plane agora separa os dominios do produto e mostra os fluxos certos,
mas algumas acoes importantes da UI continuam marcadas como Planned porque os
endpoints admin ainda nao existem. O proximo slice de maior valor e tornar essas
acoes reais sem expandir para um motor SQL completo nem para um framework
frontend grande.

## Comportamento

- Adicionar endpoints admin/server para insert de record estruturado e import de
  arquivo.
- Reusar APIs core existentes: `put_record` e `import_file`.
- Manter isolamento por tenant explicito em toda rota.
- Atualizar Ingestion & Recall no Control Plane para Memory, Document, Record e
  File serem acionaveis quando suportados pelo backend.
- Manter fluxos futuros marcados como Planned em vez de simular dados.

## Arquivos provaveis

- `crates/hippocore-server/src/handlers/records.rs`
- `crates/hippocore-server/src/handlers/files.rs`
- `crates/hippocore-server/src/lib.rs`
- `crates/hippocore-server/src/admin/app.js`
- `crates/hippocore-server/tests/server_integration.rs`
- `docs/en/ADMIN_INTERFACE.md`
- `docs/pt-br/ADMIN_INTERFACE.md`
- `docs/en/SERVER.md`
- `docs/pt-br/SERVER.md`

## Criterios de aceite

- `POST /admin/tenants/:tid/records` cria um record JSON-first.
- `POST /admin/tenants/:tid/files/import` importa um caminho local text-like.
- Login/auth admin existentes continuam funcionando.
- Control Plane consegue criar Memory, Document e Record em Ingestion & Recall.
- Import de arquivo so e implementado se puder ser feito com seguranca a partir
  de caminho local; caso contrario, permanece Planned com docs claros.
- Testes cobrem sucesso, erros de validacao e isolamento por tenant.
- `cargo fmt --all --check`, `cargo test --workspace` e
  `cargo clippy --workspace --all-targets -- -D warnings` passam.

## Fora do escopo

- Upload multipart no browser.
- CSV como rows de records.
- Upload PDF.
- SQL `INSERT` completo.
- Catalogo de table/schema.
- Migracao para framework frontend.
