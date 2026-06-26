# Próxima Feature

## Nome

**Audit Retention v0.1** — retenção local limitada para `audit.log`.

## Por que importa

O RAG Audit Engine registra cada chamada de `build_context`, e Graph-Aware
Context adiciona entradas de auditoria mais ricas. Sem política de retenção,
`audit.log` pode crescer sem limite em aplicações embedded de longa duração. O
próximo incremento de maior valor é manter auditabilidade local e determinística
com uma forma simples de limitar uso de disco.

Audit Retention v0.1 deve adicionar:

- Opções de configuração para retenção por máximo de registros e/ou máximo de
  bytes.
- API `compact_audit()` ou `rotate_audit()` que reescreve `audit.log`
  atomicamente preservando os registros mais novos.
- Aplicação automática da retenção após append de auditoria quando configurada.
- Comando CLI `compact-audit --db <path> [--json]`.
- Visibilidade em status/stats para bytes do audit log e quantidade de registros
  retidos.

## Critérios de aceite

- Comportamento default permanece igual: nenhum registro é removido sem política
  configurada ou chamada explícita a `compact-audit`.
- Retenção mantém os registros mais novos e preserva JSON-lines válido.
- Reescrita é atômica: arquivo temporário, fsync, rename.
- `query_audit` continua funcionando após retenção.
- CLI `compact-audit --json` emite resumo parseável.
- Mínimo de 4 testes determinísticos: default inalterado, retenção por máximo de
  registros, retenção por máximo de bytes ou compactação manual, query após
  compactação.
- Documentação atualizada em inglês e português.
- Quality gate:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Fora de escopo

- Export remoto de auditoria.
- Compressão.
- Criptografia.
- Server/HTTP mode.
- Busca cruzando múltiplos arquivos de auditoria.

## Follow-up

Após Audit Retention v0.1, avaliar ranking de recall ciente de grafo versus
controles admin/studio mais amplos para dados de grafo e auditoria.
