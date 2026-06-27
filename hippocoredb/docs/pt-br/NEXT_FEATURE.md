# Proxima Feature

## Nome da feature

**Control Plane TS — Data Explorer (leitura) v0.3**

## Por que importa

Com tenants e collections gerenciáveis no `/console`, o próximo passo de maior
valor é ver os dados dentro deles. Um Data Explorer read-only permite navegar
Records, Memories, Documents e Files do tenant ativo e de uma collection
escolhida, estabelecendo o padrão de listagem de entidades reutilizado por todas
as páginas de gestão seguintes.

## Comportamento

- Adicionar uma página **Data Explorer** à sidebar, condicionada a um tenant
  ativo.
- Um seletor de collection (populado por `GET /admin/collections?tenant_id=`)
  escolhe o escopo; lembrar a última collection por tenant localmente.
- Abas por tipo — Records, Memories, Documents, Files — carregam em paralelo para
  a collection ativa e mostram contagens por tipo nas abas.
- Records renderizam as chaves do payload como colunas deterministas (id e table
  ficam como colunas de sistema estáveis); outros tipos mostram seus campos
  principais. Um drawer de detalhe mostra o JSON completo da linha selecionada.
- Empty states orientam a criar uma collection ou ingerir o primeiro dado pelo
  console clássico enquanto a ingestão não foi portada.

## Arquivos provaveis

- `crates/hippocore-server/admin-ui/src/api.ts` (listar records/memories/
  documents/files; reutilizar `/admin/{records,memories,documents,files}`)
- `crates/hippocore-server/admin-ui/src/views/DataExplorerView.tsx` (novo)
- `crates/hippocore-server/admin-ui/src/components/Drawer.tsx` (novo)
- `crates/hippocore-server/admin-ui/src/App.tsx`, `views.ts`, `styles.css`
- Assets rebuildados em `crates/hippocore-server/src/console/`
- `crates/hippocore-server/tests/server_integration.rs`
- Status/CHANGELOG bilíngues e `CONTROL_PLANE_TS.md`

## Criterios de aceitacao

- Selecionar tenant e collection lista cada tipo com contagens corretas.
- Chaves do payload de records aparecem como colunas deterministas; o drawer
  mostra JSON completo; isolamento de tenant é respeitado (só dados do tenant
  ativo aparecem).
- A collection escolhida é lembrada por tenant entre reloads.
- `pnpm type-check` e `pnpm build` passam; o portão Rust fica verde.
- Texto novo voltado ao usuário existe em inglês e português.

## Testes

- Assets de `/console` continuam públicos (teste de integração existente).
- Endpoints de listagem do servidor continuam verdes.
- `tsc --noEmit` strict passa; build atualiza `src/console/`.

## Fora de escopo

- Criar/editar/excluir entidades pelo Explorer (read-only por enquanto).
- Portar SQL Editor, Ingestion & Recall, Integrations, Prompts, Observability.
- Paginação, busca server-side ou export CSV.
