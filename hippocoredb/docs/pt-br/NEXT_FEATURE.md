# Proxima Feature

## Nome da feature

**Control Plane TS — Recall v0.6**

## Por que importa

O console já cria, navega e consulta dados. A capacidade assinatura — recall RAG
— ainda está apenas no `/admin`. Portar o Recall para o `/console` permite aos
operadores testar a recuperação (vector/text/hybrid) com scores e termos
encontrados no mesmo lugar onde ingeriram os dados, completando o loop central
de demonstração do produto na nova UI tipada.

## Comportamento

- Adicionar uma página **Recall** à sidebar, condicionada a um tenant ativo.
- Uma caixa de query com escopo opcional de collection, seletor de modo
  (vector/text/hybrid) e `top_k`, chamando `POST /admin/tenants/{tid}/recall`.
- Renderizar resultados como cards mostrando tipo, score, termos encontrados e
  um trecho de texto, mais um toggle Raw JSON para a resposta completa.
- Expor o `ApiError` tipado inline; resultados vazios mostram um empty state
  claro.

## Arquivos provaveis

- `crates/hippocore-server/admin-ui/src/api.ts` (adicionar `recall` + tipos)
- `crates/hippocore-server/admin-ui/src/views/RecallView.tsx` (novo)
- `crates/hippocore-server/admin-ui/src/App.tsx`, `views.ts`, `styles.css`
- Assets rebuildados em `crates/hippocore-server/src/console/`
- `crates/hippocore-server/tests/server_integration.rs`
- Status/CHANGELOG bilíngues e `CONTROL_PLANE_TS.md`

## Criterios de aceitacao

- Rodar uma query retorna resultados ranqueados com scores e termos encontrados
  para o tenant ativo; modo e top_k são respeitados; isolamento de tenant vale.
- Resultados vazios e erros renderizam sem quebrar a view.
- `pnpm type-check` e `pnpm build` passam; o portão Rust fica verde.
- Texto novo voltado ao usuário existe em inglês e português.

## Testes

- Assets de `/console` continuam públicos (teste de integração existente).
- Testes do endpoint de recall existente continuam verdes.
- `tsc --noEmit` strict passa; build atualiza `src/console/`.

## Fora de escopo

- Context builder / chat (incrementos futuros).
- Upload de arquivo / drag-and-drop, Integrations, Prompts, Observability.
- Ajustar knobs de MMR/dedup/min-score além de modo e top_k.
