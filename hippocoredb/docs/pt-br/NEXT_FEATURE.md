# Próxima Feature

## Nome

**Benchmark regression guard** — tornar regressões de latência no recall
visíveis e acionáveis, completando a Fase 5 do roadmap.

## Por que importa

O Hippocore DB já tem um gate de *qualidade* de retrieval (hit@1/hit@k/MRR).
Ainda não tem um gate de *performance* de retrieval. À medida que o codebase
evolui — novos modos de retrieval, filtro temporal, lógica de scoring mais
rica — é fácil introduzir regressões O(n) que só aparecem em produção. Um
baseline de benchmark comprometido com comparação por execução dá feedback
imediato aos contribuidores quando uma mudança piora a latência de recall além
de um threshold configurável.

## Comportamento esperado

Um novo target de `cargo bench` mede a latência de recall com um tamanho padrão
de fixture (ex: 500 memórias). O benchmark salva resultados em um arquivo de
baseline (`benches/baseline.json`). Um comando auxiliar lê o baseline, executa
o benchmark novamente e falha se qualquer medição exceder o baseline por mais
que um threshold (ex: 20%).

O baseline é commitado no repositório, para que CI detecte regressões.
O baseline pode ser atualizado explicitamente (`cargo xtask update-baseline`).

### Decisões de design

- Baseline armazenado como JSON: `{benchmark_name, mean_ns, std_ns, timestamp}`.
- Threshold configurável via variável de ambiente `HIPPO_BENCH_THRESHOLD`
  (padrão 0.20 = 20% mais lento é falha).
- O check é um binário/script separado para não complicar o `cargo test`.
- Usa o setup existente de `criterion`; nenhum novo framework de benchmark.

## Critérios de aceite

- `cargo bench -p hippocore` grava dados de timing atualizados.
- Arquivo de baseline commitado em `benches/baseline.json`.
- Script ou xtask de verificação de regressão lê o baseline e sai com código
  não-zero se qualquer benchmark de recall regredir além do threshold.
- Latência de recall medida com fixture realista (≥ 200 memórias).
- Todos os 76+ testes passam.
- `cargo fmt`, `cargo clippy -D warnings` limpos.

## Fora de escopo

- Medição de P95/P99 (média e stddev suficientes para um guard).
- Mudanças na infraestrutura de CI.
- Profiling ou flame-graph.
- Server mode.

## Follow-up

Após o benchmark guard, a Fase 5 estará completa. O próximo passo é a
**Fase 7 — Temporal Truth Layer spec completa**: relações `supersedes` /
`contradicts`, resolução de conflitos e semântica temporal mais rica.
