# Próxima Feature

## Nome da feature

**Ingestion & Recall UX Hardening v0.1**

## Por que importa

A pagina Ingestion & Recall agora funciona de ponta a ponta, mas a experiencia
ainda e baseada em formularios simples. Usuarios que nao sabem qual collection
usar precisam sair da pagina para verificar. Adicionar autocomplete de collection
(populado da lista carregada), renderizacao de cards de resultado de recall
(em vez de dump JSON bruto) e contagem de tokens para build_context tornara
o ciclo ingestao → recall muito mais fluido sem mudancas no servidor.

## Comportamento

- **Autocomplete de collection**: campos de collection nas secoes Store e Recall
  auto-populam com `<datalist>` de `state.collections`.
- **Cards de resultado de recall**: substituir `JsonViewer` por uma lista de cards
  estruturados com score, badge de tipo, matched_terms, snippet de texto e
  botao de copiar id.
- **Resultado de build_context**: mostrar contagem de tokens, badges de
  items incluidos/descartados e texto formatado em bloco `<pre>` legivel.
- **Selecao de tipo por radio**: chaves Memory/Document/Record/File devem
  mostrar/ocultar campos relevantes (ex: nome da tabela apenas para Record).

## Arquivos provaveis

- `crates/hippocore-server/src/admin/app.js` (IngestionRecallPage, cards)
- `crates/hippocore-server/src/admin/styles.css` (estilos de card de recall)
- `docs/en/STATUS.md`
- `docs/pt-br/STATUS.md`
- `CHANGELOG.md`

## Criterios de aceitacao

- Input de collection exibe datalist com collections conhecidas para o tenant selecionado.
- Resultados de recall renderizam como cards, nao JSON bruto (exceto aba JSON).
- Resultado de build_context mostra contagem de tokens + items incluidos/descartados + texto.
- Selecao de tipo por radio exibe/oculta o campo de tabela dinamicamente.
- Toda a funcionalidade existente preservada; portao de qualidade verde.

## Fora de escopo

- UI de upload de arquivo (drag-and-drop e feature separada).
- Ingestao em lote (API batch disponivel mas UI fora de escopo aqui).
- Streaming em tempo real de progresso de ingestao.
