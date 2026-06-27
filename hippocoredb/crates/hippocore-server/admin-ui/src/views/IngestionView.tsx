import { useState, type FormEvent } from "react";
import {
  ApiError,
  createDocument,
  createMemory,
  createRecord,
  type Collection,
} from "../api";
import { loadActiveCollection, saveActiveCollection } from "../session";

interface IngestionViewProps {
  session: string;
  activeTenant: string;
  collections: Collection[];
  onChanged: (message: string) => void;
}

export function IngestionView({
  session,
  activeTenant,
  collections,
  onChanged,
}: IngestionViewProps) {
  const [collection, setCollection] = useState<string>(() =>
    loadActiveCollection(activeTenant),
  );

  if (!activeTenant) {
    return (
      <section className="panel glass">
        <h2>Ingestion</h2>
        <p className="muted">Select a tenant in the top bar to add data.</p>
      </section>
    );
  }

  function onSelectCollection(value: string) {
    setCollection(value);
    saveActiveCollection(activeTenant, value);
  }

  const ready = collection.length > 0;

  return (
    <>
      <section className="panel glass">
        <div className="explorer-head">
          <h2>Ingestion</h2>
          <label className="tenant-select">
            <span className="muted small">Collection</span>
            <select
              value={collection}
              onChange={(e) => onSelectCollection(e.target.value)}
            >
              <option value="">— select —</option>
              {collections.map((c) => (
                <option key={c.name} value={c.name}>
                  {c.name}
                </option>
              ))}
            </select>
          </label>
        </div>
        {!ready ? (
          <p className="muted">
            Choose a target collection to enable the forms. Create one on the
            Collections page first if needed.
          </p>
        ) : null}
      </section>

      {ready ? (
        <section className="cols-3">
          <MemoryForm
            session={session}
            tenant={activeTenant}
            collection={collection}
            onChanged={onChanged}
          />
          <RecordForm
            session={session}
            tenant={activeTenant}
            collection={collection}
            onChanged={onChanged}
          />
          <DocumentForm
            session={session}
            tenant={activeTenant}
            collection={collection}
            onChanged={onChanged}
          />
        </section>
      ) : null}
    </>
  );
}

interface FormProps {
  session: string;
  tenant: string;
  collection: string;
  onChanged: (message: string) => void;
}

function useSubmit(run: () => Promise<string>, reset: () => void) {
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  return {
    error,
    busy,
    async submit(event: FormEvent, onChanged: (m: string) => void) {
      event.preventDefault();
      setError("");
      setBusy(true);
      try {
        const message = await run();
        reset();
        onChanged(message);
      } catch (err) {
        setError(err instanceof ApiError ? err.message : String(err));
      } finally {
        setBusy(false);
      }
    },
  };
}

function MemoryForm({ session, tenant, collection, onChanged }: FormProps) {
  const [text, setText] = useState("");
  const [type, setType] = useState("semantic");
  const { error, busy, submit } = useSubmit(
    async () => {
      const m = await createMemory(session, tenant, {
        collection,
        text: text.trim(),
        memory_type: type,
      });
      return `Memory ${m.id} created.`;
    },
    () => setText(""),
  );

  return (
    <form className="panel glass" onSubmit={(e) => submit(e, onChanged)}>
      <h3>Memory</h3>
      <label className="field">
        <span>Text</span>
        <textarea
          rows={4}
          value={text}
          onChange={(e) => setText(e.target.value)}
        />
      </label>
      <label className="field">
        <span>Type</span>
        <select value={type} onChange={(e) => setType(e.target.value)}>
          <option value="semantic">semantic</option>
          <option value="episodic">episodic</option>
          <option value="procedural">procedural</option>
        </select>
      </label>
      {error ? <div className="notice error">{error}</div> : null}
      <button className="btn primary" type="submit" disabled={busy || !text.trim()}>
        {busy ? "Saving…" : "Add memory"}
      </button>
    </form>
  );
}

function RecordForm({ session, tenant, collection, onChanged }: FormProps) {
  const [table, setTable] = useState("");
  const [payload, setPayload] = useState('{\n  "key": "value"\n}');
  const [jsonError, setJsonError] = useState("");
  const { error, busy, submit } = useSubmit(
    async () => {
      let parsed: unknown;
      try {
        parsed = JSON.parse(payload);
      } catch {
        throw new ApiError(0, "Payload is not valid JSON.");
      }
      const r = await createRecord(session, tenant, {
        collection,
        table: table.trim(),
        payload: parsed,
      });
      return `Record ${r.id} created in ${r.table}.`;
    },
    () => undefined,
  );

  function validate(value: string) {
    setPayload(value);
    try {
      JSON.parse(value);
      setJsonError("");
    } catch {
      setJsonError("Invalid JSON");
    }
  }

  return (
    <form className="panel glass" onSubmit={(e) => submit(e, onChanged)}>
      <h3>Record</h3>
      <label className="field">
        <span>Table</span>
        <input
          value={table}
          placeholder="systems"
          onChange={(e) => setTable(e.target.value)}
        />
      </label>
      <label className="field">
        <span>Payload (JSON)</span>
        <textarea
          className="mono-input"
          rows={5}
          value={payload}
          spellCheck={false}
          onChange={(e) => validate(e.target.value)}
        />
      </label>
      {jsonError ? <div className="notice error">{jsonError}</div> : null}
      {error ? <div className="notice error">{error}</div> : null}
      <button
        className="btn primary"
        type="submit"
        disabled={busy || !table.trim() || jsonError.length > 0}
      >
        {busy ? "Saving…" : "Add record"}
      </button>
    </form>
  );
}

function DocumentForm({ session, tenant, collection, onChanged }: FormProps) {
  const [text, setText] = useState("");
  const { error, busy, submit } = useSubmit(
    async () => {
      const d = await createDocument(session, tenant, {
        collection,
        text: text.trim(),
      });
      return `Document ${d.id} created (${d.chunk_count} chunks).`;
    },
    () => setText(""),
  );

  return (
    <form className="panel glass" onSubmit={(e) => submit(e, onChanged)}>
      <h3>Document</h3>
      <label className="field">
        <span>Text</span>
        <textarea
          rows={6}
          value={text}
          onChange={(e) => setText(e.target.value)}
        />
      </label>
      {error ? <div className="notice error">{error}</div> : null}
      <button className="btn primary" type="submit" disabled={busy || !text.trim()}>
        {busy ? "Saving…" : "Add document"}
      </button>
    </form>
  );
}
