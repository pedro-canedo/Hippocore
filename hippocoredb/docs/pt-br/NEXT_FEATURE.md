# Próxima Feature

## Nome

**Hybrid Search Ranking Tests v0.1** — testes determinísticos verificando que
`SearchMode::Hybrid` mistura corretamente scores vetoriais e textuais.

## Por que importa

O caminho de busca híbrida funde scores TF-IDF com scores de cosseno vetorial.
Se a fusão ou normalização estiver errada, correspondências puramente textuais
ou puramente semânticas podem ser suprimidas silenciosamente. Nenhum teste
atualmente bloqueia a garantia de ordenação relativa: uma memory que corresponde
tanto ao texto quanto ao vetor deve classificar acima de uma que só corresponde
ao texto.

## Comportamento

Sem novo código de produção. A suite de testes verificará:

1. `SearchMode::Vector` retorna uma memory semanticamente próxima que não
   compartilha palavras-chave exatas com a query.
2. `SearchMode::Text` retorna uma memory que compartilha palavras exatas com a
   query.
3. `SearchMode::Hybrid` retorna ambos os tipos; o conjunto de resultados é um
   superconjunto dos hits de `Vector` e `Text` quando esses hits existem.
4. Uma memory que corresponde exatamente à query aparece nos 3 primeiros
   resultados em modo `Hybrid`.
5. Mudar de `Hybrid` para `Vector` não retorna um resultado cujo texto é
   exatamente a query mas semanticamente não relacionado.

## Arquivos

- `crates/hippocore/tests/hybrid_search.rs` — novo arquivo de testes dedicado.
- `docs/en/HYBRID_SEARCH.md` e `docs/pt-br/HYBRID_SEARCH.md`.
- `docs/en/STATUS.md` e `docs/pt-br/STATUS.md`.

## Critérios de aceite

- Mínimo de 5 testes de integração determinísticos usando `TempDir`.
- Quality gate:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Fora de escopo

- Ajuste de pesos de fusão.
- Modo server.
- Mudanças no código de produção.
