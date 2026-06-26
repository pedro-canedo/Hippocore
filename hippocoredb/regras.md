# Regras de Ouro — Hippocore DB

> Este documento é a **constituição** do projeto. Toda contribuição (humana ou
> de IA) deve respeitá-lo. Em caso de conflito entre conveniência e estas
> regras, **as regras vencem**. Mudar uma regra exige justificativa explícita
> registrada em `docs/DECISIONS.md`.

Documentos relacionados: [`CLAUDE.md`](CLAUDE.md),
[`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md),
[`docs/MVP_SCOPE.md`](docs/MVP_SCOPE.md),
[`docs/DECISIONS.md`](docs/DECISIONS.md),
[`docs/STATUS.md`](docs/STATUS.md), [`ROADMAP.md`](ROADMAP.md).

---

## 1. A visão acima de tudo

1. **Hippocore não é "só mais um vector store".** É um **banco de memória e
   contexto AI-native** para agentes, copilots, RAG e sistemas multi-tenant.
   Toda decisão deve aproximar o projeto desse banco de contexto completo
   (documentos, memórias, metadados, proveniência, fatos temporais, auditoria),
   não apenas de busca vetorial.
2. **Local-first e offline por padrão.** Nada de dependência obrigatória de
   rede, nuvem ou serviço externo no núcleo. O usuário deve conseguir rodar tudo
   na própria máquina.
3. **Multi-tenant e isolamento são inegociáveis.** Todo recall/search é escopado
   por `tenant_id`. Nenhum caminho de código pode retornar dado de outro tenant.
   Toda feature nova que toque retrieval precisa de teste de isolamento.
4. **Não puxar fases futuras prematuramente.** Entregar **incrementos pequenos e
   completos** (código + testes + docs), na ordem do `ROADMAP.md`. Recusar PRs
   gigantes que misturam várias fases.

## 2. Desempenho é requisito, não enfeite

5. **Leitura (recall/search) é servida da memória.** O caminho de consulta
   **nunca toca o disco**. Índices ficam em RAM e são reconstruídos no `open`.
6. **Escrita é append-only + índice incremental.** Cada escrita anexa ao WAL e
   atualiza o índice **incrementalmente** — nunca reconstruir o índice inteiro a
   cada escrita. Reconstrução total só no `open`.
7. **Sem O(n²) em hot path.** Operações de escrita e de consulta por item devem
   ser O(1) ou O(log n) amortizado. Custo linear (varredura por tenant) é
   aceitável só onde já documentado (busca brute-force) e tem teto conhecido.
8. **Complexidade documentada.** Toda estrutura/loop em hot path deve ter sua
   complexidade citada em comentário ou doc. Se algo é O(n), diga por quê e qual
   o plano de escala (ex.: ANN na Fase 3).
9. **Meça antes de otimizar — mas nunca regrida.** Use os benchmarks
   (`cargo bench`). Uma feature nova **não pode regredir** os benchmarks
   existentes sem justificativa registrada. Funcionalidades de retrieval novas
   devem vir acompanhadas de benchmark ou de uma nota de impacto.
10. **Evite trabalho e alocação desnecessários.** Sem `clone` gratuito em hot
    path; sem recomputar o que pode ser mantido incrementalmente (ex.: `avgdl`,
    contadores, tamanhos). Prefira referências e iteradores a coleções
    intermediárias.
11. **Custo de disco é limitado.** O WAL é compactado automaticamente
    (política em `Config`) e o snapshot é o estado vivo. Nenhuma feature pode
    fazer o armazenamento crescer de forma ilimitada sem compactação.

## 3. Correção e segurança de persistência

12. **Durabilidade primeiro: WAL antes da memória.** Toda mutação é
    **anexada ao WAL (com flush/fsync conforme `Config`) antes** de alterar o
    estado em memória. Um crash só pode perder uma escrita ainda não confirmada,
    nunca corromper o estado já confirmado.
13. **Escritas de snapshot são atômicas.** Sempre arquivo temporário → `fsync` →
    `rename`. Nunca sobrescrever um arquivo de estado in-place.
14. **Recuperação nunca entra em pânico e nunca carrega lixo.** Linha truncada,
    checksum inválido ou payload ilegível **param o replay com segurança**,
    mantendo o que já foi lido. Todo registro do WAL é verificado por checksum.
15. **Erros são tipados; sem panic em fluxo normal.** Toda API pública retorna
    `hippocore::Result<T>`. Nada de `unwrap`/`expect`/`panic!` em caminho de
    produção. Funções puras (cosine, embedder) retornam `Option`/pulam entrada
    inválida em vez de quebrar.
16. **Idempotência e segurança de no-op.** Operações como `delete`/`forget` em id
    inexistente são no-op seguras; reabrir o banco reproduz exatamente o mesmo
    estado. Toda feature de persistência precisa de teste de **restart**.

## 4. Arquitetura limpa

17. **Fronteiras de módulo são sagradas.** `storage` conhece operações e bytes
    (não scoring); `index`/`query` não tocam disco; `memory` (embedder/chunker) e
    `index::cosine` são puros e não entram em pânico; o motor `Hippocore`
    (`lib.rs`) é o único orquestrador. Ver `docs/ARCHITECTURE.md`.
18. **`unsafe` é proibido.** `#![forbid(unsafe_code)]` nos dois crates. Se algo
    parece exigir `unsafe`, repense o design.
19. **Dependências enxutas.** Adicionar uma dependência exige justificativa em
    `docs/DECISIONS.md`. Preferir implementação pequena e auditável (ex.: CRC32 e
    embedder próprios) a arrastar crates pesados.
20. **API ergonômica e estável.** Tipos públicos claros, com `///` docs.
    Mudanças que quebram a API pública precisam de nota no `CHANGELOG.md`.

## 5. Qualidade — portão obrigatório

21. **Nenhuma tarefa termina com o portão vermelho.** Antes de concluir qualquer
    trabalho, devem passar:
    ```
    cargo fmt --all --check
    cargo test --workspace
    cargo clippy --workspace --all-targets -- -D warnings
    ```
22. **Corrigir o código, não silenciar o lint.** Nada de `#[allow(...)]` para
    fugir do clippy sem motivo forte e documentado.
23. **Testes determinísticos e sem rede.** Use o embedder determinístico e
    `tempfile`. Nenhum teste pode depender de serviço externo, relógio real ou
    ordem de execução.
24. **Toda feature vem com testes.** No mínimo: caminho feliz, validação/erro
    tipado, e (se toca persistência) restart. Bugs corrigidos viram teste de
    regressão.

## 6. Documentação viva

25. **Docs fazem parte do produto.** Nenhuma feature está "pronta" até
    `docs/STATUS.md` e `docs/NEXT_FEATURE.md` estarem atualizados, e o
    `CHANGELOG.md` quando houver mudança visível/arquitetural.
26. **`docs/NEXT_FEATURE.md` sempre aponta o próximo passo de maior valor**, com
    comportamento esperado, módulos afetados, critérios de aceite e testes.
27. **Decisões relevantes viram ADR** em `docs/DECISIONS.md` (contexto,
    alternativas, motivo, trade-offs).

## 7. Open source e privacidade

28. **Zero dados de clientes ou segredos no repositório.** Sem nomes de clientes,
    credenciais, hosts, dumps ou qualquer informação identificável. Exemplos
    devem ser **genéricos e fictícios**.
29. **Exemplos realistas, porém neutros.** Termos técnicos públicos (mensagens de
    erro, comandos) são permitidos; vínculos a uma organização real, não.
30. **Licença e atribuição respeitadas.** Mantenha o `LICENSE` (MIT) e não
    introduza código incompatível.

---

### Checklist rápido antes de abrir/concluir um PR

- [ ] Aproxima o produto da visão (memória/contexto AI-native), sem puxar fase futura.
- [ ] Recall serve da memória; índice atualizado incremental; sem O(n²) novo.
- [ ] WAL antes da memória; snapshot atômico; recuperação segura testada.
- [ ] Erros tipados; sem `unsafe`; sem panic em fluxo normal.
- [ ] `fmt` + `test` + `clippy -D warnings` verdes.
- [ ] Testes (feliz, erro, restart) adicionados; benchmarks não regrediram.
- [ ] `STATUS.md` + `NEXT_FEATURE.md` (+ `CHANGELOG.md`) atualizados.
- [ ] Sem dados de cliente/segredos; exemplos genéricos.
