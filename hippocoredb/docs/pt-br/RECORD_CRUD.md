# Record CRUD Tests v0.1

## Visão geral

Uma suite de testes dedicada cobrindo o ciclo de vida completo da entidade
`Record`: armazenar, recuperar por id, aparecer no recall, incremento de versão
na atualização, deletar e isolamento entre tenants. Nenhum código de produção
foi alterado; esta feature é puramente testes aditivos.

## Arquivo de testes

`crates/hippocore/tests/record_crud.rs`

## O que é testado

| Teste | Cenário |
|---|---|
| `stored_record_is_retrievable_by_id` | `put_record` → `get_record` retorna o mesmo id |
| `record_appears_in_recall` | Record armazenado aparece em `recall()` para query compatível |
| `update_increments_version` | Re-`put_record` com mesmo id incrementa `version` de 0 para 1 |
| `delete_record_returns_none_on_get` | Após `delete_record`, `get_record` retorna `None` |
| `deleted_record_absent_from_recall` | Após deleção, record não aparece em `recall()` |
| `records_are_tenant_isolated` | Um tenant não pode recuperar ou fazer recall de records de outro tenant |

## Invariantes de design confirmados

- `version` inicial é `0`; cada atualização incrementa em 1.
- `delete_record` é durável — o record desaparece tanto na busca por id quanto
  no recall.
- Isolamento de tenant se aplica tanto ao `get_record` (busca por id) quanto ao
  `recall` (busca vetorial/textual).
