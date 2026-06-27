# Proxima Feature

## Nome da feature

**Data Explorer Workspace v0.1**

## Por que importa

O Data Explorer possui abas logica, JSON e fisica, mas ainda se comporta como
uma pagina vazia de records. Um desenvolvedor deve selecionar tenant, navegar
collections e tipos, inspecionar payloads como colunas e abrir detalhes completos
sem entender primeiro a arquitetura interna do Hippocore.

## Comportamento

- Adicionar busca de collection e acao contextual para criar collection na arvore.
- Mostrar Records, Memories, Documents e Files dentro da collection selecionada,
  com contagens derivadas das listas carregadas no escopo do tenant.
- Selecionar um tipo carrega e exibe esse tipo sem sair do Explorer.
- Renderizar chaves do payload de records como colunas logicas, mantendo ids e
  nome da table visiveis.
- Manter Raw JSON para objetos persistidos completos e Physical Info para
  lineage, kind, timestamps, source, metadata, referencias e versao.
- Empty states orientam criacao de collection ou ingestao do primeiro item.

## Arquivos provaveis

- `crates/hippocore-server/src/admin/app.js`
- `crates/hippocore-server/src/admin/styles.css`
- `crates/hippocore-server/tests/server_integration.rs`
- docs bilingues de status/interface e `CHANGELOG.md`

## Criterios de aceitacao

- Explorer navega tenant -> collection -> tipo de dado em uma tela.
- Busca filtra collections sem mudar o escopo do tenant.
- Campos do payload aparecem como colunas logicas em ordem determinista.
- Views JSON e fisica expoem dados completos sem misturar detalhes de storage na
  tabela logica padrao.
- Todo texto novo esta completo em ingles e portugues.
- Auth e isolamento permanecem inalterados; portao de qualidade verde.

## Testes

- Assertions de assets cobrem busca, selecao de tipo, colunas de payload e tres views.
- Testes existentes de isolamento e listas admin continuam verdes.
- Check JavaScript e portao Rust completo passam.

## Fora de escopo

- Editar ou excluir objetos arbitrarios pela tabela do Explorer.
- Novos endpoints backend de agregacao/contagem.
- Tabelas virtualizadas, migracoes de schema ou migracao React/Vite.
