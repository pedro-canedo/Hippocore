# Próxima Feature

## Nome

**Temporal Decay v0.1** — aplicar um viés de recência aos scores de recall.

## Por que importa

O Graph-Aware Ranking v0.1 recompensa conectividade; a lacuna remanescente é
desatualização. Uma memória sobre "a config de staging" armazenada há seis
meses deveria ter score menor do que uma memória equivalente adicionada ontem,
porque contexto recente é mais provavelmente preciso e acionável. Sem
ponderação de recência, similaridade semântica sozinha determina o ranking —
uma memória desatualizada mas altamente similar pode deslocar uma mais recente,
ligeiramente menos similar.

## Comportamento

Após o recall híbrido produzir candidatos com scores, aplica um multiplicador
de decaimento temporal antes do re-ranking de grafo e do passo de budget:

- `age_seconds = now_ms - item.updated_at_ms / 1000`.
- `decay = exp(-decay_rate × age_seconds / 86400)` (exponencial, por dia).
- `effective_score = recall_score * (1 - temporal_weight) + decay * temporal_weight`.

Quando `temporal_weight = 0.0` (padrão) o multiplicador é no-op e a ordenação
não muda. Quando `temporal_weight = 1.0` apenas a recência importa.

## Arquivos

- `crates/hippocore/src/lib.rs` — blend de decaimento temporal em
  `build_context` após `run_query` e antes do ranking de grafo.
- `crates/hippocore/src/config.rs` — novos campos `temporal_weight: f32`
  (padrão `0.0`) e `temporal_decay_days: f32` (padrão `30.0`).
- `crates/hippocore/tests/database.rs` — testes determinísticos.
- `docs/en/TEMPORAL_DECAY.md` e `docs/pt-br/TEMPORAL_DECAY.md`.
- `docs/en/STATUS.md` e `docs/pt-br/STATUS.md`.

## Critérios de aceite

- `temporal_weight = 0.0` mantém exatamente a mesma ordenação que o recall
  simples.
- Uma memória atualizada recentemente é ranqueada acima de uma memória
  idêntica (conteúdo, vetor) mais antiga quando `temporal_weight > 0`.
- `temporal_weight` ou `temporal_decay_days` fora do range válido retorna erro
  tipado.
- Mínimo de 3 testes de integração determinísticos.
- Quality gate:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Fora de escopo

- Reindexação periódica em background por idade.
- Modo server.
- Evicção automática de memórias desatualizadas.
- Travessia multi-hop de grafo.
