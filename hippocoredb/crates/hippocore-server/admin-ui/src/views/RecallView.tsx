import { useState, type FormEvent } from "react";
import { ApiError, recall, type Collection, type RecallItem } from "../api";
import { truncate } from "../format";

interface RecallViewProps {
  session: string;
  activeTenant: string;
  collections: Collection[];
}

export function RecallView({
  session,
  activeTenant,
  collections,
}: RecallViewProps) {
  const [query, setQuery] = useState("");
  const [collection, setCollection] = useState("");
  const [topK, setTopK] = useState(5);
  const [dedup, setDedup] = useState(false);
  const [mmr, setMmr] = useState(false);
  const [results, setResults] = useState<RecallItem[] | null>(null);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [raw, setRaw] = useState(false);

  if (!activeTenant) {
    return (
      <section className="panel glass">
        <h2>Recall</h2>
        <p className="muted">
          Select a tenant in the top bar to test retrieval against its memory.
        </p>
      </section>
    );
  }

  async function run(event: FormEvent) {
    event.preventDefault();
    const q = query.trim();
    if (!q) return;
    setBusy(true);
    setError("");
    try {
      const body = {
        query: q,
        collection: collection || undefined,
        top_k: topK,
        dedup_chunks: dedup,
        mmr,
      };
      setResults(await recall(session, activeTenant, body));
    } catch (err) {
      setResults(null);
      setError(err instanceof ApiError ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="panel glass">
      <div className="explorer-head">
        <h2>
          Recall <span className="muted small">in {activeTenant}</span>
        </h2>
        {results ? (
          <button className="btn small-btn" onClick={() => setRaw((v) => !v)}>
            {raw ? "Cards" : "Raw JSON"}
          </button>
        ) : null}
      </div>

      <form onSubmit={run}>
        <label className="field">
          <span>Query</span>
          <input
            value={query}
            placeholder="What do you want to recall?"
            onChange={(e) => setQuery(e.target.value)}
          />
        </label>
        <div className="recall-controls">
          <label className="field">
            <span>Collection</span>
            <select
              value={collection}
              onChange={(e) => setCollection(e.target.value)}
            >
              <option value="">all collections</option>
              {collections.map((c) => (
                <option key={c.name} value={c.name}>
                  {c.name}
                </option>
              ))}
            </select>
          </label>
          <label className="field">
            <span>top_k</span>
            <input
              type="number"
              min={1}
              max={50}
              value={topK}
              onChange={(e) => setTopK(Number(e.target.value) || 1)}
            />
          </label>
          <label className="checkbox">
            <input
              type="checkbox"
              checked={dedup}
              onChange={(e) => setDedup(e.target.checked)}
            />
            <span>dedup chunks</span>
          </label>
          <label className="checkbox">
            <input
              type="checkbox"
              checked={mmr}
              onChange={(e) => setMmr(e.target.checked)}
            />
            <span>MMR diversity</span>
          </label>
        </div>
        <div className="page-actions">
          <button className="btn primary" type="submit" disabled={busy || !query.trim()}>
            {busy ? "Recalling…" : "Recall"}
          </button>
          {results ? (
            <span className="muted small">{results.length} result(s)</span>
          ) : null}
        </div>
      </form>

      {error ? <div className="notice error">{error}</div> : null}

      {results && raw ? (
        <pre className="json">{JSON.stringify(results, null, 2)}</pre>
      ) : null}

      {results && !raw ? (
        results.length === 0 ? (
          <p className="muted">No results for this query.</p>
        ) : (
          <div className="recall-cards">
            {results.map((r) => (
              <article className="recall-card glass" key={r.id}>
                <header className="recall-card-head">
                  <span className="pill">{r.kind}</span>
                  <span className="muted small">score {r.score.toFixed(4)}</span>
                </header>
                <p className="recall-text">{truncate(r.text, 240)}</p>
                {r.matched_terms.length > 0 ? (
                  <div className="terms">
                    {r.matched_terms.map((t) => (
                      <span className="term" key={t}>
                        {t}
                      </span>
                    ))}
                  </div>
                ) : null}
                <footer className="muted small mono">{r.id}</footer>
              </article>
            ))}
          </div>
        )
      ) : null}
    </section>
  );
}
