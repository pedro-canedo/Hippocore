# Proxima Feature

## Nome da feature

**SQL Editor Workbench v0.1**

## Por que importa

O SQL Editor executa queries, mas ainda se comporta como um textarea grande.
Desenvolvedores precisam de um workbench eficiente que ensine o mapeamento de
payload, preserve trabalho recente e facilite inspecao ou exportacao.

## Comportamento

- Dar ao editor toolbar de codigo com Run e suporte a `Ctrl/Cmd + Enter`.
- Manter a query atual no estado da pagina e fazer exemplos preencherem o editor
  sem perder o tenant.
- Armazenar as ultimas 20 queries bem-sucedidas localmente por diretorio e
  tenant; selecionar historico restaura texto sem executar automaticamente.
- Renderizar campos do payload como colunas deterministas, com abas Table e JSON.
- Mostrar erros de validacao estruturados e duracao observada no cliente.
- Adicionar Copy JSON e Export JSON para resultados bem-sucedidos.
- Explicar que campos sem prefixo mapeiam para `payload.<field>` com exemplo curto.

## Arquivos provaveis

- `crates/hippocore-server/src/admin/app.js`
- `crates/hippocore-server/src/admin/styles.css`
- `crates/hippocore-server/tests/server_integration.rs`
- docs bilingues de status/interface e `CHANGELOG.md`

## Criterios de aceitacao

- Botao Run e atalho executam o mesmo request tenant-scoped.
- Exemplos e historico restauram texto sem executar automaticamente.
- Resultados mostram colunas de payload em ordem determinista e JSON completo.
- Copy/export usam exatamente o resultado atual.
- Historico e limitado, local e escopado por banco e tenant.
- Todo texto novo esta em ingles e portugues; portao verde.

## Testes

- Assertions de assets cobrem atalho, escopo/limite do historico, colunas de
  payload, copy/export e erros.
- Testes existentes de parser, executor e isolamento SQL continuam verdes.
- Check JavaScript e portao Rust completo passam.

## Fora de escopo

- Expandir gramatica SQL ou adicionar escrita.
- Historico no servidor, queries salvas, paginacao ou export CSV.
- Monaco/CodeMirror, dependencias npm ou migracao React/Vite.
