import { useCallback, useEffect, useState } from "react";
import {
  ApiError,
  apiInfo as fetchApiInfo,
  listProviders,
  pingProvider,
  upsertProvider,
  validateProvider,
  type ApiInfo,
  type PingResult,
  type ProviderConfig,
  type ProviderView,
} from "../api";

interface IntegrationsViewProps {
  session: string;
  onChanged: (message: string) => void;
}

const BLANK: ProviderConfig = {
  id: "",
  kind: "ollama",
  base_url: "http://localhost:11434",
  model: "llama3.2",
  api_key: "",
  is_default: true,
};

export function IntegrationsView({ session, onChanged }: IntegrationsViewProps) {
  const [providers, setProviders] = useState<ProviderView[]>([]);
  const [info, setInfo] = useState<ApiInfo | null>(null);
  const [form, setForm] = useState<ProviderConfig>(BLANK);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState<
    { ok: boolean; message: string; latency?: number } | null
  >(null);

  const refresh = useCallback(async () => {
    try {
      const [list, apiInf] = await Promise.all([
        listProviders(session),
        fetchApiInfo(session).catch(() => null),
      ]);
      setProviders(list);
      setInfo(apiInf);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }, [session]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  function set<K extends keyof ProviderConfig>(key: K, value: ProviderConfig[K]) {
    setForm((f) => ({ ...f, [key]: value }));
  }

  async function save() {
    setError("");
    setBusy(true);
    try {
      const saved = await upsertProvider(session, form);
      onChanged(`Provider "${saved.id}" saved.`);
      await refresh();
    } catch (err) {
      setError(err instanceof ApiError ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  async function validate() {
    setError("");
    setResult(null);
    try {
      const res = await validateProvider(session, form);
      setResult({ ok: res.ok, message: res.message });
    } catch (err) {
      setError(err instanceof ApiError ? err.message : String(err));
    }
  }

  async function ping() {
    if (!form.id.trim()) {
      setError("Enter the provider id to ping.");
      return;
    }
    setError("");
    setResult(null);
    try {
      const res: PingResult = await pingProvider(session, form.id.trim());
      setResult({ ok: res.ok, message: res.message, latency: res.latency_ms });
    } catch (err) {
      setError(err instanceof ApiError ? err.message : String(err));
    }
  }

  return (
    <>
      <section className="cols">
        <div className="panel glass">
          <h2>LLM providers</h2>
          {providers.length > 0 ? (
            <table className="table">
              <thead>
                <tr>
                  <th>id</th>
                  <th>kind</th>
                  <th>model</th>
                  <th>key</th>
                  <th>default</th>
                </tr>
              </thead>
              <tbody>
                {providers.map((p) => (
                  <tr
                    key={p.id}
                    onClick={() =>
                      setForm({
                        id: p.id,
                        kind: p.kind,
                        base_url: p.base_url,
                        model: p.model,
                        api_key: "",
                        is_default: p.is_default,
                      })
                    }
                  >
                    <td className="mono">{p.id}</td>
                    <td>{p.kind}</td>
                    <td>{p.model}</td>
                    <td>{p.api_key_set ? "set" : "—"}</td>
                    <td>{p.is_default ? "✓" : ""}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          ) : (
            <p className="muted">No providers configured. Add one.</p>
          )}
        </div>

        <div className="panel glass">
          <h2>Add / update provider</h2>
          <label className="field">
            <span>Provider id</span>
            <input value={form.id} onChange={(e) => set("id", e.target.value)} />
          </label>
          <label className="field">
            <span>Kind</span>
            <select value={form.kind} onChange={(e) => set("kind", e.target.value)}>
              <option value="ollama">ollama</option>
              <option value="openrouter">openrouter</option>
              <option value="openai">openai</option>
            </select>
          </label>
          <label className="field">
            <span>Base URL</span>
            <input
              value={form.base_url}
              onChange={(e) => set("base_url", e.target.value)}
            />
          </label>
          <label className="field">
            <span>Model</span>
            <input value={form.model} onChange={(e) => set("model", e.target.value)} />
          </label>
          <label className="field">
            <span>API key (optional, write-only)</span>
            <input
              type="password"
              value={form.api_key ?? ""}
              onChange={(e) => set("api_key", e.target.value)}
            />
          </label>
          <label className="checkbox">
            <input
              type="checkbox"
              checked={form.is_default}
              onChange={(e) => set("is_default", e.target.checked)}
            />
            <span>default provider</span>
          </label>
          {error ? <div className="notice error">{error}</div> : null}
          {result ? (
            <div className={`notice ${result.ok ? "success" : "error"}`}>
              {result.ok ? "✓ " : "✗ "}
              {result.message}
              {result.latency !== undefined ? ` (${result.latency} ms)` : ""}
            </div>
          ) : null}
          <div className="page-actions">
            <button className="btn primary" onClick={() => void save()} disabled={busy}>
              {busy ? "Saving…" : "Save"}
            </button>
            <button className="btn" onClick={() => void validate()}>
              Validate
            </button>
            <button className="btn" onClick={() => void ping()}>
              Ping
            </button>
          </div>
        </div>
      </section>

      <section className="panel glass">
        <h2>API exposure</h2>
        {info ? (
          <dl className="kv">
            <div>
              <dt>Base URL</dt>
              <dd className="mono">{info.base_url}</dd>
            </div>
            <div>
              <dt>API key</dt>
              <dd className="mono">{info.api_key_hint}</dd>
            </div>
          </dl>
        ) : (
          <p className="muted">API info unavailable.</p>
        )}
        <pre className="json">{`curl -X POST ${info?.base_url ?? "http://localhost:8080"}/tenants/TENANT/recall \\
  -H "x-api-key: YOUR_KEY" \\
  -H "content-type: application/json" \\
  -d '{"query":"hello","collection":"main"}'`}</pre>
      </section>
    </>
  );
}
