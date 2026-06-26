# Próxima Feature

## Nome

**File ingestion v0.1** — armazenar arquivos textuais como objetos de banco e
projetá-los em contexto nativo.

## Por que importa

Hippocore DB está evoluindo de memória/document retrieval para um banco onde
informações armazenadas viram contexto. Records estruturados já existem. O
próximo tipo de dado prático é arquivo, começando por formatos textuais locais e
sem dependências externas.

## Comportamento esperado

- Adicionar um modelo `FileObject` ou equivalente:
  - tenant;
  - collection;
  - id;
  - path/nome original;
  - media type ou extensão;
  - checksum;
  - metadata;
  - source;
  - created/updated/version.
- Adicionar comando:

```bash
hippocore import-file --db ./data --tenant acme --collection kb --path ./notes.md
```

- Suportar primeiro:
  - `.txt`;
  - `.md`;
  - `.json`;
  - `.csv`.
- Armazenar metadata do arquivo de forma durável.
- Projetar texto extraído para documentos/chunks ou contexto record-like.
- Não criar blob engine complexa ainda.

## Arquivos afetados

- `model.rs`;
- `storage.rs`;
- `lib.rs`;
- `cli.rs`;
- helper de ingestão textual;
- `DATA_MODEL.md`, `README.md`, `STATUS.md`, `CHANGELOG.md`;
- testes de import, recall, metadata, restart, delete e formato não suportado.

## Critérios de aceite

- `.txt` ou `.md` pode ser importado e recuperado.
- `.json` e `.csv` têm projeção determinística.
- Metadata do arquivo sobrevive reopen.
- Extensões não suportadas retornam erro claro.
- Tenant isolation e metadata filters continuam válidos.
- Sem rede ou parser externo obrigatório.
- `fmt`, `test` e clippy passam.

## Fora de escopo

- PDF;
- OCR;
- imagem/áudio/vídeo;
- blob engine completa;
- server mode;
- admin UI.
