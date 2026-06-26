# Próxima Feature

## Nome

**Recall Min-Score Filter v0.1**

## Por que importa

Após o confidence-weighted recall, a faixa de scores pode variar muito entre
candidatos. Um contexto LLM montado com `top_k=10` pode incluir varios itens
com scores próximos de zero — ruido que desperdiça tokens e pode confundir o
modelo. Permitir que o chamador defina um limiar mínimo de score troca cobertura
por precisão sem alterar `top_k`.

## Comportamento

- Adicionar `min_score: Option<f32>` a `RecallRequest` (padrão `None`).
- Após confidence weighting e ordenação final, filtrar resultados com
  `score < min_score`. A truncagem a `top_k` ocorre após a filtragem.
- Itens exatamente em `min_score` são incluídos (limite inferior inclusivo).
- Adicionar flag `--min-score <f32>` ao comando `hippocore recall`.
- Endpoints HTTP de recall aceitam `{"min_score": 0.5}` no body.

## Arquivos prováveis

- `crates/hippocore/src/lib.rs`
- `crates/hippocore/src/query.rs`
- `crates/hippocore/src/cli.rs`
- `crates/hippocore-server/src/handlers/recall.rs`
- `crates/hippocore/tests/retrieval_quality.rs`
- `docs/en/STATUS.md`
- `docs/pt-br/STATUS.md`
- `CHANGELOG.md`

## Critérios de aceite

- `RecallRequest` tem `min_score: Option<f32>` com padrão `None`.
- Com `min_score = Some(0.5)`, nenhum resultado com `score < 0.5` é retornado.
- Com `min_score = None`, o comportamento existente é preservado.
- `hippocore recall --min-score 0.3 ...` passa o limiar para o core.
- Body HTTP aceita `{"min_score": 0.5}` sem quebrar chamadas existentes.
- Testes cobrem: limiar remove itens de baixo score, `None` retorna todos,
  itens exatamente no limiar são incluídos.
- Portão verde passa.

## Fora do escopo

- Limiares por modo (vector vs text).
- Calibração dinâmica de threshold.
- Expor `min_score` em `BuildContextRequest`.
- Qualquer alteração em storage ou index.
