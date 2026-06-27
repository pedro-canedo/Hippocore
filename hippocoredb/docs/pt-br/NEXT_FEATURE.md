# Proxima Feature

## Nome da feature

**Control Plane i18n Completeness v0.1**

## Por que importa

O seletor de idioma traduz apenas parte da interface. Ainda existem textos em
ingles na navegacao, onboarding, headers, empty states, formularios, toasts e
erros, criando um produto com idioma misturado. Completar a fronteira PT/EN
existente e requisito P0 antes de ampliar a UI.

## Comportamento

- Mover todos os textos visiveis do Control Plane para o catalogo `I18N` existente.
- Traduzir navegacao, onboarding, paginas de dados, SQL, arquivos, integracoes,
  prompts, observabilidade, settings, acoes, erros e toasts.
- Manter identificadores tecnicos como `tenant`, `collection`, `record`, SQL,
  paths de endpoint, nomes de campos e providers quando traduzir reduzir precisao.
- Alternar PT/EN renderiza a pagina ativa sem perder tenant, collection,
  resultados, drafts ou sessao.

## Arquivos provaveis

- `crates/hippocore-server/src/admin/app.js`
- `crates/hippocore-server/tests/server_integration.rs`
- docs bilingues de status/interface e `CHANGELOG.md`

## Criterios de aceitacao

- Toda frase e comando visivel segue o idioma selecionado.
- Nao resta mistura PT/EN, exceto vocabulario tecnico documentado.
- Troca de idioma preserva o estado do fluxo ativo.
- Endpoints e autenticacao permanecem inalterados.
- Check de sintaxe JavaScript e portao Rust completo ficam verdes.

## Testes

- Assertions dos assets cobrem marcadores dos catalogos ingles e portugues.
- Auditoria estatica confirma que caminhos visiveis usam chaves de traducao.
- Testes de integracao existentes do servidor continuam verdes.

## Fora de escopo

- Terceiro idioma ou framework externo de localizacao.
- Traduzir campos da API, sintaxe SQL, ids ou dados armazenados pelo usuario.
- Migrar frontend para React/Vite.
