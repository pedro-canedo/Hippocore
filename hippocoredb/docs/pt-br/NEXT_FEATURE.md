# Proxima Feature

## Nome da feature

**Control Plane TS — Upload de Arquivos (drag-and-drop) v0.7**

## Por que importa

A ingestão no `/console` cobre Memory, Record e Document, mas não arquivos. O
console clássico tem upload drag-and-drop (PDF/CSV/TXT/MD/JSON) via
`POST /admin/tenants/{tid}/files`. Portá-lo completa a paridade de ingestão e é
o último caminho de escrita comum faltando no console TypeScript.

## Comportamento

- Adicionar uma drop zone à página Ingestion (ou uma aba Files dedicada)
  aceitando `.pdf`, `.txt`, `.md`, `.csv`, `.json`, condicionada a tenant +
  collection.
- Upload via `XMLHttpRequest` para expor progresso real (barra de progresso),
  postando multipart `file` + `collection` em `POST /admin/tenants/{tid}/files`.
- Mostrar um banner de sucesso com o resultado de dispatch do servidor
  (`{ file_id, kind, count, name }`) e atualizar as contagens do Dashboard;
  clicar na zona também abre um seletor de arquivos.
- Tipos não suportados e erros do servidor renderizam inline.

## Arquivos provaveis

- `crates/hippocore-server/admin-ui/src/api.ts` (helper de upload XHR + progresso)
- `crates/hippocore-server/admin-ui/src/views/IngestionView.tsx` (drop zone) ou
  um novo `views/FilesView.tsx`
- `crates/hippocore-server/admin-ui/src/components/DropZone.tsx` (novo)
- `crates/hippocore-server/admin-ui/src/App.tsx`, `views.ts`, `styles.css`
- Assets rebuildados em `crates/hippocore-server/src/console/`
- `crates/hippocore-server/tests/server_integration.rs`
- Status/CHANGELOG bilíngues e `CONTROL_PLANE_TS.md`

## Criterios de aceitacao

- Arrastar ou selecionar um arquivo suportado faz upload com barra de progresso
  visível e mostra o resultado de dispatch; os dados aparecem no Data Explorer.
- Extensões não suportadas e erros do servidor renderizam inline sem quebrar.
- `pnpm type-check` e `pnpm build` passam; o portão Rust fica verde.
- Texto novo voltado ao usuário existe em inglês e português.

## Testes

- Assets de `/console` continuam públicos (teste de integração existente).
- Testes do endpoint de upload existente continuam verdes.
- `tsc --noEmit` strict passa; build atualiza `src/console/`.

## Fora de escopo

- Integrations (LLM), Prompts, Context builder/Chat, Observability, Graph.
- Filas multi-arquivo além de uploads sequenciais.
- Aposentar o `/admin`.
