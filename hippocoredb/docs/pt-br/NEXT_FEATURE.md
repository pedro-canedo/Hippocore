# Próxima Feature

## Nome

**Temporal Truth Layer spec completa** — relações `supersedes` e `contradicts`
entre memórias, detecção de conflitos no recall e resolução com confiança.

## Por que importa

A Fase 7 v0.1 adicionou `valid_from`/`valid_until` e filtro `as_of` básico.
A spec completa do Temporal Truth Layer adiciona a camada semântica: agentes de
IA frequentemente armazenam fatos que se contradizem ou se atualizam, e sem
metadados explícitos de `supersedes`/`contradicts` o Hippocore não consegue
surfaçar ou resolver esses conflitos. Sem isso, um conjunto de resultados de
recall pode conter tanto uma crença antiga quanto um fato mais novo que a
contradiz, deixando para o agente resolver — ou pior, usando informação obsoleta
silenciosamente.

Relações explícitas de supersedura/contradição permitem:
- Marcar uma nova memória como substituindo uma mais antiga.
- Sinalizar duas memórias como conflitantes (revisão necessária por humano ou
  agente).
- Excluir memórias supersedidas dos resultados padrão de recall.
- Retornar avisos de contradição junto aos resultados de recall.

## Comportamento esperado

### Mudanças no modelo

`Memory` ganha dois campos opcionais:
```
supersedes:   Vec<String>  // ids de memórias que esta substitui
contradicts:  Vec<String>  // ids de memórias que esta contradiz
```

### Comportamento na escrita

`remember` valida que qualquer id listado em `supersedes`/`contradicts` existe
no mesmo tenant. Se uma memória é supersedida, ela recebe `superseded_by = <id>`
(armazenado na memória antiga) e é excluída do recall padrão.

### Comportamento na query

- Memórias supersedidas são excluídas do recall padrão (exceto se
  `include_superseded = true` estiver no request).
- Quando uma memória recall tem `contradicts` não vazio, o `RecallResult`
  carrega um campo `contradictions: Vec<String>` informativo.

### Mudanças na CLI

- `remember` ganha `--supersedes <id>` (repetível) e `--contradicts <id>`
  (repetível).
- `recall` ganha `--include-superseded` para surfaçar o histórico completo.

## Critérios de aceite

- `supersedes` e `contradicts` armazenados e recuperados via WAL/snapshot.
- Memórias supersedidas excluídas do recall padrão; surfaçadas via
  `--include-superseded`.
- Ids de contradição aparecem no `RecallResult` quando presentes.
- Novos testes de integração: exclusão por supersedura, surfacing de
  contradições, override com include-superseded, validação de ids desconhecidos.
- Todos os 76+ testes passam.
- `cargo fmt`, `cargo clippy -D warnings` limpos.
- Sem novas dependências externas.

## Fora de escopo

- Resolução automática de conflitos (responsabilidade do loop humano/agente,
  não do engine).
- Grafos de proveniência ou traversal de grafo de conhecimento (Fase 10 —
  Graph Memory).
- Server mode.

## Follow-up

Após o Temporal Truth Layer completo, a Fase 7 estará concluída. A Fase 8 é o
**Context Compiler**: `build_context(query, user, max_tokens)` — montagem de
contexto com budget de tokens para LLMs, a partir de todos os tipos de items
armazenados.
