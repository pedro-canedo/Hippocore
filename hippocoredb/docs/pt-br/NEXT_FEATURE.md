# Próxima Feature

## Nome

**Collection CRUD Tests v0.1** — testes determinísticos para o ciclo de vida
completo de coleções.

## Por que importa

As coleções são a unidade primária de escopo para todos os itens (memories,
documentos, records). Seu ciclo de vida de criação/listagem/deleção e isolamento
entre tenants são exercidos incidentalmente por muitos testes existentes, mas
não há uma suite focada cobrindo:

- Nomes de coleção duplicados dentro de um tenant retornam erro.
- Coleções de tenants diferentes não vazam na lista um do outro.
- Deletar uma coleção remove todos os seus itens do índice de retrieval.
- Consultar itens de uma coleção deletada retorna resultados vazios.

## Comportamento

Sem novo código de produção. A suite de testes cobrirá:

1. Criar, depois listar coleções de um tenant — coleção aparece.
2. Criar nome de coleção duplicado dentro do mesmo tenant retorna erro.
3. Dois tenants podem ter coleções com o mesmo nome sem interferência.
4. Após deletar uma coleção, `list_collections` não a mostra mais.
5. Após deletar uma coleção, recall escopado a ela retorna vazio.

## Arquivos

- `crates/hippocore/tests/collection_crud.rs` — novo arquivo de testes dedicado.
- `docs/en/COLLECTION_CRUD.md` e `docs/pt-br/COLLECTION_CRUD.md`.
- `docs/en/STATUS.md` e `docs/pt-br/STATUS.md`.

## Critérios de aceite

- Mínimo de 5 testes de integração determinísticos.
- Quality gate:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Fora de escopo

- Atualizações de metadata ou descrição no nível de coleção.
- Modo server.
- Mudanças no código de produção.
