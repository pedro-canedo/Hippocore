# Consultas de Records estilo SQL

Hippocore tem uma camada pequena e tipada de comandos estilo SQL para records
estruturados. Ela intencionalmente não é um motor SQL completo.

## Sintaxe suportada

```sql
SELECT * FROM <tabela>
WHERE <campo> = <valor> [AND ...]
LIMIT <n>;
```

Exemplos:

```bash
hippocore query --db ./data --tenant acme \
  --sql "select * from sistemas where engine = 'postgresql' limit 20" \
  --json
```

```sql
SELECT * FROM sistemas
WHERE engine = 'postgresql'
  AND metadata.tier = 'database'
LIMIT 5;
```

## Semântica

- Consultas sempre são escopadas ao tenant informado.
- `FROM <tabela>` mapeia para `Record.table`.
- Campos sem prefixo, como `engine`, mapeiam para `payload.engine`.
- Campos explícitos suportados no `WHERE`:
  - `id`
  - `collection`
  - `table`
  - `payload.<key>`
  - `metadata.<key>`
- Condições usam apenas igualdade exata.
- Valores podem ser strings entre aspas ou identificadores simples.
- O limite padrão é `100`; o máximo é truncado para `500`.

## Compatibilidade

A forma restrita antiga continua funcionando:

```sql
select * from records where table = 'sistemas' and payload.engine = 'postgresql'
```

Use `hippocore query-records` ou `/admin/query-records` para o formato legado de
resposta. Use `hippocore query` ou `/admin/sql` para a nova resposta
`SqlResult`.

## Fora do escopo

Joins, ordenação, projeções, agregações, predicados JSON aninhados, escritas e
imports via SQL ainda não foram implementados.
