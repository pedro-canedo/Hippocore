# Próxima Feature

## Nome

**Temporal Truth Layer v0.1** — adicionar campos `valid_from` / `valid_until`
em memórias e documentos, e suportar queries "o que era verdade no instante T?".

## Por que importa

Agentes de IA lidam frequentemente com fatos que mudam ao longo do tempo. Sem
metadados temporais, o Hippocore só consegue responder "o que está armazenado
agora?" — não "o que era conhecido em uma data específica?" Isso faz com que
fatos desatualizados contaminem silenciosamente os resultados de retrieval e
impossibilita auditar o que o agente acreditava em um dado momento.

Os campos `valid_from` / `valid_until` permitem ao chamador:
- Marcar fatos como vigentes (`valid_until = None`) ou expirados
  (`valid_until = Some(epoch_ms)`).
- Consultar memórias e documentos como estavam em um ponto no tempo via `as_of`.
- Substituir um fato antigo de forma limpa criando um novo com `valid_from`
  atualizado, sem deletar o antigo (auditabilidade preservada).

## Comportamento esperado

### Mudanças no modelo

`Memory` e `Document` ganham dois campos opcionais:
```
valid_from:  Option<i64>   // epoch ms; None = válido desde a criação
valid_until: Option<i64>   // epoch ms; None = ainda válido
```

### Mudanças na API

`RememberRequest` e `StoreDocumentRequest` ganham:
```
valid_from:  Option<i64>
valid_until: Option<i64>
```

`RecallRequest` / `SearchRequest` ganham:
```
as_of: Option<i64>  // epoch ms; None = agora
```

Quando `as_of` está definido, o retrieval filtra qualquer entrada onde:
- `valid_from > as_of` (ainda não válida), ou
- `valid_until <= as_of` (já expirada).

### Mudanças na CLI

- `remember` ganha `--valid-from <ms>` e `--valid-until <ms>`.
- `recall` ganha `--as-of <ms>`.
- `put-document` ganha `--valid-from <ms>` e `--valid-until <ms>`.

### Persistência

`valid_from` e `valid_until` são armazenados no WAL/snapshot como parte do
modelo JSON existente; entradas legadas sem esses campos são tratadas como
`valid_from = created_at`, `valid_until = None` (sempre válido).

## Critérios de aceite

- `valid_from` / `valid_until` armazenados e recuperados via WAL/snapshot.
- `recall --as-of <ms>` filtra entradas corretamente (passado válido, presente e
  futuro).
- Entradas expiradas não aparecem no recall padrão (`as_of = now` por padrão).
- Entradas legadas sem campos temporais se recuperam com semântica sempre-válido.
- Flags de CLI funcionam (`--valid-from`, `--valid-until`, `--as-of`).
- Novos testes unitários e de integração: sem filtro, as-of-passado,
  as-of-futuro, entradas expiradas, recuperação de entradas legadas.
- Todos os 71+ testes passam.
- `cargo fmt`, `cargo clippy -D warnings` limpos.

## Fora de escopo

- Relações `supersedes` / `contradicts` (especificação completa da Fase 7 —
  adiado).
- Resolução de conflitos entre fatos temporalmente sobrepostos.
- Otimização de índice para queries de intervalo temporal (brute force aceitável
  nesta escala).
- Server mode.

## Follow-up

Após o Temporal Truth Layer v0.1, o próximo passo pode ser:
- Completar o item restante da Fase 5: **benchmark regression guard** para
  evitar regressões silenciosas de latência no recall.
- Ou itens da Fase 6: Studio local para navegar e editar dados de contexto
  (adiado até que a Fase 6 seja agendada).
