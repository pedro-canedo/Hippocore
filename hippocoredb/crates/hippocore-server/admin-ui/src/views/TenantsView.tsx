import { useState, type FormEvent } from "react";
import { ApiError, createTenant, type Tenant } from "../api";

interface TenantsViewProps {
  session: string;
  tenants: Tenant[];
  loading: boolean;
  onChanged: (message: string) => void;
}

export function TenantsView({
  session,
  tenants,
  loading,
  onChanged,
}: TenantsViewProps) {
  const [id, setId] = useState("");
  const [name, setName] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);

  async function submit(event: FormEvent) {
    event.preventDefault();
    setError("");
    setBusy(true);
    try {
      const created = await createTenant(session, id.trim(), name.trim());
      setId("");
      setName("");
      onChanged(`Tenant "${created.name}" created.`);
    } catch (err) {
      setError(err instanceof ApiError ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="cols">
      <div className="panel glass">
        <h2>Tenants</h2>
        {tenants.length > 0 ? (
          <table className="table">
            <thead>
              <tr>
                <th>Name</th>
                <th>ID</th>
              </tr>
            </thead>
            <tbody>
              {tenants.map((t) => (
                <tr key={t.id}>
                  <td>{t.name}</td>
                  <td className="mono muted">{t.id}</td>
                </tr>
              ))}
            </tbody>
          </table>
        ) : (
          <p className="muted">
            {loading ? "Loading…" : "No tenants yet. Create the first one."}
          </p>
        )}
      </div>

      <div className="panel glass">
        <h2>New tenant</h2>
        <form onSubmit={submit}>
          <label className="field">
            <span>Tenant ID</span>
            <input
              value={id}
              placeholder="acme"
              onChange={(e) => setId(e.target.value)}
            />
          </label>
          <label className="field">
            <span>Name</span>
            <input
              value={name}
              placeholder="Acme Inc."
              onChange={(e) => setName(e.target.value)}
            />
          </label>
          {error ? <div className="notice error">{error}</div> : null}
          <button className="btn primary" type="submit" disabled={busy}>
            {busy ? "Creating…" : "Create tenant"}
          </button>
        </form>
      </div>
    </section>
  );
}
