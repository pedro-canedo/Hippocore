import { useState, type KeyboardEvent } from "react";
import { ApiError, runSql, type SqlResult } from "../api";
import { cell, recordColumns } from "../format";

interface SqlEditorViewProps {
  session: string;
  activeTenant: string;
}

const EXAMPLES: ReadonlyArray<[string, string]> = [
  ["All rows", "select * from systems limit 20"],
  ["Filter", "select * from systems where engine = 'postgresql' limit 10"],
];

export function SqlEditorView({ session, activeTenant }: SqlEditorViewProps) {
  const [sql, setSql] = useState("select * from systems limit 20");
  const [result, setResult] = useState<SqlResult | null>(null);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [durationMs, setDurationMs] = useState<number | null>(null);
  const [raw, setRaw] = useState(false);

  if (!activeTenant) {
    return (
      <section className="panel glass">
        <h2>SQL Editor</h2>
        <p className="muted">
          Select a tenant in the top bar to run queries against its records.
        </p>
      </section>
    );
  }

  async function run() {
    const trimmed = sql.trim();
    if (!trimmed) return;
    setBusy(true);
    setError("");
    const started = performance.now();
    try {
      const res = await runSql(session, activeTenant, trimmed);
      setResult(res);
      setDurationMs(Math.round(performance.now() - started));
    } catch (err) {
      setResult(null);
      setError(err instanceof ApiError ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  function onKeyDown(event: KeyboardEvent<HTMLTextAreaElement>) {
    if ((event.metaKey || event.ctrlKey) && event.key === "Enter") {
      event.preventDefault();
      void run();
    }
  }

  const cols = result ? recordColumns(result.rows) : [];

  return (
    <section className="panel glass">
      <div className="explorer-head">
        <h2>
          SQL Editor <span className="muted small">in {activeTenant}</span>
        </h2>
        <div className="examples">
          {EXAMPLES.map(([label, query]) => (
            <button
              key={label}
              className="btn small-btn"
              onClick={() => setSql(query)}
            >
              {label}
            </button>
          ))}
        </div>
      </div>

      <textarea
        className="sql-input"
        value={sql}
        spellCheck={false}
        onChange={(e) => setSql(e.target.value)}
        onKeyDown={onKeyDown}
        rows={5}
      />
      <p className="muted small">
        Read-only <code>SELECT</code> over record payloads. Unprefixed fields map
        to <code>payload.&lt;field&gt;</code>. Press <kbd>Ctrl/Cmd</kbd> +{" "}
        <kbd>Enter</kbd> to run.
      </p>
      <div className="page-actions">
        <button className="btn primary" onClick={() => void run()} disabled={busy}>
          {busy ? "Running…" : "Run"}
        </button>
        {result ? (
          <button className="btn" onClick={() => setRaw((v) => !v)}>
            {raw ? "Table" : "Raw JSON"}
          </button>
        ) : null}
        {durationMs !== null && result ? (
          <span className="muted small">
            {result.row_count} row(s) · {durationMs} ms
          </span>
        ) : null}
      </div>

      {error ? <div className="notice error">{error}</div> : null}

      {result && !raw ? (
        result.rows.length === 0 ? (
          <p className="muted">No rows.</p>
        ) : (
          <div className="table-scroll">
            <table className="table">
              <thead>
                <tr>
                  <th>id</th>
                  <th>table</th>
                  {cols.map((c) => (
                    <th key={c}>{c}</th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {result.rows.map((r) => (
                  <tr key={`${r.table}/${r.id}`}>
                    <td className="mono">{r.id}</td>
                    <td>{r.table}</td>
                    {cols.map((c) => (
                      <td key={c}>{cell(r.payload?.[c])}</td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )
      ) : null}

      {result && raw ? (
        <pre className="json">{JSON.stringify(result, null, 2)}</pre>
      ) : null}
    </section>
  );
}
