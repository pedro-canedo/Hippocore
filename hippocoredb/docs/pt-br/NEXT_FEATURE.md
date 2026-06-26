# Próxima Feature

## Nome

**Metadata Filter Regression Suite v0.1** — testes determinísticos cobrindo
combinações de filtros de metadata exatos para todos os tipos de item.

## Por que importa

O filtro de metadata é o mecanismo principal para escopar o recall a um usuário,
sessão ou contexto. Embora a lógica de filtro exista, não há uma suite de testes
focada que cubra todos os tipos de item × combinações de filtro × casos extremos.
Uma lacuna aqui arrisca regressões silenciosas quando as camadas de query ou
storage mudarem.

## Comportamento

Sem novo código de produção. A suite de testes cobrirá:

1. Filtro de correspondência exata (`key == value`) em memories, chunks de
   documento e records.
2. Filtros multi-chave (todas as chaves devem corresponder, semântica AND).
3. Um filtro que não corresponde a nenhum item (resultado vazio, não um erro).
4. Um filtro escopado a uma coleção específica dentro de um tenant.
5. Verificar que filtrar em um tenant não afeta outro tenant com as mesmas
   chaves e valores de metadata.
6. `build_context` com filtro de metadata respeita o filtro (sem vazamento).

## Arquivos

- `crates/hippocore/tests/metadata_filter.rs` — novo arquivo de testes dedicado.
- `docs/en/METADATA_FILTER_REGRESSION.md` e
  `docs/pt-br/METADATA_FILTER_REGRESSION.md`.
- `docs/en/STATUS.md` e `docs/pt-br/STATUS.md`.

## Critérios de aceite

- Mínimo de 6 testes de integração determinísticos.
- Quality gate:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Fora de escopo

- Novos operadores de filtro (prefixo, range, regex).
- Modo server.
- Mudanças no código de produção.
