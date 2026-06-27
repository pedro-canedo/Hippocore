# Proxima Feature

## Nome da feature

**Control Plane TS — Prompts + Chat (SSE) v0.9**

## Por que importa

Com o cérebro LLM configurável no `/console`, a recompensa é o Chat RAG → LLM. O
console clássico tem CRUD de System Prompts e um endpoint de Chat com streaming.
Portá-los realiza a visão "LLM as brain" na nova UI tipada e é a última grande
superfície de produto antes de o `/admin` poder ser aposentado.

## Comportamento

- Adicionar uma página **Prompts**: listar/criar/atualizar/excluir templates de
  system prompt via `GET/POST/PUT/DELETE /admin/prompts` (id, name, description,
  content com placeholders `{{context}}`/`{{query}}`, escopo opcional de tenant).
- Adicionar um painel **Chat** (condicionado a um tenant ativo): query +
  collection opcional + prompt id opcional, com streaming da resposta token a
  token de `POST /admin/tenants/{tid}/chat` usando `fetch` +
  `response.body.getReader()`.
- Renderizar tokens transmitidos ao vivo numa área de saída; mostrar a lista
  final de sources e tratar `event: error` do stream SSE sem quebrar.

## Arquivos provaveis

- `crates/hippocore-server/admin-ui/src/api.ts` (CRUD de prompts + chat stream)
- `crates/hippocore-server/admin-ui/src/views/PromptsView.tsx` (novo)
- `crates/hippocore-server/admin-ui/src/App.tsx`, `views.ts`, `styles.css`
- Assets rebuildados em `crates/hippocore-server/src/console/`
- `crates/hippocore-server/tests/server_integration.rs`
- Status/CHANGELOG bilíngues e `CONTROL_PLANE_TS.md`

## Criterios de aceitacao

- CRUD de prompts funciona ponta a ponta; um prompt pode ser selecionado no chat.
- O chat transmite tokens ao vivo e mostra sources; erros SSE renderizam inline.
- `pnpm type-check` e `pnpm build` passam; o portão Rust fica verde.
- Texto novo voltado ao usuário existe em inglês e português.

## Testes

- Assets de `/console` continuam públicos (teste de integração existente).
- Testes dos endpoints de prompts/chat existentes continuam verdes.
- `tsc --noEmit` strict passa; build atualiza `src/console/`.

## Fora de escopo

- Portar Observability, Graph, API Reference, Settings, Documentation.
- Geradores de config Traefik/Docker.
- Aposentar o `/admin` (somente após paridade nas páginas restantes).
