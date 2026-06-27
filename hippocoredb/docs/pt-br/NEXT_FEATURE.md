# Proxima Feature

## Nome da feature

**Control Plane TS — Shell de Navegação + Tenants & Collections v0.2**

## Por que importa

O console TypeScript (`/console`) hoje tem apenas a fatia Login → Dashboard. O
próximo incremento de maior valor é o shell de navegação mais as primeiras
páginas de mutação de dados — Tenants e Collections — que estabelecem a sidebar,
o roteamento, o seletor global de tenant e o padrão de formulário create/list
reutilizado por todas as páginas seguintes.

## Comportamento

- Adicionar um shell de sidebar persistente (marca, navegação agrupada, logout)
  e troca de view client-side dirigida por estado de componente (ainda sem
  biblioteca de router).
- Um seletor global de tenant na topbar persiste o tenant ativo no local storage
  e o compartilha com o resto do console.
- **Página Tenants**: listar tenants de `GET /admin/tenants`; criar tenant via
  `POST /admin/tenants`; mostrar empty state e erros de validação inline.
- **Página Collections**: listar collections do tenant ativo de
  `GET /admin/collections?tenant_id=`; criar via
  `POST /admin/tenants/{tid}/collections`; exigir um tenant ativo primeiro.
- Todas as requisições passam pelo cliente de API tipado; sucesso atualiza a
  lista e mostra um toast; o `ApiError` tipado expõe mensagens do servidor.
- O Dashboard ganha links para as novas páginas.

## Arquivos provaveis

- `crates/hippocore-server/admin-ui/src/api.ts` (calls de tenants/collections)
- `crates/hippocore-server/admin-ui/src/App.tsx` (estado de view + shell)
- `crates/hippocore-server/admin-ui/src/components/Shell.tsx` (novo)
- `crates/hippocore-server/admin-ui/src/views/TenantsView.tsx` (novo)
- `crates/hippocore-server/admin-ui/src/views/CollectionsView.tsx` (novo)
- `crates/hippocore-server/admin-ui/src/styles.css`
- Assets rebuildados em `crates/hippocore-server/src/console/`
- `crates/hippocore-server/tests/server_integration.rs`
- Status/CHANGELOG bilíngues e `CONTROL_PLANE_TS.md`

## Criterios de aceitacao

- A navegação da sidebar troca entre Dashboard, Tenants e Collections sem reload
  completo.
- Criar tenant e collection persiste e aparece imediatamente nas listas;
  isolamento de tenant é respeitado para collections.
- O tenant ativo é lembrado entre reloads e compartilhado com `/admin`.
- `pnpm type-check` e `pnpm build` passam; o portão Rust fica verde.
- Texto novo existe em inglês e português onde voltado ao usuário.

## Testes

- Assertions de integração de que os assets de `/console` continuam servindo e
  públicos.
- Testes admin/server existentes continuam verdes (endpoints de
  tenants/collections já cobertos no servidor).
- `tsc --noEmit` strict passa; build conclui e atualiza `src/console/`.

## Fora de escopo

- Portar Records, Memories, Documents, Files, SQL Editor, Recall, Integrations,
  Prompts ou Observability (incrementos futuros).
- Adicionar biblioteca de router, de state-management ou kit de componentes.
- Aposentar `/admin` ou redirecionar `/` (só após paridade completa).
