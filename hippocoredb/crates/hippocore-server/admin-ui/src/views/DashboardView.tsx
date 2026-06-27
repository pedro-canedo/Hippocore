import type { Bootstrap, BootstrapStats } from "../api";

interface DashboardViewProps {
  username: string;
  data: Bootstrap | null;
  loading: boolean;
  error: string;
  onRefresh: () => void;
  onLogout: () => void;
}

const STAT_CARDS: ReadonlyArray<[keyof BootstrapStats, string]> = [
  ["tenants", "Tenants"],
  ["collections", "Collections"],
  ["memories", "Memories"],
  ["documents", "Documents"],
  ["records", "Records"],
  ["files", "Files"],
  ["graph_edges", "Graph edges"],
  ["indexed_entries", "Indexed entries"],
];

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function DashboardView({
  username,
  data,
  loading,
  error,
  onRefresh,
  onLogout,
}: DashboardViewProps) {
  return (
    <div className="shell">
      <header className="topbar glass">
        <div className="brand">
          <div className="brand-mark" aria-hidden="true" />
          <strong>Hippocore Control Plane</strong>
          <span className="pill">TypeScript preview</span>
        </div>
        <div className="topbar-actions">
          <span className="muted small">Signed in as {username}</span>
          <button className="btn" onClick={onRefresh} disabled={loading}>
            {loading ? "Refreshing…" : "Refresh"}
          </button>
          <button className="btn ghost" onClick={onLogout}>
            Sign out
          </button>
        </div>
      </header>

      <main className="content">
        {error ? <div className="notice error">{error}</div> : null}

        <section className="panel glass">
          <h2>Server status</h2>
          {data ? (
            <dl className="kv">
              <div>
                <dt>Data directory</dt>
                <dd className="mono">{data.config.data_dir}</dd>
              </div>
              <div>
                <dt>API key</dt>
                <dd>
                  {data.config.api_key_set
                    ? `configured (${data.config.api_key_length} chars)`
                    : "not set"}
                </dd>
              </div>
              <div>
                <dt>WAL entries</dt>
                <dd>{data.stats.wal_entries}</dd>
              </div>
              <div>
                <dt>Disk usage</dt>
                <dd>{formatBytes(data.stats.disk_bytes)}</dd>
              </div>
              <div>
                <dt>Audit log</dt>
                <dd>
                  {data.stats.audit_records} records /{" "}
                  {formatBytes(data.stats.audit_log_bytes)}
                </dd>
              </div>
            </dl>
          ) : (
            <p className="muted">{loading ? "Loading…" : "No data."}</p>
          )}
        </section>

        <section className="grid">
          {STAT_CARDS.map(([key, label]) => (
            <div className="stat-card glass" key={key}>
              <span className="stat-value">{data ? data.stats[key] : "—"}</span>
              <span className="stat-label">{label}</span>
            </div>
          ))}
        </section>

        <section className="panel glass">
          <h2>Tenants</h2>
          {data && data.tenants.length > 0 ? (
            <ul className="tenant-list">
              {data.tenants.map((t) => (
                <li key={t.id}>
                  <strong>{t.name}</strong> <span className="mono muted">{t.id}</span>
                </li>
              ))}
            </ul>
          ) : (
            <p className="muted">
              No tenants yet. Create one in the classic console at{" "}
              <a href="/admin">/admin</a> while pages are being migrated.
            </p>
          )}
        </section>
      </main>
    </div>
  );
}
