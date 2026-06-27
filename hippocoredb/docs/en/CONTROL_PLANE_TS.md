# Control Plane — TypeScript Rebuild

The admin Control Plane is being rebuilt as a **React + Vite + TypeScript**
application. It is served at **`/console`**. The original zero-build vanilla-JS
console remains at **`/admin`** until every page is ported.

See ADR-016 for the rationale and trade-offs.

## Layout

```
crates/hippocore-server/
├── admin-ui/                 # React + Vite + TypeScript source (Node-only)
│   ├── src/
│   │   ├── api.ts            # typed admin API client
│   │   ├── session.ts        # session token persistence
│   │   ├── main.tsx          # React entry point
│   │   ├── App.tsx           # top-level state + routing between views
│   │   ├── views/
│   │   │   ├── LoginView.tsx
│   │   │   └── DashboardView.tsx
│   │   └── styles.css
│   ├── index.html            # Vite dev entry
│   ├── vite.config.ts        # base "/console/", outDir ../src/console
│   ├── tsconfig.json
│   └── package.json
└── src/
    └── console/              # BUILT, COMMITTED, embedded with include_str!
        ├── index.html
        ├── index.js
        └── index.css
```

## Why the build output is committed

The Rust server embeds the console with `include_str!`, so the binary stays
self-contained and `cargo build` (and the Docker image) never needs Node. The
build therefore writes flat, non-hashed files into
`crates/hippocore-server/src/console/`, which is tracked in git. The `dist/`
name is avoided because `.dockerignore` excludes `**/dist`.

When the frontend changes, rebuild and re-commit the generated assets.

## Develop

```bash
cd crates/hippocore-server/admin-ui
pnpm install
pnpm dev          # Vite dev server; run `hippocore serve` for the API
```

## Build (regenerate embedded assets)

```bash
cd crates/hippocore-server/admin-ui
pnpm install
pnpm type-check   # tsc --noEmit
pnpm build        # writes ../src/console/{index.html,index.js,index.css}
cargo build -p hippocore-server   # re-embed the new assets
```

## Quality gate

Frontend changes must pass, in addition to the Rust gate:

```bash
pnpm type-check   # tsc --noEmit, strict
pnpm build        # must succeed and update src/console/
```

Rust gate (always):

```bash
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Current scope

- Login → Dashboard with server status and database counts.
- Navigation shell (sidebar + topbar) with client-side view switching.
- Topbar tenant selector; active tenant persisted and shared with `/admin`
  (`hippocore.tenant`).
- Tenants page: list and create.
- Collections page: list and create for the active tenant.
- Data Explorer: read-only browse of Records/Memories/Documents/Files for the
  active tenant and a per-tenant-remembered collection, with a JSON drawer.
- Shared session key (`hippocore.adminSession`) with the `/admin` console.

## Migration plan

Pages are ported one at a time (Tenants, Collections, Data Explorer, SQL Editor,
Ingestion & Recall, Integrations, Prompts, Observability). When the new console
reaches parity, `/admin` is retired and `/` redirects to `/console`.
