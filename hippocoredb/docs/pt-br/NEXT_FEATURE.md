# Próxima Feature

## Nome

**build_context Smoke Tests v0.1** — testes determinísticos verificando que
`build_context` monta um bloco de contexto utilizável para LLM.

## Por que importa

`build_context` é o caminho de saída primário para consumidores de IA — ele
classifica, aparar e serializa itens em uma string pronta para prompt. Não há
testes dedicados verificando: (a) a string de contexto não é vazia quando itens
relevantes existem, (b) `max_tokens` é respeitado, (c) itens relacionados
expandidos pelo grafo aparecem quando `include_related = true`. Uma regressão
em qualquer desses caminhos degrada silenciosamente as saídas de IA.

## Comportamento

Sem novo código de produção. A suite de testes cobrirá:

1. Uma memory → `build_context` produz uma string `text` não vazia.
2. A string de contexto contém um snippet reconhecível da memory.
3. `max_tokens` baixo → menos itens incluídos do que disponíveis.
4. `include_related = true` com aresta de grafo → item relacionado aparece em
   `items_included`.
5. Store vazio → `build_context` retorna `text = ""` (não erro).

## Arquivos

- `crates/hippocore/tests/build_context_smoke.rs` — novo arquivo de testes.
- `docs/en/BUILD_CONTEXT_SMOKE.md` e `docs/pt-br/BUILD_CONTEXT_SMOKE.md`.
- `docs/en/STATUS.md` e `docs/pt-br/STATUS.md`.

## Critérios de aceite

- Mínimo de 5 testes de integração determinísticos usando `TempDir`.
- Quality gate:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Fora de escopo

- Mudanças no formato de contexto customizado.
- Modo server.
- Mudanças no código de produção.
