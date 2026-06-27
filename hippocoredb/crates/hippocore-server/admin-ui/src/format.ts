import type { RecordRow } from "./api";

/** Union of payload keys across record rows, sorted for stable columns. */
export function recordColumns(rows: RecordRow[]): string[] {
  const keys = new Set<string>();
  for (const row of rows) {
    if (row.payload && typeof row.payload === "object") {
      for (const k of Object.keys(row.payload)) keys.add(k);
    }
  }
  return Array.from(keys).sort();
}

/** Render a cell value as a compact string. */
export function cell(value: unknown): string {
  if (value === null || value === undefined) return "—";
  if (typeof value === "object") return JSON.stringify(value);
  return String(value);
}

export function truncate(text: string, max = 80): string {
  return text.length > max ? `${text.slice(0, max)}…` : text;
}
