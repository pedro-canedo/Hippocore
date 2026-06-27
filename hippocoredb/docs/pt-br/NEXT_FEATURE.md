# Proxima Feature

## Nome da feature

**Control Plane TS — Ingestion v0.5**

## Por que importa

O console TypeScript já navega e consulta dados, mas não consegue criar nenhum.
O próximo passo de maior valor é a ingestão: criar Memories, Records e Documents
no tenant e na collection ativos, para o `/console` se tornar autossuficiente no
loop central de escrita sem recorrer ao `/admin`.

## Comportamento

- Adicionar uma página **Ingestion** à sidebar, condicionada a um tenant ativo e
  a um seletor de collection (reutilizando a memória de collection por tenant).
- Três formulários tipados:
  - **Memory**: texto, tipo de memória, confiança opcional → `POST
    /admin/tenants/{tid}/memories`.
  - **Record**: nome da tabela + payload JSON (validado no cliente) → `POST
    /admin/tenants/{tid}/records`.
  - **Document**: texto (e título/metadata opcionais) → `POST
    /admin/tenants/{tid}/documents`.
- Cada criação bem-sucedida limpa o formulário, mostra um toast e atualiza as
  contagens do Dashboard; o `ApiError` tipado renderiza inline.
- JSON inválido no payload de Record é detectado antes do envio.

## Arquivos provaveis

- `crates/hippocore-server/admin-ui/src/api.ts` (adicionar calls de ingestão)
- `crates/hippocore-server/admin-ui/src/views/IngestionView.tsx` (novo)
- `crates/hippocore-server/admin-ui/src/App.tsx`, `views.ts`, `styles.css`
- Assets rebuildados em `crates/hippocore-server/src/console/`
- `crates/hippocore-server/tests/server_integration.rs`
- Status/CHANGELOG bilíngues e `CONTROL_PLANE_TS.md`

## Criterios de aceitacao

- Criar Memory, Record e Document persiste no tenant e collection ativos e
  aparece no Data Explorer.
- JSON inválido de Record é rejeitado no cliente com mensagem clara.
- Contagens do Dashboard atualizam após cada criação.
- `pnpm type-check` e `pnpm build` passam; o portão Rust fica verde.
- Texto novo voltado ao usuário existe em inglês e português.

## Testes

- Assets de `/console` continuam públicos (teste de integração existente).
- Testes dos endpoints de ingestão existentes continuam verdes.
- `tsc --noEmit` strict passa; build atualiza `src/console/`.

## Fora de escopo

- Upload de arquivo / drag-and-drop (incremento futuro) e Recall.
- Editar ou excluir entidades existentes.
- Portar Integrations, Prompts ou Observability.
