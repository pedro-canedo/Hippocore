# Próxima Feature

## Nome

**Graph-Aware Context v0.1** — expansão opcional por vizinhos diretos na montagem
de contexto.

## Por que importa

Graph Memory v0.1 armazena relacionamentos diretos duráveis e expõe ids de
vizinhos na proveniência de contexto/auditoria, mas ainda não usa essas relações
para melhorar o contexto montado. O próximo incremento de maior valor é permitir
que chamadores optem por incluir vizinhos diretos do grafo em `build_context`,
sem implementar travessia GraphRAG completa nem alterar o ranking padrão.

Graph-Aware Context v0.1 deve adicionar:

- Opção `include_related: bool` em `BuildContextRequest` (default `false`).
- Opção pequena `related_limit: usize` para limitar expansão por vizinhos
  diretos.
- Montagem de contexto capaz de incluir vizinhos diretos dos itens recuperados
  quando couberem no budget de tokens.
- Proveniência marcando se um item entrou por recall ou por expansão de grafo.
- Registros de auditoria que diferenciem itens expandidos por grafo dos
  candidatos recuperados.
- Flags no CLI `build-context`: `--include-related` e `--related-limit <n>`.

## Critérios de aceite

- Comportamento padrão de `build_context` não muda quando
  `include_related = false`.
- Quando habilitado, apenas vizinhos diretos de itens recuperados são
  considerados.
- Expansão continua isolada por tenant e limitada pelo budget de tokens.
- Itens expandidos não são duplicados se já vieram pelo recall.
- Saída de contexto/auditoria distingue claramente recall vs expansão por
  grafo.
- Mínimo de 4 testes determinísticos: default inalterado, inclusão de vizinho
  direto, respeito a budget/limit, isolamento por tenant/sem duplicação.
- Documentação atualizada em inglês e português.
- Quality gate:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Fora de escopo

- Travessia multi-hop.
- Ranking ou boost ciente de grafo.
- Extração de entidades.
- Linguagem de query de grafo.
- Server/HTTP mode.

## Follow-up

Após Graph-Aware Context v0.1, avaliar se ranking de recall ciente de grafo vale
a pena ou se retenção/rotação de auditoria é mais valiosa.
