# Próxima Feature

## Nome

**Context Compiler** — `build_context(query, user, max_tokens)`

## Por que importa

Após armazenar memórias, documentos e registros, o caso de uso mais comum de
agentes de IA é montar uma string de contexto pronta para prompt: pegar os
top-k resultados de recall de todos os tipos de itens, ranqueá-los, cortar para
um budget de tokens e formatá-los para que um LLM possa raciocinar sobre eles.
Hoje cada aplicação precisa fazer isso manualmente — escolher quais itens incluir,
quantos tokens cada um ocupa e em que ordem devem aparecer.

Um Context Compiler integrado remove esse código repetitivo e fornece:
- Um seletor com budget de tokens que preenche o budget greedy por score.
- Uma chamada de API única que substitui o loop recall + format na maioria dos agentes.
- Um valor `ContextBlock` com a string montada e metadados de proveniência (itens
  incluídos, tokens usados, itens descartados).

## Comportamento esperado

### Nova API

```rust
pub struct BuildContextRequest {
    pub tenant_id: String,
    pub query: String,
    pub user_id: Option<String>,
    pub max_tokens: usize,           // teto rígido; padrão 2048
    pub top_k_candidates: usize,     // recall de até este número; padrão 20
    pub mode: SearchMode,            // padrão Hybrid
    pub collection: Option<String>,
    pub metadata_filter: Metadata,
}

pub struct ContextBlock {
    pub text: String,                // contexto montado, pronto para LLM
    pub token_count: usize,
    pub items_included: Vec<ContextItem>,
    pub items_dropped: usize,
}

pub struct ContextItem {
    pub id: String,
    pub kind: ItemKind,
    pub score: f32,
    pub token_count: usize,
    pub snippet: String,             // primeiros 120 chars do conteúdo
}
```

### Algoritmo

1. Executa recall (híbrido por padrão) para até `top_k_candidates` itens.
2. Ordena por score decrescente.
3. Adiciona itens greedily (maior score primeiro) até que `max_tokens` seria
   excedido.
4. Formata: cada item incluído vira `[<kind>:<id>] <text>` separado por `\n\n`.
5. Retorna `ContextBlock` com texto montado, proveniência e contagem de
   descartados.

### Contagem de tokens

Aproximação integrada: 1 token ≈ 4 bytes UTF-8. Quem precisar de tokenização
exata pode pós-processar; a aproximação é suficiente para enforcement de budget
sem dependência de tokenizador.

### CLI

```
hippocore build-context --tenant <t> --query "..." [--max-tokens 2048]
  [--top-k 20] [--mode hybrid] [--collection c] [--json]
```

## Critérios de aceite

- `Hippocore::build_context(req)` retorna `ContextBlock`.
- Budget de tokens respeitado (texto nunca excede `max_tokens * 4` bytes além
  do tamanho do último item incluído).
- Itens ranqueados por score de recall (maior primeiro).
- `ContextBlock.items_included` lista cada item com id, kind, score, token
  count e snippet.
- `ContextBlock.items_dropped` conta itens buscados mas que não couberam.
- Subcomando CLI `build-context` funciona; saída `--json` é parseável.
- Mínimo 3 testes de integração: enforcement de budget, montagem multi-item,
  round-trip de saída JSON.
- Todos os 82+ testes passam; sem novas dependências externas.

## Fora de escopo

- Integração com tokenizador externo (tiktoken etc.) — usar aproximação por bytes.
- Templating de prompt ou injeção de few-shot.
- Server/HTTP mode.

## Follow-up

Fase 8 concluída → Fase 9: **RAG Audit Engine** — rastreamento de proveniência,
scores de confiança por item e log de auditoria no momento da query.
