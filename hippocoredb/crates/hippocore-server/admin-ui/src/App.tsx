import { useCallback, useEffect, useState } from "react";
import {
  ApiError,
  bootstrap as fetchBootstrap,
  login as apiLogin,
  type Bootstrap,
} from "./api";
import {
  clearSession,
  loadSession,
  loadUsername,
  saveSession,
} from "./session";
import { LoginView } from "./views/LoginView";
import { DashboardView } from "./views/DashboardView";

export function App() {
  const [session, setSession] = useState<string>(() => loadSession());
  const [username, setUsername] = useState<string>(() => loadUsername());
  const [data, setData] = useState<Bootstrap | null>(null);
  const [error, setError] = useState<string>("");
  const [loading, setLoading] = useState<boolean>(false);

  const refresh = useCallback(async (token: string) => {
    setLoading(true);
    setError("");
    try {
      setData(await fetchBootstrap(token));
    } catch (err) {
      if (err instanceof ApiError && err.status === 401) {
        clearSession();
        setSession("");
        setData(null);
        setError("Session expired. Please sign in again.");
      } else {
        setError(err instanceof Error ? err.message : String(err));
      }
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (session) void refresh(session);
  }, [session, refresh]);

  const handleLogin = useCallback(
    async (user: string, password: string) => {
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
    },
    [],
  );

  const handleLogout = useCallback(() => {
    clearSession();
    setSession("");
    setData(null);
  }, []);

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
    <DashboardView
      username={username}
      data={data}
      loading={loading}
      error={error}
      onRefresh={() => void refresh(session)}
      onLogout={handleLogout}
    />
  );
}
