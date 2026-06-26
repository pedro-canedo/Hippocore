/**
 * Thin TypeScript wrapper over the `hippocore` CLI binary.
 *
 * We talk to Hippocore by spawning the binary as a subprocess (the MVP is
 * embedded/local-first and has no HTTP server yet). Embeddings are passed in
 * explicitly so an external model (Ollama) provides the vectors.
 */
import { execFileSync } from "node:child_process";

export type ItemKind = "memory" | "document_chunk" | "record";
export type SearchMode = "hybrid" | "vector" | "text";

export interface RecallHit {
  id: string;
  kind: ItemKind;
  tenant_id: string;
  collection: string;
  document_id: string | null;
  record_table: string | null;
  user_id: string | null;
  memory_type: string | null;
  text: string;
  metadata: Record<string, string>;
  source: { label: string; uri?: string; title?: string } | null;
  score: number;
  vector_score: number;
  text_score: number;
  reason: string;
}

export interface RememberOptions {
  tenant: string;
  collection: string;
  text: string;
  embedding: number[];
  id?: string;
  type?: "episodic" | "semantic" | "procedural" | "note";
  user?: string;
  meta?: Record<string, string>;
}

export interface RecallOptions {
  tenant: string;
  query?: string;
  embedding?: number[];
  topK?: number;
  mode?: SearchMode;
  collection?: string;
  meta?: Record<string, string>;
}

const toVec = (v: number[]) => v.join(",");
const metaArgs = (meta?: Record<string, string>): string[] =>
  Object.entries(meta ?? {}).flatMap(([k, val]) => ["--meta", `${k}=${val}`]);

export class Hippocore {
  constructor(
    private readonly bin: string,
    private readonly db: string,
  ) {}

  private run(args: string[]): string {
    return execFileSync(this.bin, args, {
      encoding: "utf8",
      maxBuffer: 32 * 1024 * 1024,
    });
  }

  init(): void {
    this.run(["init", "--db", this.db]);
  }

  remember(o: RememberOptions): void {
    const args = [
      "remember",
      "--db", this.db,
      "--tenant", o.tenant,
      "--collection", o.collection,
      "--type", o.type ?? "semantic",
      "--text", o.text,
      "--embedding", toVec(o.embedding),
      ...(o.id ? ["--id", o.id] : []),
      ...(o.user ? ["--user", o.user] : []),
      ...metaArgs(o.meta),
    ];
    this.run(args);
  }

  recall(o: RecallOptions): RecallHit[] {
    const args = [
      "recall",
      "--db", this.db,
      "--tenant", o.tenant,
      "--query", o.query ?? "",
      "--mode", o.mode ?? "hybrid",
      "--top-k", String(o.topK ?? 5),
      "--json",
      ...(o.embedding ? ["--embedding", toVec(o.embedding)] : []),
      ...(o.collection ? ["--collection", o.collection] : []),
      ...metaArgs(o.meta),
    ];
    return JSON.parse(this.run(args)) as RecallHit[];
  }

  forget(o: { tenant: string; collection: string; id: string }): void {
    this.run([
      "forget",
      "--db", this.db,
      "--tenant", o.tenant,
      "--collection", o.collection,
      "--id", o.id,
    ]);
  }

  stats(): string {
    return this.run(["stats", "--db", this.db]);
  }

  /** Fold the WAL into a fresh snapshot and truncate the log. */
  compact(): string {
    return this.run(["compact", "--db", this.db]);
  }
}
