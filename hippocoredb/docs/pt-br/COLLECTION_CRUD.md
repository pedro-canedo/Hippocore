# Collection CRUD Tests v0.1

## Visão geral

Uma suite de testes dedicada cobrindo o ciclo de vida completo de coleções:
criação, listagem, idempotência, isolamento entre tenants e recall escopado a
coleções inexistentes. Nenhum código de produção foi alterado; esta feature é
puramente testes aditivos.

## Arquivo de testes

`crates/hippocore/tests/collection_crud.rs`

## O que é testado

| Teste | Cenário |
|---|---|
| `created_collection_appears_in_list` | Após `create_collection`, aparece em `collections(tenant)` |
| `duplicate_collection_is_idempotent` | Chamar `create_collection` com o mesmo nome tem sucesso e retorna a coleção existente; apenas uma entrada existe |
| `tenants_can_share_collection_names` | Dois tenants podem ter coleções com o mesmo nome sem conflito |
| `collection_list_is_tenant_isolated` | `collections(tenant_id)` nunca retorna coleções de outro tenant |
| `recall_scoped_to_nonexistent_collection_returns_empty` | Recall com `collection = Some("inexistente")` retorna vazio, não erro |
| `collection_description_is_stored` | Descrição persiste após criação |

## Invariantes de design confirmados

- `create_collection` é idempotente: chamá-la duas vezes com o mesmo par
  `(tenant_id, name)` retorna a coleção existente e não cria uma duplicata.
  Callers podem usar como upsert seguro sem verificar existência previamente.
- A listagem `collections(tenant_id)` é filtrada por `tenant_id`; coleções de
  outros tenants nunca aparecem.
