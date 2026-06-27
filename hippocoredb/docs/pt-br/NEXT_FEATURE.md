# Proxima Feature

## Nome da feature

**Control Plane TS — SQL Editor v0.4**

## Por que importa

O Data Explorer navega records por collection; o próximo passo é consultá-los.
Portar o SQL Editor read-only para o `/console` dá aos operadores o poder do
endpoint existente `POST /admin/sql` (`SELECT` escopado por tenant sobre payloads
de records) dentro do novo console tipado, completando o loop central de "ver e
consultar seus dados".

## Comportamento

- Adicionar uma página **SQL Editor** à sidebar, condicionada a um tenant ativo.
- Um textarea de query com botão Run e `Ctrl/Cmd + Enter` para executar contra
  `POST /admin/sql` com `{ tenant_id, sql }`.
- Renderizar resultados de sucesso como tabela com colunas deterministas de
  payload e um toggle Raw JSON; mostrar a duração observada no cliente e a
  contagem de linhas.
- Expor erros estruturados de validação/parse via o `ApiError` tipado.
- Manter a query atual no estado da view; snippets de exemplo preenchem o editor
  sem executar automaticamente. Explicar brevemente que campos sem prefixo
  mapeiam para `payload.<field>`.

## Arquivos provaveis

- `crates/hippocore-server/admin-ui/src/api.ts` (adicionar `runSql`)
- `crates/hippocore-server/admin-ui/src/views/SqlEditorView.tsx` (novo)
- `crates/hippocore-server/admin-ui/src/App.tsx`, `views.ts`, `styles.css`
- Assets rebuildados em `crates/hippocore-server/src/console/`
- `crates/hippocore-server/tests/server_integration.rs`
- Status/CHANGELOG bilíngues e `CONTROL_PLANE_TS.md`

## Criterios de aceitacao

- Botão Run e `Ctrl/Cmd + Enter` executam o mesmo request escopado por tenant.
- Resultados mostram colunas deterministas de payload e uma visão JSON completa;
  duração e contagem de linhas aparecem.
- Erros de parse/validação renderizam inline sem quebrar a view.
- `pnpm type-check` e `pnpm build` passam; o portão Rust fica verde.
- Texto novo voltado ao usuário existe em inglês e português.

## Testes

- Assets de `/console` continuam públicos (teste de integração existente).
- Testes do endpoint `/admin/sql` existente continuam verdes.
- `tsc --noEmit` strict passa; build atualiza `src/console/`.

## Fora de escopo

- Expandir a gramática SQL ou adicionar escrita.
- Histórico de queries, queries salvas, paginação ou export CSV.
- Portar Ingestion & Recall, Integrations, Prompts, Observability.
