# Control Plane — Reconstrução TypeScript

O Control Plane admin está sendo reconstruído como uma aplicação
**React + Vite + TypeScript**, servida em **`/console`**. O console original
vanilla-JS zero-build permanece em **`/admin`** até todas as páginas serem
portadas.

Veja o ADR-016 para a motivação e os trade-offs.

## Estrutura

```
crates/hippocore-server/
├── admin-ui/                 # Código React + Vite + TypeScript (só Node)
│   ├── src/
│   │   ├── api.ts            # cliente tipado da API admin
│   │   ├── session.ts        # persistência do token de sessão
│   │   ├── main.tsx          # entry point React
│   │   ├── App.tsx           # estado de topo + roteamento entre views
│   │   ├── views/
│   │   │   ├── LoginView.tsx
│   │   │   └── DashboardView.tsx
│   │   └── styles.css
│   ├── index.html            # entry de dev do Vite
│   ├── vite.config.ts        # base "/console/", outDir ../src/console
│   ├── tsconfig.json
│   └── package.json
└── src/
    └── console/              # BUILDADO, COMMITADO, embarcado via include_str!
        ├── index.html
        ├── index.js
        └── index.css
```

## Por que o build é commitado

O servidor Rust embarca o console via `include_str!`, então o binário continua
auto-contido e `cargo build` (e a imagem Docker) nunca precisa de Node. O build
então escreve arquivos planos sem hash em
`crates/hippocore-server/src/console/`, que é versionado no git. O nome `dist/`
é evitado porque o `.dockerignore` exclui `**/dist`.

Quando o front-end muda, rebuilde e re-commite os assets gerados.

## Desenvolver

```bash
cd crates/hippocore-server/admin-ui
pnpm install
pnpm dev          # servidor de dev do Vite; rode `hippocore serve` para a API
```

## Build (regenerar assets embarcados)

```bash
cd crates/hippocore-server/admin-ui
pnpm install
pnpm type-check   # tsc --noEmit
pnpm build        # escreve ../src/console/{index.html,index.js,index.css}
cargo build -p hippocore-server   # re-embarca os novos assets
```

## Portão de qualidade

Mudanças de front-end devem passar, além do portão Rust:

```bash
pnpm type-check   # tsc --noEmit, strict
pnpm build        # deve concluir e atualizar src/console/
```

Portão Rust (sempre):

```bash
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Escopo atual

- Login → Dashboard com status do servidor e contagens do banco.
- Shell de navegação (sidebar + topbar) com troca de view client-side.
- Seletor de tenant na topbar; tenant ativo persistido e compartilhado com
  `/admin` (`hippocore.tenant`).
- Página Tenants: listar e criar.
- Página Collections: listar e criar para o tenant ativo.
- Data Explorer: navegação read-only de Records/Memories/Documents/Files do
  tenant ativo e de uma collection lembrada por tenant, com drawer JSON.
- SQL Editor: `SELECT` read-only sobre payloads de records via `POST /admin/sql`,
  com Run/`Ctrl+Enter`, colunas deterministas e toggle Raw JSON.
- Ingestion: criar Memory, Record (payload JSON) e Document no tenant/collection
  ativos, com validação de JSON no cliente.
- Recall: testar recuperação via `POST /admin/tenants/{tid}/recall` com escopo
  opcional de collection, `top_k`, toggles dedup/MMR e cards de resultado.
- Chave de sessão compartilhada (`hippocore.adminSession`) com o console `/admin`.

## Plano de migração

As páginas são portadas uma a uma (Tenants, Collections, Data Explorer, SQL
Editor, Ingestion & Recall, Integrations, Prompts, Observability). Quando o novo
console atingir paridade, `/admin` é aposentado e `/` redireciona para
`/console`.
