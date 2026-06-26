# RAG Audit Engine

A Fase 9 adiciona uma trilha local de auditoria para montagem de contexto.

Cada chamada `Hippocore::build_context(req)` anexa um objeto JSON em
`<data_dir>/audit.log`. O registro captura:

- `timestamp_ms`
- `tenant_id`
- `query` original
- `mode` de recall
- `collection` opcional
- `max_tokens` solicitado
- `token_count` final
- `latency_ms` da montagem de contexto antes da persistência da auditoria
- `items_dropped`
- ids, tipos, coleções, scores, confiança, tokens e status de inclusão dos
  itens recuperados
- `inclusion_source` (`recalled`, `graph_expanded` ou `not_included`)

O log de auditoria é histórico operacional append-only. Ele fica separado de
`wal.log` de propósito, porque uma query não altera o estado vivo do banco.

## API

```rust
let block = db.build_context(req)?;
let records = db.query_audit(0, i64::MAX, "acme")?;
```

`query_audit(from_ms, to_ms, tenant_id)` retorna registros de um único tenant em
uma janela inclusiva de epoch milliseconds.

## CLI

```bash
hippocore build-context --db ./data --tenant acme --query "postgresql python" --json
hippocore audit --db ./data --tenant acme --from 0 --to 9223372036854775807 --json
```

Sem `--json`, `audit` imprime um resumo compacto para humanos.

## Feedback de Confiança

A API `rate_memory` e o comando `rate-memory` existentes fazem parte desse
ciclo de auditoria. A confiança é persistida em `Memory` e aparece em
`RecallResult`, `ContextItem` e itens de auditoria.

## Fora de Escopo

- Calibração automática de confiança.
- Grafo completo de proveniência.
- Ingestão de auditoria via HTTP/server.
- Política de retenção ou rotação de auditoria.
