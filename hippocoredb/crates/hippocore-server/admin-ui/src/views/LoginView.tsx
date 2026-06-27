import { useState, type FormEvent } from "react";

interface LoginViewProps {
  username: string;
  loading: boolean;
  error: string;
  onSubmit: (username: string, password: string) => void;
}

export function LoginView({
  username,
  loading,
  error,
  onSubmit,
}: LoginViewProps) {
  const [user, setUser] = useState(username);
  const [password, setPassword] = useState("");

  function submit(event: FormEvent) {
    event.preventDefault();
    onSubmit(user.trim(), password);
  }

  return (
    <div className="login-screen">
      <form className="login-card glass" onSubmit={submit}>
        <div className="brand-mark" aria-hidden="true" />
        <h1>Hippocore Control Plane</h1>
        <p className="muted">AI-native memory database — admin console</p>
        <label className="field">
          <span>Admin user</span>
          <input
            value={user}
            autoComplete="username"
            onChange={(e) => setUser(e.target.value)}
          />
        </label>
        <label className="field">
          <span>Password</span>
          <input
            type="password"
            value={password}
            autoComplete="current-password"
            onChange={(e) => setPassword(e.target.value)}
          />
        </label>
        {error ? <div className="notice error">{error}</div> : null}
        <button className="btn primary" type="submit" disabled={loading}>
          {loading ? "Signing in…" : "Sign in"}
        </button>
        <p className="muted small">
          New TypeScript console (preview). The classic console stays at{" "}
          <a href="/admin">/admin</a>.
        </p>
      </form>
    </div>
  );
}
