# Hippocore DB — Ideia Original e Regras de Ouro do Projeto

## 1. Visão original do produto

O **Hippocore DB** é um banco de dados **AI-native**, **open source**, **local-first** e escrito em **Rust**, criado para ser uma infraestrutura de **memória, contexto e conhecimento confiável para agentes de Inteligência Artificial, RAG corporativo e sistemas multimodais**.

O Hippocore DB **não deve ser apenas mais um banco vetorial**.

A visão principal é criar um:

> **Context Database for AI Agents**
> Um banco de memória e contexto para agentes inteligentes.

O objetivo não é somente armazenar embeddings e retornar vetores parecidos. O objetivo é permitir que sistemas de IA consigam:

* lembrar contexto entre sessões;
* recuperar conhecimento confiável;
* diferenciar informação atual de informação antiga;
* lidar com documentos contraditórios;
* montar contexto otimizado para LLMs;
* auditar por que uma resposta foi gerada;
* trabalhar localmente com soberania de dados;
* armazenar documentos, vetores, metadados, fatos temporais, relações, fontes e confiança.

---

## 2. Problemas que o Hippocore DB deve resolver

O projeto nasceu para atacar problemas reais de IA em produção.

### 2.1 IA esquece contexto

Agentes e chatbots normalmente perdem contexto entre sessões. O Hippocore DB deve permitir memória persistente, separada por usuário, organização, projeto, agente e sessão.

### 2.2 RAG comum busca o trecho errado

RAG tradicional costuma fazer:

```txt
PDF -> chunks -> embeddings -> vector search -> resposta
```

Isso é limitado. O Hippocore DB deve evoluir para busca híbrida e contextual:

```txt
busca vetorial + busca textual + metadados + tempo + fonte + confiança + permissões + relações
```

### 2.3 IA usa informação antiga como se fosse atual

Documentos mudam. Contratos mudam. Regras mudam. Ambientes mudam.

O Hippocore DB deve ter uma futura camada de verdade temporal, permitindo saber:

* quando uma informação passou a valer;
* quando deixou de valer;
* qual informação substituiu outra;
* quais documentos contradizem outros;
* qual fonte é mais confiável.

### 2.4 Empresas não conseguem auditar respostas de IA

Toda resposta gerada por IA deveria poder ser rastreada.

O Hippocore DB deve evoluir para registrar:

* pergunta original;
* query reescrita;
* documentos recuperados;
* contexto enviado ao modelo;
* resposta gerada;
* fontes usadas;
* latência;
* custo em tokens;
* feedback do usuário;
* score de confiança.

### 2.5 Contexto de LLM é caro e limitado

Não basta retornar chunks. O banco deve evoluir para um **Context Compiler**, capaz de montar o melhor contexto possível dentro de um limite de tokens.

Exemplo futuro:

```rust
db.build_context(query, user, max_tokens)
```

O retorno deve conter:

* fatos relevantes;
* documentos fonte;
* memórias importantes;
* contradições;
* citações;
* nível de confiança;
* resumo otimizado para o LLM.

### 2.6 IA local precisa de soberania de dados

O projeto deve valorizar execução local, offline e self-hosted.

A proposta é permitir que empresas, órgãos públicos, times técnicos e desenvolvedores possam usar IA com dados sensíveis sem depender obrigatoriamente de SaaS externo.

---

## 3. Posicionamento do produto

Frase principal:

```txt
Hippocore DB is not just a vector database.
It is a memory and context engine for AI agents.
```

Em português:

```txt
Hippocore DB não é apenas um banco vetorial.
É um motor de memória e contexto para agentes de IA.
```

Descrição curta:

```txt
Hippocore DB é um banco AI-native de memória e contexto, escrito em Rust, focado em RAG, agentes, busca vetorial, memória temporal e auditoria de contexto.
```

Slogan possível:

```txt
The memory core for intelligent systems.
```

Ou:

```txt
Memory infrastructure for AI agents.
```

---

## 4. Escopo inicial do MVP

O MVP deve ser pequeno, funcional, bem testado e extensível.

A primeira versão deve implementar apenas a fundação:

* banco embedded/local-first;
* document store;
* persistência em disco;
* append-only log;
* índice em memória;
* `put`;
* `get`;
* `delete`;
* recovery ao abrir o banco;
* embeddings como `Vec<f32>`;
* busca vetorial brute force com similaridade cosseno;
* filtros simples por metadados;
* CLI básica;
* testes;
* documentação;
* roadmap.

O MVP não deve tentar resolver tudo de uma vez.

---

## 5. O que o MVP não deve implementar ainda

Para manter coerência e evitar overengineering, o MVP não deve implementar:

* HNSW;
* BM25;
* busca híbrida avançada;
* GraphRAG;
* servidor HTTP/gRPC;
* cluster distribuído;
* autenticação;
* multitenancy completo;
* permissões complexas;
* SQL/query language;
* processamento multimodal real;
* Context Compiler completo;
* RAG Audit Engine completo;
* Temporal Truth Layer completo.

Esses itens pertencem ao roadmap, não ao MVP inicial.

---

## 6. Modelo mental da arquitetura futura

O Hippocore DB deve evoluir para ter os seguintes motores internos:

```txt
1. Document Store
2. Vector Engine
3. Metadata Index
4. Temporal Memory Layer
5. Graph Memory
6. Context Compiler
7. Semantic Cache
8. RAG Audit Engine
9. Permission Layer
10. Multimodal Storage
```

Mas a fundação deve começar simples:

```txt
Document Store
    ↓
Append-only Log
    ↓
In-memory Index
    ↓
Brute-force Vector Search
    ↓
Metadata Filter
```

---

## 7. Regras de Ouro do Hippocore DB

### Regra 1 — O projeto não é apenas um vector database

Toda decisão deve respeitar a visão principal:

```txt
Hippocore DB é um banco de memória e contexto para IA.
```

Busca vetorial é apenas uma parte do produto, não o produto inteiro.

---

### Regra 2 — O banco deve entregar contexto confiável, não apenas dados parecidos

O objetivo não é só encontrar documentos similares. O objetivo é ajudar uma IA a responder melhor, com mais confiança, mais rastreabilidade e menos alucinação.

---

### Regra 3 — Local-first é um princípio central

O Hippocore DB deve funcionar localmente, como banco embedded, antes de depender de servidor, cloud ou SaaS.

O projeto deve favorecer:

* privacidade;
* soberania de dados;
* execução offline;
* uso em ambiente corporativo;
* self-hosting;
* baixo acoplamento com fornecedores externos.

---

### Regra 4 — Rust é a linguagem principal do core

O core do banco deve ser escrito em Rust para garantir:

* performance;
* segurança de memória;
* previsibilidade;
* controle fino de alocações;
* boa base para storage engine;
* boa base para busca vetorial futura;
* binário nativo e eficiente.

SDKs em Go, Python e TypeScript podem existir no futuro, mas o coração do banco deve ser Rust.

---

### Regra 5 — O MVP deve ser simples, mas não descartável

O MVP pode ser simples, mas precisa nascer com arquitetura limpa.

Evitar hacks que impeçam evolução futura.

O MVP deve ser uma fundação real para:

* HNSW;
* BM25;
* compaction;
* WAL robusto;
* Context Compiler;
* auditoria;
* memória temporal;
* Graph Memory.

---

### Regra 6 — Não implementar feature futura antes da fundação estar sólida

Antes de adicionar funcionalidades avançadas, garantir que o básico está correto:

* persistência;
* recovery;
* testes;
* API limpa;
* CLI funcional;
* tratamento de erro;
* documentação;
* benchmarks básicos.

Sem isso, qualquer feature avançada será frágil.

---

### Regra 7 — Performance deve ser medida, não presumida

Nunca afirmar que algo é performático sem benchmark.

Toda otimização relevante deve nascer de:

* benchmark;
* profiling;
* comparação;
* medição de latência;
* medição de uso de memória;
* análise de throughput.

Primeiro: correto.
Depois: medido.
Depois: otimizado.

---

### Regra 8 — Toda informação importante deve carregar origem e confiança

O Hippocore DB deve favorecer dados com rastreabilidade.

Sempre que possível, documentos e fatos devem poder ter:

* source;
* confidence;
* created_at;
* updated_at;
* version;
* collection;
* metadata;
* origem do dado;
* validade temporal futura.

---

### Regra 9 — Dados antigos não são necessariamente falsos, mas podem estar superados

O banco deve evoluir para diferenciar:

* informação atual;
* informação antiga;
* informação supersedida;
* informação contraditória;
* informação incerta.

Nunca tratar memória como simples chave-valor sem tempo e contexto no roadmap de longo prazo.

---

### Regra 10 — Auditoria deve ser uma feature nativa, não um detalhe externo

O banco deve evoluir para permitir rastrear como a IA chegou em uma resposta.

No futuro, cada resposta de RAG/agente deve poder registrar:

* pergunta;
* busca realizada;
* documentos candidatos;
* documentos usados;
* contexto final;
* resposta;
* modelo usado;
* tokens;
* latência;
* feedback;
* score de confiança.

---

### Regra 11 — O Context Compiler é uma das features centrais do produto

O futuro diferencial do Hippocore DB será a capacidade de montar contexto otimizado para LLMs.

Não basta retornar documentos. O banco deve evoluir para entregar um pacote de contexto pronto para IA.

Exemplo futuro:

```txt
build_context(query, user, max_tokens)
```

Esse recurso deve ser tratado como uma feature estratégica.

---

### Regra 12 — Metadata e filtros são tão importantes quanto vetores

Em RAG corporativo, não basta similaridade semântica.

O banco precisa considerar:

* cliente;
* ambiente;
* projeto;
* versão;
* permissão;
* origem;
* data;
* tipo de documento;
* coleção;
* confiança.

Busca vetorial sem filtro pode trazer contexto errado.

---

### Regra 13 — Multitenancy e permissões devem ser considerados desde cedo no design

Mesmo que não sejam implementados no MVP, o design deve evitar escolhas que dificultem:

* múltiplos clientes;
* múltiplas organizações;
* múltiplos projetos;
* isolamento de dados;
* permissões por usuário;
* permissões por agente;
* permissões por coleção.

O banco deve ser pensado para uso corporativo.

---

### Regra 14 — O projeto deve ser modular

Cada área deve ter responsabilidade clara:

```txt
storage
document
index
vector
query
memory
audit
context
cli
server futuro
```

Evitar arquivos gigantes, acoplamento excessivo e lógica misturada.

---

### Regra 15 — CLI e API pública devem ser simples

O projeto deve ser fácil de testar e demonstrar.

A CLI deve permitir:

```txt
hippocore put
hippocore get
hippocore delete
hippocore search
hippocore stats
```

A API Rust deve ser clara e previsível.

---

### Regra 16 — Testes são parte do produto

Nenhuma funcionalidade crítica deve existir sem teste.

Testes obrigatórios para o MVP:

* put/get;
* overwrite;
* version increment;
* delete;
* recovery;
* search;
* metadata filter;
* not found;
* validation error;
* dimension mismatch;
* empty database;
* CLI smoke test quando possível.

---

### Regra 17 — Recovery é obrigatório

Banco que não recupera dados após reiniciar não é banco.

O Hippocore DB deve sempre garantir que documentos persistidos possam ser recuperados após fechar e abrir novamente.

---

### Regra 18 — Erros devem ser claros e tipados

Evitar erros genéricos sem contexto.

O banco deve ter erros claros para:

* documento não encontrado;
* coleção inválida;
* ID inválido;
* embedding inválido;
* confiança inválida;
* erro de storage;
* erro de encoding;
* erro de recovery.

---

### Regra 19 — Segurança e privacidade devem fazer parte do roadmap

Como o banco lida com memória de IA, no futuro deve considerar:

* LGPD;
* direito ao esquecimento;
* anonimização;
* remoção de dados sensíveis;
* criptografia;
* isolamento por tenant;
* controle de acesso;
* logs auditáveis.

---

### Regra 20 — Documentação é parte da arquitetura

Toda decisão importante deve estar documentada.

Arquivos essenciais:

* README.md;
* ROADMAP.md;
* CLAUDE.md;
* CONTRIBUTING.md;
* LICENSE;
* docs/architecture.md futuramente;
* docs/adr/ futuramente.

O projeto deve ser fácil de entender por novos contribuidores.

---

## 8. Roadmap estratégico

### Fase 1 — Fundação embedded

* Document store;
* append-only log;
* in-memory index;
* put/get/delete;
* recovery;
* brute-force vector search;
* metadata filter;
* CLI;
* testes;
* documentação.

### Fase 2 — Storage robusto

* WAL mais forte;
* checksums;
* segment files;
* compaction;
* batch writes;
* fsync configurável;
* recovery mais resiliente.

### Fase 3 — Busca vetorial performática

* HNSW;
* índice vetorial pluggable;
* cache;
* benchmarks;
* otimização de memória.

### Fase 4 — Busca híbrida

* BM25;
* full-text search;
* reranking;
* combinação de score vetorial + textual + metadata.

### Fase 5 — Temporal Truth Layer

* valid_from;
* valid_until;
* supersedes;
* contradicts;
* confidence;
* source;
* versão de fatos.

### Fase 6 — Context Compiler

* build_context;
* limite de tokens;
* seleção de fatos;
* seleção de documentos;
* compressão de contexto;
* citações;
* contradições.

### Fase 7 — RAG Audit Engine

* trace de perguntas;
* documentos recuperados;
* contexto enviado;
* resposta gerada;
* tokens;
* latência;
* modelo usado;
* feedback.

### Fase 8 — Graph Memory

* entidades;
* relações;
* GraphRAG;
* busca por caminhos;
* memória conectada.

### Fase 9 — Server Mode

* HTTP API;
* gRPC;
* autenticação;
* multitenancy;
* permissões;
* observabilidade.

### Fase 10 — Multimodal

* PDF;
* imagem;
* áudio;
* vídeo;
* código;
* embeddings multimodais;
* extração de texto;
* OCR;
* metadados visuais.

---

## 9. Critério permanente de coerência do projeto

Antes de implementar qualquer nova feature, responder:

```txt
1. Essa feature fortalece o Hippocore DB como banco de memória e contexto para IA?
2. Ela ajuda agentes ou RAG a recuperarem contexto mais confiável?
3. Ela respeita local-first e open source?
4. Ela mantém a arquitetura modular?
5. Ela tem teste?
6. Ela tem documentação?
7. Ela não antecipa complexidade desnecessária?
8. Ela melhora storage, busca, memória, auditoria, contexto ou confiança?
```

Se a resposta for “não” para a maioria dessas perguntas, a feature provavelmente está fora do escopo.

---

## 10. Definição final do projeto

O Hippocore DB deve ser construído com esta identidade:

```txt
Hippocore DB é um banco de dados AI-native, local-first e open source,
escrito em Rust, criado para servir como núcleo de memória e contexto
para agentes de IA, RAG corporativo e sistemas inteligentes.
```

Ele deve priorizar:

```txt
memória
contexto
confiança
rastreabilidade
recuperação semântica
verdade temporal
auditoria
privacidade
performance medida
arquitetura limpa
```

E deve evitar virar apenas:

```txt
mais um CRUD
mais um vector database
mais uma API de embeddings
mais um cache simples
mais um projeto sem testes
mais um banco sem recovery
```

A regra máxima do projeto:

> **O Hippocore DB existe para entregar contexto confiável para sistemas inteligentes.**
