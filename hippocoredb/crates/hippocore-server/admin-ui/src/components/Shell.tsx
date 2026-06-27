import type { ReactNode } from "react";
import type { Tenant } from "../api";
import { NAV_ITEMS, type View } from "../views";

interface ShellProps {
  username: string;
  view: View;
  tenants: Tenant[];
  activeTenant: string;
  loading: boolean;
  onNavigate: (view: View) => void;
  onSelectTenant: (tenantId: string) => void;
  onRefresh: () => void;
  onLogout: () => void;
  children: ReactNode;
}

export function Shell({
  username,
  view,
  tenants,
  activeTenant,
  loading,
  onNavigate,
  onSelectTenant,
  onRefresh,
  onLogout,
  children,
}: ShellProps) {
  return (
    <div className="layout">
      <aside className="sidebar glass">
        <div className="brand">
          <div className="brand-mark" aria-hidden="true" />
          <strong>Hippocore</strong>
          <span className="pill">TS</span>
        </div>
        <nav className="nav">
          {NAV_ITEMS.map((item) => (
            <button
              key={item.view}
              className={`nav-item${view === item.view ? " active" : ""}`}
              onClick={() => onNavigate(item.view)}
            >
              {item.label}
            </button>
          ))}
        </nav>
        <div className="sidebar-foot">
          <a href="/admin" className="muted small">
            Classic console →
          </a>
        </div>
      </aside>

      <div className="main">
        <header className="topbar glass">
          <label className="tenant-select">
            <span className="muted small">Tenant</span>
            <select
              value={activeTenant}
              onChange={(e) => onSelectTenant(e.target.value)}
            >
              <option value="">— none —</option>
              {tenants.map((t) => (
                <option key={t.id} value={t.id}>
                  {t.name} ({t.id})
                </option>
              ))}
            </select>
          </label>
          <div className="topbar-actions">
            <span className="muted small">{username}</span>
            <button className="btn" onClick={onRefresh} disabled={loading}>
              {loading ? "…" : "Refresh"}
            </button>
            <button className="btn ghost" onClick={onLogout}>
              Sign out
            </button>
          </div>
        </header>
        <main className="content">{children}</main>
      </div>
    </div>
  );
}
