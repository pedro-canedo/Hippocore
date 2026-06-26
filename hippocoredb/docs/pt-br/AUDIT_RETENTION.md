# Audit Retention v0.1

## Visão geral

O RAG Audit Engine registra cada chamada de `build_context` em `audit.log`
como um arquivo JSON-lines append-only. Sem política de retenção, o log cresce
sem limite em aplicações embedded de longa duração.

O Audit Retention v0.1 fornece:

- **Limites na configuração** (`audit_max_records`, `audit_max_bytes`).
- **`compact_audit(max_records, max_bytes)`** — reescreve `audit.log`
  atomicamente, preservando apenas os registros mais recentes.
- **Aplicação automática** — retenção é executada após cada append de
  `build_context` quando um limite está configurado.
- **Comando CLI** — `compact-audit`.
- **Visibilidade em stats** — `DatabaseStats` agora expõe `audit_log_bytes` e
  `audit_records`.

## Configuração

```rust
use hippocore::Config;

let mut cfg = Config::new("./data");
cfg.audit_max_records = 1000;       // manter no máximo 1000 registros
cfg.audit_max_bytes   = 10_000_000; // manter no máximo ~10 MB
```

Ambos os campos têm valor padrão `0` (ilimitado). Quando ambos são não-zero, a
restrição mais restritiva prevalece (menor número de registros mantidos).

## API da biblioteca

```rust
// Compactação manual — usa valores da configuração quando Some(…) é None; ou
// desativa uma restrição para esta chamada com Some(0).
let summary = db.compact_audit(Some(100), None)?;
println!(
    "mantidos {} registros, removidos {}, {} → {} bytes",
    summary.records_kept,
    summary.records_removed,
    summary.bytes_before,
    summary.bytes_after,
);

// Stats incluem informações do audit log.
let stats = db.stats()?;
println!("registros de auditoria: {}, bytes: {}", stats.audit_records, stats.audit_log_bytes);
```

## CLI

```bash
# Compactar mantendo no máximo 200 registros (saída humana).
hippocore compact-audit --db ./data --max-records 200

# Compactar mantendo no máximo 5 MB, emitir JSON.
hippocore compact-audit --db ./data --max-bytes 5242880 --json
```

Saída JSON:

```json
{
  "records_kept": 45,
  "records_removed": 155,
  "bytes_before": 204800,
  "bytes_after": 18432
}
```

## Invariantes

- O comportamento padrão permanece inalterado quando nenhum limite de
  configuração está definido e `compact_audit` não é chamado.
- Apenas os registros mais recentes são mantidos; a ordem é sempre preservada.
- A reescrita é atômica: `audit.tmp` é escrito e fsynced, depois renomeado
  sobre `audit.log`.
- `query_audit` continua funcionando após a compactação.
- A compactação automática após append é best-effort: a chamada de
  `build_context` tem sucesso mesmo que a compactação falhe.

## Fora de escopo (esta versão)

- Export remoto de auditoria.
- Compressão ou criptografia do log.
- Busca cruzando múltiplos arquivos de auditoria.
- Modo server/HTTP.
