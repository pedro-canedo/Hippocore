# Hippocore Control Plane (TypeScript)

React + Vite + TypeScript admin console for Hippocore DB. This is the new
Control Plane being migrated from the legacy zero-build vanilla-JS app
(`../src/admin/`). It is served at **`/console`** while the classic console
stays at **`/admin`** during the migration.

## Why the build output lives in `../src/console`

The Rust server embeds the console into the binary with `include_str!`, so the
build is a single, dependency-free artifact and `cargo build` never needs Node.
`vite build` therefore emits flat, non-hashed files
(`index.html`, `index.js`, `index.css`) directly into
`crates/hippocore-server/src/console/`, which **is committed to git**. The
`dist/` name is avoided because `.dockerignore` excludes `**/dist`.

## Develop

```bash
pnpm install
pnpm dev          # Vite dev server (proxy /admin and /health to a running hippocore serve)
```

## Build (regenerates the embedded assets)

```bash
pnpm install
pnpm type-check   # tsc --noEmit
pnpm build        # writes ../src/console/{index.html,index.js,index.css}
```

After building, rebuild the server so the new assets are embedded:

```bash
cargo build -p hippocore-server
```

## Quality gate

Frontend changes must pass `pnpm type-check` and `pnpm build` in addition to the
Rust gate (`cargo fmt --all --check`, `cargo test --workspace`,
`cargo clippy --workspace --all-targets -- -D warnings`).
