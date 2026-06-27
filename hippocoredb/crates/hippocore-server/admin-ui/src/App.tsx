import { useCallback, useEffect, useState } from "react";
import {
  ApiError,
  bootstrap as fetchBootstrap,
  listCollections,
  listTenants,
  login as apiLogin,
  type Bootstrap,
  type Collection,
  type Tenant,
} from "./api";
import {
  clearSession,
  loadActiveTenant,
  loadSession,
  loadUsername,
  saveActiveTenant,
  saveSession,
} from "./session";
import type { View } from "./views";
import { Shell } from "./components/Shell";
import { Toast } from "./components/Toast";
import { LoginView } from "./views/LoginView";
import { DashboardView } from "./views/DashboardView";
import { TenantsView } from "./views/TenantsView";
import { CollectionsView } from "./views/CollectionsView";
import { DataExplorerView } from "./views/DataExplorerView";
import { SqlEditorView } from "./views/SqlEditorView";
import { IngestionView } from "./views/IngestionView";

export function App() {
  const [session, setSession] = useState<string>(() => loadSession());
  const [username, setUsername] = useState<string>(() => loadUsername());
  const [view, setView] = useState<View>("dashboard");
  const [data, setData] = useState<Bootstrap | null>(null);
  const [tenants, setTenants] = useState<Tenant[]>([]);
  const [collections, setCollections] = useState<Collection[]>([]);
  const [activeTenant, setActiveTenant] = useState<string>(() =>
    loadActiveTenant(),
  );
  const [error, setError] = useState<string>("");
  const [loading, setLoading] = useState<boolean>(false);
  const [toast, setToast] = useState<string>("");

  const handleAuthError = useCallback((err: unknown): boolean => {
    if (err instanceof ApiError && err.status === 401) {
      clearSession();
      setSession("");
      setData(null);
      setTenants([]);
      setCollections([]);
      setError("Session expired. Please sign in again.");
      return true;
    }
    return false;
  }, []);

  const refresh = useCallback(
    async (token: string) => {
      setLoading(true);
      setError("");
      try {
        const [boot, tns] = await Promise.all([
          fetchBootstrap(token),
          listTenants(token),
        ]);
        setData(boot);
        setTenants(tns);
      } catch (err) {
        if (!handleAuthError(err)) {
          setError(err instanceof Error ? err.message : String(err));
        }
      } finally {
        setLoading(false);
      }
    },
    [handleAuthError],
  );

  const loadCollections = useCallback(
    async (token: string, tenantId: string) => {
      if (!tenantId) {
        setCollections([]);
        return;
      }
      try {
        setCollections(await listCollections(token, tenantId));
      } catch (err) {
        if (!handleAuthError(err)) {
          setError(err instanceof Error ? err.message : String(err));
        }
      }
    },
    [handleAuthError],
  );

  useEffect(() => {
    if (session) void refresh(session);
  }, [session, refresh]);

  useEffect(() => {
    if (
      session &&
      (view === "collections" || view === "explorer" || view === "ingestion")
    ) {
      void loadCollections(session, activeTenant);
    }
  }, [session, view, activeTenant, loadCollections]);

  useEffect(() => {
    if (!toast) return;
    const handle = window.setTimeout(() => setToast(""), 2600);
    return () => window.clearTimeout(handle);
  }, [toast]);

  const handleLogin = useCallback(async (user: string, password: string) => {
    setError("");
    setLoading(true);
    try {
      const res = await apiLogin(user, password);
      saveSession(res.session, res.username);
      setUsername(res.username);
      setSession(res.session);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  const handleLogout = useCallback(() => {
    clearSession();
    setSession("");
    setData(null);
    setTenants([]);
    setCollections([]);
    setView("dashboard");
  }, []);

  const selectTenant = useCallback((tenantId: string) => {
    setActiveTenant(tenantId);
    saveActiveTenant(tenantId);
  }, []);

  const onTenantsChanged = useCallback(
    (message: string) => {
      setToast(message);
      if (session) void refresh(session);
    },
    [session, refresh],
  );

  const onCollectionsChanged = useCallback(
    (message: string) => {
      setToast(message);
      if (session) {
        void refresh(session);
        void loadCollections(session, activeTenant);
      }
    },
    [session, activeTenant, refresh, loadCollections],
  );

  if (!session) {
    return (
      <LoginView
        username={username}
        loading={loading}
        error={error}
        onSubmit={handleLogin}
      />
    );
  }

  return (
    <Shell
      username={username}
      view={view}
      tenants={tenants}
      activeTenant={activeTenant}
      loading={loading}
      onNavigate={setView}
      onSelectTenant={selectTenant}
      onRefresh={() => void refresh(session)}
      onLogout={handleLogout}
    >
      {error ? <div className="notice error">{error}</div> : null}
      {view === "dashboard" && (
        <DashboardView data={data} loading={loading} onNavigate={setView} />
      )}
      {view === "tenants" && (
        <TenantsView
          session={session}
          tenants={tenants}
          loading={loading}
          onChanged={onTenantsChanged}
        />
      )}
      {view === "collections" && (
        <CollectionsView
          session={session}
          activeTenant={activeTenant}
          collections={collections}
          loading={loading}
          onChanged={onCollectionsChanged}
        />
      )}
      {view === "explorer" && (
        <DataExplorerView
          session={session}
          activeTenant={activeTenant}
          collections={collections}
        />
      )}
      {view === "ingestion" && (
        <IngestionView
          session={session}
          activeTenant={activeTenant}
          collections={collections}
          onChanged={onCollectionsChanged}
        />
      )}
      {view === "sql" && (
        <SqlEditorView session={session} activeTenant={activeTenant} />
      )}
      <Toast message={toast} />
    </Shell>
  );
}
