# Proxima Feature

## Nome da feature

**Control Plane TS — Integrations (LLM) v0.8**

## Por que importa

O console cobre todo o loop de dados, mas não a configuração do "cérebro" LLM. O
console clássico tem uma página Integrations (providers + validate + ping ao vivo
+ exposição da API). Portar a configuração de provider para o `/console`
desbloqueia a feature de Chat (RAG → LLM) futura e permite conectar o cérebro na
nova UI tipada.

## Comportamento

- Adicionar uma página **Integrations** à sidebar.
- Listar providers configurados de `GET /admin/llm-providers`; adicionar/atualizar
  um via `POST /admin/llm-providers` (id, kind, base_url, model, api_key opcional).
- **Validate** a config de um provider localmente
  (`POST /admin/llm-providers/validate`) e **Ping** ao vivo
  (`POST /admin/llm-providers/ping`), mostrando ok/mensagem e latência sem
  quebrar em falha de rede.
- Mostrar a exposição da API de `GET /admin/api-info` (base URL, hint da key
  mascarada) com um exemplo curl de recall.

## Arquivos provaveis

- `crates/hippocore-server/admin-ui/src/api.ts` (calls de provider + api-info)
- `crates/hippocore-server/admin-ui/src/views/IntegrationsView.tsx` (novo)
- `crates/hippocore-server/admin-ui/src/App.tsx`, `views.ts`, `styles.css`
- Assets rebuildados em `crates/hippocore-server/src/console/`
- `crates/hippocore-server/tests/server_integration.rs`
- Status/CHANGELOG bilíngues e `CONTROL_PLANE_TS.md`

## Criterios de aceitacao

- Listar, adicionar/atualizar, validar e pingar providers funcionam; a API key
  armazenada nunca é mostrada (apenas indicador de set/length e hint mascarado).
- Uma URL de ping ruim retorna ok:false inline (sem crash, sem 500 fatal).
- `pnpm type-check` e `pnpm build` passam; o portão Rust fica verde.
- Texto novo voltado ao usuário existe em inglês e português.

## Testes

- Assets de `/console` continuam públicos (teste de integração existente).
- Testes existentes do registro de providers (mascaramento de segredo) continuam
  verdes.
- `tsc --noEmit` strict passa; build atualiza `src/console/`.

## Fora de escopo

- Prompts e Chat (o próximo incremento depois deste).
- Geradores de config Traefik/Docker (podem vir depois).
- Observability, Graph, Settings.
