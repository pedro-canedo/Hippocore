import type { Bootstrap, BootstrapStats } from "../api";
import type { View } from "../views";

interface DashboardViewProps {
  data: Bootstrap | null;
  loading: boolean;
  onNavigate: (view: View) => void;
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
  data,
  loading,
  onNavigate,
}: DashboardViewProps) {
  return (
    <>
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
          <button
            className="stat-card glass as-button"
            key={key}
            onClick={() =>
              onNavigate(key === "collections" ? "collections" : "tenants")
            }
          >
            <span className="stat-value">{data ? data.stats[key] : "—"}</span>
            <span className="stat-label">{label}</span>
          </button>
        ))}
      </section>

      <section className="panel glass">
        <h2>Get started</h2>
        <p className="muted">
          Manage <button className="link" onClick={() => onNavigate("tenants")}>
            Tenants
          </button>{" "}
          and{" "}
          <button className="link" onClick={() => onNavigate("collections")}>
            Collections
          </button>{" "}
          here. Other pages are still in the classic console at{" "}
          <a href="/admin">/admin</a> while the rebuild continues.
        </p>
      </section>
    </>
  );
}
