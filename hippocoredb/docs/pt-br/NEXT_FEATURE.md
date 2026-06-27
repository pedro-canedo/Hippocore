# Proxima Feature

## Nome da feature

**Guided Workspace Onboarding v0.1**

## Por que importa

O Dashboard mostra onboarding apenas quando nao ha tenants e depois vira uma
grade de estatisticas. Desenvolvedores novos precisam de um caminho persistente
e orientado por estado entre banco vazio, primeira query e primeiro recall sem
saber antecipadamente qual pagina ou tipo de objeto usar.

## Comportamento

- Mostrar checklist com cinco passos: criar tenant, criar collection, adicionar
  primeiro dado, executar SQL e testar recall/contexto.
- Derivar tenant, collection e dados dos stats; registrar sucesso de SQL e recall
  localmente apenas para progresso da UI.
- Dar ao primeiro passo incompleto uma acao primaria para o fluxo correto e
  manter passos concluidos visualmente marcados.
- Separar status operacional, tenant ativo, storage e health da API das
  contagens de dados e acoes rapidas.
- Retirar JSON bruto do foco do Dashboard e apontar para Observabilidade.
- Exibir a proxima acao de setup na sidebar enquanto o onboarding estiver incompleto.

## Arquivos provaveis

- `crates/hippocore-server/src/admin/app.js`
- `crates/hippocore-server/src/admin/styles.css`
- `crates/hippocore-server/tests/server_integration.rs`
- docs bilingues de status/interface e `CHANGELOG.md`

## Criterios de aceitacao

- Usuario novo sempre ve uma proxima acao clara ate concluir os cinco passos.
- Progresso atualiza apos tenant, collection, ingestao, SQL e recall com sucesso.
- JSON bruto deixa de ser painel primario do Dashboard.
- Stats, navegacao, auth e isolamento existentes permanecem inalterados.
- Todo texto novo esta em ingles e portugues; portao de qualidade verde.

## Testes

- Assertions de assets cobrem derivacao de passos, rota da proxima acao e flags
  locais de SQL/recall.
- Testes de integracao admin existentes continuam verdes.
- Check JavaScript e portao Rust completo passam.

## Fora de escopo

- Estado de onboarding por usuario persistido no servidor ou analytics.
- Tour overlay, command palette ou migracao de framework.
- Alterar schemas, retrieval ou persistencia.
