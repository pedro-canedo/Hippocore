# AGENTS.md — Hippocore DB

## Project Vision

Hippocore DB is an open-source, local-first, AI-native memory and context database written in Rust.

It is not just a vector database. It is a memory and context engine for AI agents, RAG systems, semantic memory, context engineering, retrieval auditing, and trustworthy AI applications.

The project exists to deliver reliable context to intelligent systems.

## Core Principles

1. Hippocore DB must not become just another vector database.
2. Retrieval quality, grounding, auditability, and context reliability matter as much as raw vector speed.
3. Local-first and embedded usage come before server/cloud features.
4. Rust is the core language.
5. Do not over-engineer the MVP.
6. Every critical feature must have tests.
7. Performance must be measured, not assumed.
8. The system must clearly distinguish documents, chunks, memories, indexed entries, and WAL entries.
9. RAG answers must be grounded in retrieved context.
10. If retrieved context is insufficient, the answer must say that instead of inventing.

## Current Architecture

Main crate: crates/hippocore  
Example RAG app: examples/ts-ollama-rag  
CLI binary: hippocore

Current working features include:
- memory ingestion
- deterministic/local embeddings or Ollama embeddings depending on example
- hybrid retrieval
- vector score
- text score
- memory recall
- stats
- compaction
- TypeScript Ollama RAG example

## Important Commands

Rust checks:

```bash
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p hippocore

TypeScript RAG example:

cd examples/ts-ollama-rag
npm start -- "Como inicio o listener do Oracle e postgress?"
npm start -- "Como postgress funciona?"
npm start -- "como faço uma conexão python no postgress?"
Current Known Issue

The RAG example retrieves Oracle memories too strongly for PostgreSQL/Python questions.

Example problem:
A question about Python connection to PostgreSQL ranked Oracle errors before PostgreSQL memories.

This must be fixed through retrieval quality improvements, not by hard-coding exact questions.

Scope Rules

Allowed now:

query normalization
typo handling
technology/entity detection
simple retrieval boost/penalty
better PostgreSQL seed memories
stricter answer prompt grounding
retrieval debug output
tests for retrieval behavior
docs explaining documents/chunks/memories/indexed entries

Not allowed yet:

HNSW
server mode
distributed cluster
GraphRAG
complex query language
full Temporal Truth Layer
full Context Compiler
huge architectural rewrite
Done Means

Before finishing a task:

Run Rust tests.
Run fmt.
Run clippy.
Validate the TypeScript RAG example when the change affects examples.
Summarize changed files.
Include before/after behavior when fixing retrieval.