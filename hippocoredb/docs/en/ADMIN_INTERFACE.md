# Hippocore DB — Admin Interface Direction

## Product principle

Hippocore DB must be operable by both machines and humans.

SDKs and agents need programmatic context access. Humans need to inspect,
insert, correct, delete, import, evaluate, and audit the data that becomes
context. A context database is not trustworthy if users cannot see and manage
what it remembers.

## Access layers

### 1. Embedded core

The Rust core remains the source of truth:

- local data directory;
- WAL/recovery/compaction;
- document/memory/record/file storage;
- indexing and retrieval;
- Rust API.

This layer must not depend on a UI, cloud service, or server process.

### 2. CLI administration

The CLI is the first human administration surface. It should grow before a full
GUI:

```bash
hippocore list-tenants
hippocore list-collections
hippocore list-memories
hippocore list-documents
hippocore inspect --json
hippocore recall --debug
hippocore import-file
hippocore export
hippocore eval-retrieval
```

The CLI should make local debugging and examples possible without running a
server.

### 3. Hippocore Control Plane

The future user-facing admin UI should be a local-first management app, similar
in spirit to pgAdmin or DBeaver, but designed for memory/context instead of SQL.

Working name:

```txt
Hippocore Control Plane
```

Expected capabilities:

- browse tenants, collections, documents, memories, records, files;
- insert/edit/delete memories and structured records;
- import files and inspect derived context;
- inspect metadata, source, timestamps, version, and future confidence fields;
- test recall queries and view top-k results;
- show vector score, text score, final score, tags, and ranking reason;
- run retrieval-quality fixtures;
- view stats, WAL entries, disk usage, and compaction status;
- export/import data for local workflows.

### 4. Server/API mode

Server mode is useful later, but it should not become a prerequisite for the
embedded database.

Future command:

```bash
hippocore serve --db ./data
```

This can power Studio, SDKs, and remote integrations when the core is mature.

## DBeaver/pgAdmin compatibility

Direct DBeaver or pgAdmin compatibility is not the right early target.

Those tools assume SQL/JDBC/PostgreSQL wire protocol. Hippocore is not SQL-first;
its primary model is memory, documents, records, files, metadata, context, and
retrieval auditability.

Long-term options can be evaluated later:

- SQL subset for structured records;
- ODBC/JDBC bridge;
- PostgreSQL wire compatibility;
- read-only virtual tables for inspection.

These should come after the core data model, context projection, and admin
workflows are stable.

## Near-term recommendation

Build administration in this order:

1. Improve CLI inspection/list/import/export commands.
2. Define the stable data/context model.
3. Add structured records and file ingestion.
4. Add local Studio once there is enough data surface to manage.
5. Add server/API mode after the embedded workflow is solid.

This keeps Hippocore DB aligned with local-first principles while making it
practical for real users to curate the context their agents rely on.

### Current web admin surface

The server now exposes a local web admin console at `/admin` when you run
`hippocore serve`. It is intentionally simple and local-first:

- human login with bootstrap credentials from `HIPPOCORE_ADMIN_USER` and
  `HIPPOCORE_ADMIN_PASSWORD`;
- Docker Compose defaults to admin user `admin`; when
  `HIPPOCORE_ADMIN_PASSWORD` is unset, the entrypoint generates one password for
  the data volume, prints it in startup logs, and saves it at
  `/data/admin-password`;
- separate API key authentication for CLI, API clients, and machine-to-machine
  integrations;
- static frontend assets (`index.html`, `styles.css`, `app.js`) embedded in the
  server binary for simple Docker/local deployment;
- Portuguese, Spanish, and English UI copy;
- browse tenants and collections;
- create tenants and collections;
- store memories and documents;
- run recall and context queries;
- run read-only record SQL through `/admin/sql` or the compatible restricted
  records query (`select * from records ...`);
- configure local LLM providers for Ollama, OpenRouter, and compatible
  endpoints;
- rotate the active API key;

The admin console is meant to complement the CLI, not replace it.
