# Próxima Feature

## Nome

**RAG Audit Engine** — scoring de confiança por item e log de auditoria no
momento da query.

## Por que importa

O Context Compiler (Fase 8) monta contexto pronto para LLM, mas depois que uma
resposta é gerada não há forma integrada de rastreá-la até os itens de origem,
pontuar sua confiabilidade ou registrar o que foi recuperado e quando. Em
sistemas RAG de produção isso é uma lacuna crítica: engenheiros de suporte
precisam saber *por que* o agente disse X, e equipes de confiabilidade precisam
detectar drift ou baixa qualidade de retrieval antes de gerar incidentes.

O RAG Audit Engine adiciona:
- Campo `confidence: Option<f32>` em `Memory` e `RecallResult` para que fatos
  carreguem um sinal explícito de confiabilidade.
- Log de auditoria append-only (`<data_dir>/audit.log`) onde cada chamada
  `build_context` é registrada: timestamp, query, tenant, itens recuperados,
  scores, tokens.
- API `query_audit(from_ms, to_ms, tenant_id)` para reproduzir o que foi
  recuperado em uma janela de tempo.
- API `rate_memory(id, confidence)` para feedback humano-no-loop.
- CLI: `audit --from <ms> --to <ms> --tenant <t> [--json]` e
  `rate-memory --id <id> --confidence <0.0-1.0>`.

## Critérios de aceite

- `Memory` e `RecallResult` ganham `confidence: Option<f32>` com serde default
  compatível (`None`).
- Cada chamada `build_context` escreve um registro JSON-lines de auditoria
  atomicamente.
- `query_audit(from_ms, to_ms, tenant_id)` retorna `Vec<AuditRecord>`.
- `rate_memory` valida que confiança está em `[0.0, 1.0]` e armazena durável.
- Comandos CLI `audit` e `rate-memory` funcionam; saída `--json` é parseável.
- Mínimo 4 testes de integração.
- Todos os 85+ testes passam; sem novas dependências externas.

## Fora de escopo

- Calibração automática de confiança (humano/agente define; engine registra).
- Grafo completo de proveniência (Fase 10 — Graph Memory).
- Server/HTTP mode.

## Follow-up

Fase 9 concluída → Fase 10: **Graph Memory** — arestas de relacionamento entre
itens e recall com consciência de grafo.
