# Próxima Feature

## Nome

**Valid-Window Recall Tests v0.1** — testes determinísticos para filtragem de
validade temporal `valid_from` / `valid_until`.

## Por que importa

Memórias e documentos suportam timestamps `valid_from` / `valid_until` para que
fatos com janela de tempo (ex: ofertas promocionais, configuração sazonal) sejam
automaticamente excluídos do recall quando expiram. O filtro existe mas não há
uma suite de regressão focada. Um bug aqui poderia expor dados expirados a
prompts de LLM ou ocultar dados atualmente válidos.

## Comportamento

Sem novo código de produção. A suite de testes cobrirá:

1. Memória com `valid_until` no passado → excluída do recall.
2. Memória com `valid_until` no futuro → incluída.
3. Memória com `valid_from` no futuro → excluída até esse momento.
4. Query backdatada via `as_of` → recupera estado em timestamp passado.
5. Caso extremo na fronteira: `valid_until == query_time` → excluída (estritamente menor).
6. Memória sem janela de validade → sempre incluída.

## Arquivos

- `crates/hippocore/tests/valid_window.rs` — novo arquivo de testes dedicado.
- `docs/en/VALID_WINDOW_RECALL.md` e `docs/pt-br/VALID_WINDOW_RECALL.md`.
- `docs/en/STATUS.md` e `docs/pt-br/STATUS.md`.

## Critérios de aceite

- Mínimo de 5 testes de integração determinísticos usando timestamps epoch-ms
  explícitos.
- Quality gate:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Fora de escopo

- Expiração / evicção automática.
- Mudanças no código de produção.
- Modo server.
