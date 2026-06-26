# Próxima Feature

## Nome

**Multi-tenant Query Isolation Audit v0.1** — suite de testes determinística
provando que nenhum resultado de recall, search ou `build_context` cruza
fronteiras de tenant.

## Por que importa

O isolamento de tenants é um invariante de correção central (aplicado na camada
de query), mas não há uma suite de testes dedicada que o verifique
explicitamente. À medida que novos caminhos de query, passes de mesclagem e
lógica de expansão por grafo são adicionados, um teste ausente poderia
silenciosamente permitir um vazamento de dados entre tenants. Uma auditoria de
isolamento abrangente fecha essa lacuna.

## Comportamento

Nenhum código de produção novo é necessário — esta feature é puramente testes
aditivos:

1. Criar pelo menos 2 tenants, cada um com nomes de coleção correspondentes.
2. Armazenar conteúdo semanticamente idêntico em ambos os tenants.
3. Para cada combinação de:
   - `recall()` / `search()` / `build_context()`
   - `SearchMode::Vector`, `Text`, `Hybrid`
   - `include_related = true/false` (para `build_context`)
4. Afirmar: cada item resultado tem `tenant_id` igual ao tenant consultado.
5. Afirmar: nenhum item do outro tenant aparece em nenhum resultado.
6. Afirmar: `add_graph_edge` e `list_graph_edges` também são isolados por tenant.

## Arquivos

- `crates/hippocore/tests/tenant_isolation.rs` — novo arquivo de testes dedicado.
- `docs/en/TENANT_ISOLATION_AUDIT.md` e `docs/pt-br/TENANT_ISOLATION_AUDIT.md`.
- `docs/en/STATUS.md` e `docs/pt-br/STATUS.md`.

## Critérios de aceite

- Mínimo de 6 asserções de isolamento direcionadas (um por caminho de query ×
  modo).
- Sem mudanças no código de produção.
- Quality gate:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Fora de escopo

- Novo enforcement de tenant no código de produção.
- Modo server ou isolamento multi-nó.
- Modelo de permissões / RBAC.
