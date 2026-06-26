# Próxima Feature

## Nome

**Contradiction Advisory Tests v0.1** — testes determinísticos cobrindo o
comportamento completo do sistema de advisory `contradicts` / `contradictions`.

## Por que importa

O campo `contradicts` em uma memória permite que callers sinalizem que duas
memórias estão em conflito. Quando presente, `build_context` aplica re-ranking
ciente de confiança para preferir o item de maior confiança. Não há uma suite de
regressão focada que verifique:

- O advisory `contradictions` aparece em `RecallResult` para pares sinalizados.
- Ambos os itens ainda são retornados (o advisory NÃO suprime itens).
- O re-ranking ciente de confiança é ativado quando há contradições.
- O item de maior confiança é ranqueado acima do de menor confiança.

## Comportamento

Sem novo código de produção. A suite de testes cobrirá:

1. Armazenar memória A (confidence 0.3) e B (confidence 0.9) onde B contradiz A.
2. Recuperar ambas; afirmar que o resultado de A tem `contradictions = [id_b]`
   e vice-versa.
3. Nem A nem B está ausente dos resultados de recall (apenas advisory, não filtro).
4. Chamar `build_context`; afirmar que B aparece antes de A (maior confiança
   vence).
5. Chamar `build_context` com `confidence = None` em ambas; afirmar que a
   ordenação não muda em relação ao recall simples.

## Arquivos

- `crates/hippocore/tests/contradiction.rs` — novo arquivo de testes dedicado.
- `docs/en/CONTRADICTION_ADVISORY.md` e `docs/pt-br/CONTRADICTION_ADVISORY.md`.
- `docs/en/STATUS.md` e `docs/pt-br/STATUS.md`.

## Critérios de aceite

- Mínimo de 4 testes de integração determinísticos.
- Quality gate:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Fora de escopo

- Detecção automática de contradição (NLI, comparação semântica).
- Mudanças no código de produção do modelo de contradição.
- Modo server.
