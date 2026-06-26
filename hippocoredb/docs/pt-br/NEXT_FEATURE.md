# Próxima Feature

## Nome

**eval-quality CLI** — permitir que operadores executem avaliações de fixture de
qualidade contra qualquer banco implantado e obtenham um relatório (hit@k, MRR).

## Por que importa

O Hippocore DB possui um formato de fixture de qualidade de retrieval
(`tests/fixtures/retrieval_quality_v01.json`) usado internamente pela suíte de
testes. Mas desenvolvedores e operadores não têm como executar a mesma avaliação
contra um banco em produção via CLI — eles precisam escrever código ou depender
da suíte interna de testes, que opera em um banco temporário recém-alimentado.

Um comando `eval-quality` fecha essa lacuna: lê um arquivo de fixture, alimenta
as memórias do fixture em um banco temporário (ou existente), executa cada query
e reporta métricas por cenário e agregadas (hit@1, hit@k, MRR). Sai com código
não-zero se qualquer threshold não for atendido, sendo útil em pipelines de CI.

## Comportamento esperado

```
hippocore eval-quality --fixture path/to/fixture.json [--json] [--db <path>]
```

- `--fixture <file>`: caminho para um arquivo de fixture JSON (mesmo formato do
  fixture interno).
- `--db <path>` (opcional): se informado, alimenta memórias em um banco
  existente e avalia contra ele. Se omitido, usa diretório temporário.
- Saída por cenário: hit@1, hit@k, MRR e ausência de `forbidden_first_ids` no
  top.
- Saída agregada: média hit@1, hit@k, MRR em todos os cenários.
- `--json` emite um relatório JSON legível por máquina.
- Sai 0 se todos os thresholds passam; sai 1 com mensagem de erro legível.

## Arquivos afetados

- `crates/hippocore/src/cli.rs` — adicionar comando `EvalQuality` e handler.
- `crates/hippocore/src/lib.rs` — API de avaliação se necessária.
- `crates/hippocore-cli/tests/cli.rs` — smoke test via subprocesso.
- `docs/en/STATUS.md` e `docs/pt-br/STATUS.md`.
- `CHANGELOG.md`.

## Critérios de aceite

- `eval-quality --fixture <file>` executa e reporta hit@1, hit@k, MRR por
  cenário e aggregate.
- `--json` emite relatório JSON parseável.
- Sai com código não-zero quando threshold não é atendido.
- Sem novas dependências externas.
- Todos os 69+ testes passam.
- `cargo fmt`, `cargo clippy -D warnings` limpos.

## Fora de escopo

- Armazenamento persistente de fixtures no banco.
- Dashboard web ou relatório visual.
- Geração automática de fixture.
- Server mode.

## Follow-up

Após eval-quality, o próximo passo é Temporal Truth Layer v0.1:
`valid_from`/`valid_until` em memórias e documentos, permitindo queries "o que
era verdade no tempo T?" (Fase 7 do roadmap).
