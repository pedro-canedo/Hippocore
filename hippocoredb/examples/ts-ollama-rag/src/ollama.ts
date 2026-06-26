/**
 * Minimal Ollama client (embeddings + chat) using the local HTTP API.
 * Requires `ollama serve` running and the models pulled (see README).
 */
const OLLAMA_URL = process.env.OLLAMA_URL ?? "http://localhost:11434";
export const EMBED_MODEL = process.env.EMBED_MODEL ?? "nomic-embed-text";
export const CHAT_MODEL = process.env.CHAT_MODEL ?? "qwen3-coder:30b";

export interface ChatMessage {
  role: "system" | "user" | "assistant";
  content: string;
}

/** Embed a single piece of text into a vector. */
export async function embed(text: string, model = EMBED_MODEL): Promise<number[]> {
  const res = await fetch(`${OLLAMA_URL}/api/embeddings`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ model, prompt: text }),
  });
  if (!res.ok) {
    throw new Error(`Ollama embeddings failed (${res.status}): ${await res.text()}`);
  }
  const json = (await res.json()) as { embedding?: number[] };
  if (!json.embedding?.length) {
    throw new Error(`Ollama returned an empty embedding for model ${model}`);
  }
  return json.embedding;
}

/** Run a non-streaming chat completion and return the assistant text. */
export async function chat(messages: ChatMessage[], model = CHAT_MODEL): Promise<string> {
  const res = await fetch(`${OLLAMA_URL}/api/chat`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ model, messages, stream: false }),
  });
  if (!res.ok) {
    throw new Error(`Ollama chat failed (${res.status}): ${await res.text()}`);
  }
  const json = (await res.json()) as { message?: { content?: string } };
  return json.message?.content ?? "";
}

/** Friendly check that Ollama is reachable before we start. */
export async function assertOllamaUp(): Promise<void> {
  try {
    const res = await fetch(`${OLLAMA_URL}/api/tags`);
    if (!res.ok) throw new Error(`status ${res.status}`);
  } catch (e) {
    throw new Error(
      `Cannot reach Ollama at ${OLLAMA_URL}. Is \`ollama serve\` running? (${String(e)})`,
    );
  }
}
