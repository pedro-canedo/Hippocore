/**
 * End-to-end RAG demo: Ollama (embeddings + generation) + Hippocore (memory).
 *
 *   1. Ingest a small knowledge base: embed each passage with Ollama and store
 *      it in Hippocore as a memory carrying that vector.
 *   2. Answer a question: embed the query, recall the most relevant passages
 *      from Hippocore (hybrid vector+text), then ask Ollama to answer using
 *      ONLY that retrieved context, with citations.
 *
 * Usage:  npm start              (uses a default question)
 *         npm start -- "sua pergunta aqui"
 */
import { createHash } from "node:crypto";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { Hippocore } from "./hippocore.js";
import { assertOllamaUp, chat, embed, CHAT_MODEL, EMBED_MODEL } from "./ollama.js";

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, "../../..");

// The compiled CLI binary and where Hippocore keeps its data.
const HIPPOCORE_BIN =
  process.env.HIPPOCORE_BIN ?? resolve(repoRoot, "target/debug/hippocore");
const DB_DIR = process.env.DB_DIR ?? resolve(here, "..", "data");

const TENANT = "demo";
const COLLECTION = "kb";

/** A tiny knowledge base. Stable ids => re-running overwrites, no duplicates. */
const KNOWLEDGE: { id: string; text: string; meta?: Record<string, string> }[] = [
  {
    id: "ora-12514",
    text: "O erro Oracle ORA-12514 significa que o listener nao conhece o servico solicitado. Verifique o SERVICE_NAME no tnsnames.ora e confirme que o servico esta registrado no listener com 'lsnrctl status'.",
    meta: { produto: "oracle", tipo: "erro" },
  },
  {
    id: "ora-12541",
    text: "O erro Oracle ORA-12541 indica que nao ha listener escutando no host/porta. Inicie o listener com 'lsnrctl start' e verifique o host e a porta no tnsnames.ora.",
    meta: { produto: "oracle", tipo: "erro" },
  },
  {
    id: "staging-policy",
    text: "No ambiente de homologacao a aplicacao roda Oracle 19c. Mudancas de configuracao do listener nesse ambiente exigem abertura de chamado antes da aplicacao.",
    meta: { produto: "oracle", ambiente: "staging" },
  },
  {
    id: "pg-vacuum",
    text: "No PostgreSQL, autovacuum remove tuplas mortas e evita inchaco de tabelas. Ajuste autovacuum_vacuum_scale_factor para tabelas grandes e muito atualizadas.",
    meta: { produto: "postgres", tipo: "tuning" },
  },
];

/** Marker file (inside the data dir) mapping passage id -> text hash, so we
 *  only re-embed passages that are new or whose text changed. */
const MARKER = resolve(DB_DIR, ".ingested.json");
const sha256 = (s: string) => createHash("sha256").update(s).digest("hex");

function loadMarker(): Record<string, string> {
  if (!existsSync(MARKER)) return {};
  try {
    return JSON.parse(readFileSync(MARKER, "utf8")) as Record<string, string>;
  } catch {
    return {};
  }
}

async function ingest(db: Hippocore): Promise<void> {
  const marker = loadMarker();
  let embedded = 0;
  let skipped = 0;
  for (const k of KNOWLEDGE) {
    const hash = sha256(k.text);
    if (marker[k.id] === hash) {
      skipped++;
      continue; // already ingested with identical text — no Ollama call
    }
    const embedding = await embed(k.text);
    db.remember({
      tenant: TENANT,
      collection: COLLECTION,
      id: k.id,
      type: "semantic",
      text: k.text,
      embedding,
      meta: k.meta,
    });
    marker[k.id] = hash;
    embedded++;
    console.log(`  stored ${k.id} (${embedding.length}-dim)`);
  }
  writeFileSync(MARKER, JSON.stringify(marker, null, 2));
  console.log(
    `Ingestion done: ${embedded} embedded, ${skipped} skipped (unchanged) ` +
      `via ${EMBED_MODEL}.`,
  );
}

async function ask(db: Hippocore, question: string): Promise<void> {
  console.log(`\nQuestion: ${question}\n`);

  // 1. Embed the query with the SAME model used for ingestion.
  const queryEmbedding = await embed(question);

  // 2. Recall the most relevant passages (hybrid: vector + keyword).
  const hits = db.recall({
    tenant: TENANT,
    query: question,
    embedding: queryEmbedding,
    mode: "hybrid",
    topK: 3,
  });

  console.log("Retrieved context:");
  for (const h of hits) {
    console.log(`  - [${h.id}] score=${h.score.toFixed(3)} (${h.reason})`);
  }
  if (hits.length === 0) {
    console.log("  (nothing relevant found)");
    return;
  }

  // 3. Build a grounded prompt and let Ollama generate the answer.
  const context = hits
    .map((h, i) => `[${i + 1}] (id=${h.id}) ${h.text}`)
    .join("\n");

  const answer = await chat([
    {
      role: "system",
      content:
        "Voce e um assistente de suporte. Responda em portugues usando APENAS o " +
        "contexto fornecido. Cite as fontes pelo id entre colchetes, ex: [ora-12514]. " +
        "Se o contexto nao contiver a resposta, diga que nao sabe.",
    },
    { role: "user", content: `Contexto:\n${context}\n\nPergunta: ${question}` },
  ]);

  console.log(`\nAnswer (via ${CHAT_MODEL}):\n${answer}\n`);
}

async function main(): Promise<void> {
  const question =
    process.argv.slice(2).join(" ").trim() ||
    "Como resolver o erro ORA-12514 no ambiente de homologacao?";

  await assertOllamaUp();

  const db = new Hippocore(HIPPOCORE_BIN, DB_DIR);
  db.init();

  await ingest(db);
  await ask(db, question);

  console.log("---\n" + db.stats());
  // Bound the WAL: fold accumulated writes into a fresh snapshot.
  console.log(db.compact().trim());
}

main().catch((err) => {
  console.error("\nDemo failed:", err instanceof Error ? err.message : err);
  process.exit(1);
});
