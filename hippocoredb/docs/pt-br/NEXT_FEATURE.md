# Próxima Feature

## Nome

**Supersession Lifecycle Tests v0.1** — testes determinísticos cobrindo o
ciclo de vida completo da substituição de memórias.

## Por que importa

A vinculação `supersedes` / `superseded_by` permite que uma nova memória
substitua uma mais antiga. Este é o mecanismo primário para atualizar fatos
desatualizados sem deletar o histórico. Embora o código de produção lide com
isso, não há uma suite de regressão focada que verifique:

- Memórias substituídas são excluídas do recall padrão.
- Memórias substituídas são visíveis com `include_superseded = true`.
- A vinculação é bidirecional e simétrica.
- O relacionamento sobrevive à compactação do WAL.

## Comportamento

Sem novo código de produção. A suite de testes cobrirá:

1. Armazenar memória A; armazenar memória B que substitui A.
2. Afirmar: recall sem `include_superseded` retorna B mas não A.
3. Afirmar: recall com `include_superseded = true` retorna A e B.
4. Afirmar: `superseded_by` de A é o id de B; `supersedes` de B contém o id de A.
5. Compactar o WAL; reabrir o banco; repetir afirmações 2–4.
6. Deletar B; afirmar: A não está mais substituída (ou que o delete é rejeitado
   se integridade referencial for aplicada).

## Arquivos

- `crates/hippocore/tests/supersession.rs` — novo arquivo de testes dedicado.
- `docs/en/SUPERSESSION_LIFECYCLE.md` e `docs/pt-br/SUPERSESSION_LIFECYCLE.md`.
- `docs/en/STATUS.md` e `docs/pt-br/STATUS.md`.

## Critérios de aceite

- Mínimo de 5 testes de integração determinísticos.
- Quality gate:
  - `cargo fmt --all --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Fora de escopo

- Cadeias de substituição (A → B → C).
- Modo server.
- Mudanças no código de produção do modelo de substituição.
