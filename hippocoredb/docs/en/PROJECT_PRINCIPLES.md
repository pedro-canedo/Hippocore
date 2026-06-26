# Hippocore DB — Original Idea and Golden Rules

## 1. Product Vision

Hippocore DB is an **AI-native**, **open-source**, **local-first** database
written in **Rust**. It exists to be infrastructure for reliable **memory,
context and knowledge** for AI agents, enterprise RAG and future multimodal
systems.

Hippocore DB must not become just another vector database.

The product identity is:

```txt
Context Database for AI Agents
```

The goal is not only to store embeddings and return similar vectors. The goal is
to help AI systems:

- remember context across sessions;
- retrieve trustworthy knowledge;
- distinguish current information from old information;
- handle contradictory documents;
- assemble optimized context for LLMs;
- audit why an answer was generated;
- work locally with data sovereignty;
- store documents, vectors, metadata, temporal facts, relations, sources and
  confidence.

## 2. Problems Hippocore Must Solve

### AI forgets context

Agents and chatbots lose context between sessions. Hippocore DB must support
persistent memory scoped by user, organization, project, agent and session.

### Traditional RAG retrieves the wrong context

Traditional RAG is often:

```txt
PDF -> chunks -> embeddings -> vector search -> answer
```

Hippocore must evolve toward contextual retrieval:

```txt
vector search + text search + metadata + time + source + confidence + permissions + relations
```

### AI treats old information as current

Documents, contracts, rules and environments change. Hippocore must eventually
support a temporal truth layer: validity ranges, supersession, contradictions
and source confidence.

### Enterprises cannot audit AI answers

Future Hippocore should record question, rewritten query, retrieved documents,
context sent to the model, generated answer, sources, latency, token cost,
feedback and confidence.

### LLM context is expensive and limited

Hippocore should evolve toward a **Context Compiler**:

```rust
db.build_context(query, user, max_tokens)
```

This should assemble facts, source documents, memories, contradictions,
citations, confidence and an optimized context package for the LLM.

### Local AI needs data sovereignty

The project must value local, offline and self-hosted execution. Sensitive data
should not require SaaS or cloud services.

## 3. Product Positioning

```txt
Hippocore DB is not just a vector database.
It is a memory and context engine for AI agents.
```

Short description:

```txt
Hippocore DB is an AI-native memory and context database written in Rust,
focused on RAG, agents, vector search, temporal memory and context auditability.
```

## 4. MVP Scope

The MVP must be small, functional, well-tested and extensible:

- embedded/local-first database;
- document store;
- disk persistence;
- append-only log;
- in-memory index;
- put/get/delete;
- recovery on open;
- embeddings as `Vec<f32>`;
- brute-force cosine vector search;
- simple metadata filters;
- CLI;
- tests;
- documentation;
- roadmap.

The MVP must not try to solve everything at once.

## 5. Not Yet in the MVP

Avoid pulling future phases too early:

- HNSW;
- advanced hybrid search;
- GraphRAG;
- HTTP/gRPC server;
- distributed cluster;
- production auth;
- complex permissions;
- SQL/query language;
- real multimodal processing;
- full Context Compiler;
- full RAG Audit Engine;
- full Temporal Truth Layer.

## 6. Future Architecture Engines

Long-term internal engines:

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

The foundation starts simple:

```txt
Document Store
  -> Append-only Log
  -> In-memory Index
  -> Brute-force Vector Search
  -> Metadata Filter
```

## 7. Golden Rules

### Rule 1 — Not Just a Vector Database

Every decision must support Hippocore as a memory and context database for AI.
Vector search is part of the product, not the product.

### Rule 2 — Reliable Context Over Similar Data

The goal is better answers with more trust, traceability and less hallucination.

### Rule 3 — Local-first Is Central

Prefer privacy, data sovereignty, offline execution, self-hosting and low vendor
lock-in.

### Rule 4 — Rust Is the Core Language

The core database must be Rust for performance, memory safety, predictable
allocation and a solid storage/indexing foundation.

### Rule 5 — Simple MVP, Not Disposable MVP

The MVP can be small, but must keep clean architecture and avoid hacks that block
future WAL, compaction, HNSW, Context Compiler, audit, temporal memory and graph
memory.

### Rule 6 — Do Not Implement Future Features Before the Foundation Is Solid

Persistence, recovery, tests, API, CLI, errors, documentation and benchmarks
come before advanced features.

### Rule 7 — Performance Must Be Measured

Never claim performance without benchmarks, profiling or latency/memory
measurements.

### Rule 8 — Important Information Needs Source and Confidence

Stored facts should be able to carry source, confidence, timestamps, version,
collection, metadata and future temporal validity.

### Rule 9 — Old Data May Be Superseded

The roadmap must distinguish current, old, superseded, contradictory and
uncertain information.

### Rule 10 — Audit Is Native

The database should evolve to trace how AI answers are produced.

### Rule 11 — Context Compiler Is Strategic

Returning chunks is not enough. Hippocore should assemble context packages for
LLMs.

### Rule 12 — Metadata and Filters Matter as Much as Vectors

Enterprise RAG requires tenant, environment, project, version, permission,
source, date, document type, collection and confidence filters.

### Rule 13 — Multitenancy and Permissions Must Shape the Design

Even before full server mode, choices must not block tenants, organizations,
projects, users, agents and collection-level permissions.

### Rule 14 — The Project Must Be Modular

Keep storage, document, index, vector, query, memory, audit, context, CLI and
future server responsibilities separated.

### Rule 15 — CLI and Public API Must Be Simple

The project must be easy to test and demonstrate.

### Rule 16 — Tests Are Part of the Product

Critical functionality must have deterministic tests.

### Rule 17 — Recovery Is Mandatory

A database that cannot recover persisted data after restart is not a database.

### Rule 18 — Errors Must Be Clear and Typed

Avoid vague errors. Public API failures should be typed and actionable.

### Rule 19 — Security and Privacy Are Roadmap Requirements

Future work should consider LGPD/GDPR, right to be forgotten, anonymization,
sensitive data removal, encryption, tenant isolation, access control and
auditable logs.

### Rule 20 — Documentation Is Architecture

Important decisions must be documented and easy for new contributors to
understand.

### Rule 21 — Documentation Under `docs/` Must Be Bilingual

Every document under `docs/` must exist in two versions:

```txt
docs/en/<file>.md
docs/pt-br/<file>.md
```

The versions do not need to be word-for-word translations, but they must preserve
the same decision, technical intent, scope, risks and acceptance criteria.

Practical rules:

- file names must match in both folders;
- internal links should point to the same-language document when possible;
- meaningful changes in one version require updating the other;
- architecture, roadmap, status, decisions and next-feature docs must never
  exist in only one language;
- root files such as `README.md`, `ROADMAP.md`, `CLAUDE.md` and `CHANGELOG.md`
  may remain at the repository root, but should point to localized docs when
  useful.

This rule exists because the project is open source and international, while its
origin and day-to-day design context include pt-BR.

## 8. Strategic Roadmap

1. Embedded foundation.
2. Robust storage.
3. Performant vector search.
4. Hybrid search.
5. Temporal Truth Layer.
6. Context Compiler.
7. RAG Audit Engine.
8. Graph Memory.
9. Server Mode.
10. Multimodal Storage.

## 9. Permanent Coherence Check

Before implementing a feature, ask:

1. Does it strengthen Hippocore as memory/context database for AI?
2. Does it help agents or RAG retrieve more reliable context?
3. Does it respect local-first and open source?
4. Does it keep architecture modular?
5. Does it have tests?
6. Does it have documentation?
7. Does it avoid unnecessary complexity?
8. Does it improve storage, retrieval, memory, audit, context or trust?

If the answer is mostly no, the feature is probably out of scope.

## 10. Final Definition

```txt
Hippocore DB is an AI-native, local-first, open-source database written in Rust,
created as the memory and context core for AI agents, enterprise RAG and
intelligent systems.
```

The highest rule:

> Hippocore DB exists to deliver reliable context to intelligent systems.
