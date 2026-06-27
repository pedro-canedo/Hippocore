import { useState, type FormEvent } from "react";
import { ApiError, createCollection, type Collection } from "../api";

interface CollectionsViewProps {
  session: string;
  activeTenant: string;
  collections: Collection[];
  loading: boolean;
  onChanged: (message: string) => void;
}

export function CollectionsView({
  session,
  activeTenant,
  collections,
  loading,
  onChanged,
}: CollectionsViewProps) {
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);

  if (!activeTenant) {
    return (
      <section className="panel glass">
        <h2>Collections</h2>
        <p className="muted">
          Select a tenant in the top bar to view and create its collections.
        </p>
      </section>
    );
  }

  async function submit(event: FormEvent) {
    event.preventDefault();
    setError("");
    setBusy(true);
    try {
      const created = await createCollection(
        session,
        activeTenant,
        name.trim(),
        description.trim(),
      );
      setName("");
      setDescription("");
      onChanged(`Collection "${created.name}" created.`);
    } catch (err) {
      setError(err instanceof ApiError ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="cols">
      <div className="panel glass">
        <h2>
          Collections <span className="muted small">in {activeTenant}</span>
        </h2>
        {collections.length > 0 ? (
          <table className="table">
            <thead>
              <tr>
                <th>Name</th>
                <th>Description</th>
              </tr>
            </thead>
            <tbody>
              {collections.map((c) => (
                <tr key={c.name}>
                  <td>{c.name}</td>
                  <td className="muted">{c.description || "—"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        ) : (
          <p className="muted">
            {loading ? "Loading…" : "No collections yet in this tenant."}
          </p>
        )}
      </div>

      <div className="panel glass">
        <h2>New collection</h2>
        <form onSubmit={submit}>
          <label className="field">
            <span>Name</span>
            <input
              value={name}
              placeholder="support"
              onChange={(e) => setName(e.target.value)}
            />
          </label>
          <label className="field">
            <span>Description (optional)</span>
            <input
              value={description}
              placeholder="Support knowledge base"
              onChange={(e) => setDescription(e.target.value)}
            />
          </label>
          {error ? <div className="notice error">{error}</div> : null}
          <button className="btn primary" type="submit" disabled={busy}>
            {busy ? "Creating…" : "Create collection"}
          </button>
        </form>
      </div>
    </section>
  );
}
