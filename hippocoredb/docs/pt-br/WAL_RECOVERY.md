# WAL Recovery Tests v0.1

## Visão geral

Uma suite de testes dedicada verificando que `Storage::replay_wal` trata
gravações rasgadas, registros truncados e entradas com checksum incorreto
sem panic e sem carregar silenciosamente dados corrompidos. Nenhum código de
produção foi alterado; esta feature é puramente testes aditivos.

## Formato do WAL

Cada linha em `wal.log` é: `<crc32-hex>\t<json>\n`

Na abertura do banco, o replay para na primeira linha que seja:
- Truncada (incompleta / sem newline) — `BufReader::lines` retorna `Err`
- JSON não parseável (retorna `Ok(None)`)
- Checksum incorreto (retorna `Err(Corruption(...))`)

Todas as operações aplicadas antes da linha ruim são mantidas.

## Arquivo de testes

`crates/hippocore/tests/wal_recovery.rs`

## O que é testado

| Teste | Cenário |
|---|---|
| `clean_wal_opens_successfully` | Reabertura a frio normal — estado persiste |
| `truncated_wal_trailing_line_skipped_on_open` | Entrada com metade escrita (sem newline) anexada; abre com sucesso; dados anteriores intactos |
| `checksum_mismatch_stops_replay_does_not_panic` | Linha com CRC errado; abertura sem panic; entrada ruim não aplicada |
| `state_before_torn_entry_is_intact` | Memória escrita antes do rasgo sobrevive em `get_memory` e `recall` |

## Fora de escopo

- Mudanças na compactação do WAL.
- Modo server.
- Mudanças no código de produção.
