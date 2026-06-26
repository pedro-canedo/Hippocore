# Contradiction Advisory Tests v0.1

## Visão geral

Uma suite de testes dedicada cobrindo o comportamento completo do sistema de
advisory de contradição: ambas as memórias em contradição aparecem no recall, o
advisory é populado em `RecallResult.contradictions`, e `build_context` aplica
re-ranking ciente de confiança quando há contradições.

Nenhum código de produção foi alterado; esta feature é puramente testes aditivos.

## Arquivo de testes

`crates/hippocore/tests/contradiction.rs`

## O que é testado

| Teste | Cenário |
|---|---|
| `contradicting_memories_both_appear_in_recall` | A e B ambas são retornadas; contradição NÃO suprime nenhum item |
| `contradictions_advisory_populated_in_recall_result` | `RecallResult.contradictions` de B contém o id de A |
| `build_context_confidence_reranking_prefers_higher_confidence` | B (confidence 0.95) é ranqueada acima de A (confidence 0.1) quando há contradições |
| `build_context_no_contradiction_no_reranking` | Sem contradições → sem re-ranking por confiança; todos os itens aparecem no contexto |

## Invariantes de design confirmados

- A anotação `contradicts` é puramente advisory: ambos os itens são retornados
  por `recall()` independentemente da confiança.
- `build_context` ativa o re-ranking ciente de confiança apenas quando pelo
  menos um candidato carrega uma lista `contradictions` não-vazia.
- Sem contradições, o passo extra de re-ranking é ignorado e a ordenação do
  recall é preservada.
