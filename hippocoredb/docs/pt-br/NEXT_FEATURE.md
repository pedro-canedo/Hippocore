# Próxima Feature

## Nome

**Fundação de UX estilo banco: camada de comando SQL**

## Slice implementado

Hippocore agora tem uma fundação pequena estilo SQL para consultas read-only de
records estruturados:

- API pública `Hippocore::execute_sql(tenant_id, sql)`;
- tipos `SqlCommand` e `SqlResult`;
- `SELECT * FROM <tabela> WHERE <campo> = <valor> [AND ...] [LIMIT n]`;
- campos sem prefixo mapeados para chaves do payload do record;
- suporte explícito a `id`, `collection`, `table`, `payload.<key>` e
  `metadata.<key>`;
- execução escopada por tenant;
- comando CLI: `hippocore query --db ... --tenant ... --sql ... --json`;
- endpoint admin: `POST /admin/sql`;
- comportamento legado de `query-records` preservado.

## Próximo slice recomendado

Adicionar um catálogo leve de tabelas/schemas sobre o namespace existente
`Record.table`. Esse slice deve persistir metadados de tabela, expor fluxos de
listar/criar tabela e manter records JSON-first até que escritas via SQL sejam
justificadas.

## Fora do escopo do MVP SQL

- Parsing ou execução SQL completa.
- Joins, projeções, ordenação, agregações e predicados JSON aninhados.
- Comandos SQL `CREATE TABLE`, `INSERT`, `AI SEARCH` e `IMPORT`.
- Substituir a API CRUD de records existente.
