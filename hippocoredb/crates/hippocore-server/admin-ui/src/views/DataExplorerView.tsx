import {
  useCallback,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import {
  listDocuments,
  listFiles,
  listMemories,
  listRecords,
  type Collection,
  type DocumentRow,
  type FileRow,
  type MemoryRow,
  type RecordRow,
} from "../api";
import { loadActiveCollection, saveActiveCollection } from "../session";
import { Drawer } from "../components/Drawer";

type Kind = "records" | "memories" | "documents" | "files";

const KINDS: ReadonlyArray<{ kind: Kind; label: string }> = [
  { kind: "records", label: "Records" },
  { kind: "memories", label: "Memories" },
  { kind: "documents", label: "Documents" },
  { kind: "files", label: "Files" },
];

interface DataExplorerViewProps {
  session: string;
  activeTenant: string;
  collections: Collection[];
}

interface Loaded {
  records: RecordRow[];
  memories: MemoryRow[];
  documents: DocumentRow[];
  files: FileRow[];
}

const EMPTY: Loaded = { records: [], memories: [], documents: [], files: [] };

function truncate(text: string, max = 80): string {
  return text.length > max ? `${text.slice(0, max)}…` : text;
}

function recordColumns(rows: RecordRow[]): string[] {
  const keys = new Set<string>();
  for (const row of rows) {
    if (row.payload && typeof row.payload === "object") {
      for (const k of Object.keys(row.payload)) keys.add(k);
    }
  }
  return Array.from(keys).sort();
}

function cell(value: unknown): string {
  if (value === null || value === undefined) return "—";
  if (typeof value === "object") return JSON.stringify(value);
  return String(value);
}

export function DataExplorerView({
  session,
  activeTenant,
  collections,
}: DataExplorerViewProps) {
  const [collection, setCollection] = useState<string>(() =>
    loadActiveCollection(activeTenant),
  );
  const [tab, setTab] = useState<Kind>("records");
  const [data, setData] = useState<Loaded>(EMPTY);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [detail, setDetail] = useState<{ title: string; value: unknown } | null>(
    null,
  );

  // Reset the chosen collection when the tenant changes.
  useEffect(() => {
    setCollection(loadActiveCollection(activeTenant));
    setData(EMPTY);
  }, [activeTenant]);

  const load = useCallback(async () => {
    if (!activeTenant) return;
    setLoading(true);
    setError("");
    try {
      const [records, memories, documents, files] = await Promise.all([
        listRecords(session, activeTenant, collection),
        listMemories(session, activeTenant, collection),
        listDocuments(session, activeTenant, collection),
        listFiles(session, activeTenant, collection),
      ]);
      setData({ records, memories, documents, files });
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, [session, activeTenant, collection]);

  useEffect(() => {
    void load();
  }, [load]);

  const counts = useMemo<Record<Kind, number>>(
    () => ({
      records: data.records.length,
      memories: data.memories.length,
      documents: data.documents.length,
      files: data.files.length,
    }),
    [data],
  );

  if (!activeTenant) {
    return (
      <section className="panel glass">
        <h2>Data Explorer</h2>
        <p className="muted">Select a tenant in the top bar to browse its data.</p>
      </section>
    );
  }

  function onSelectCollection(value: string) {
    setCollection(value);
    saveActiveCollection(activeTenant, value);
  }

  const recordCols = recordColumns(data.records);

  return (
    <>
      <section className="panel glass">
        <div className="explorer-head">
          <h2>Data Explorer</h2>
          <label className="tenant-select">
            <span className="muted small">Collection</span>
            <select
              value={collection}
              onChange={(e) => onSelectCollection(e.target.value)}
            >
              <option value="">all collections</option>
              {collections.map((c) => (
                <option key={c.name} value={c.name}>
                  {c.name}
                </option>
              ))}
            </select>
          </label>
        </div>

        <div className="tabs">
          {KINDS.map(({ kind, label }) => (
            <button
              key={kind}
              className={`tab${tab === kind ? " active" : ""}`}
              onClick={() => setTab(kind)}
            >
              {label} <span className="tab-count">{counts[kind]}</span>
            </button>
          ))}
        </div>

        {error ? <div className="notice error">{error}</div> : null}
        {loading ? <p className="muted">Loading…</p> : null}

        {!loading && tab === "records" && (
          <div className="table-scroll">
            {data.records.length === 0 ? (
              <p className="muted">No records in this scope.</p>
            ) : (
              <table className="table">
                <thead>
                  <tr>
                    <th>id</th>
                    <th>table</th>
                    {recordCols.map((c) => (
                      <th key={c}>{c}</th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {data.records.map((r) => (
                    <tr
                      key={`${r.table}/${r.id}`}
                      onClick={() =>
                        setDetail({ title: `record ${r.id}`, value: r })
                      }
                    >
                      <td className="mono">{r.id}</td>
                      <td>{r.table}</td>
                      {recordCols.map((c) => (
                        <td key={c}>{cell(r.payload?.[c])}</td>
                      ))}
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </div>
        )}

        {!loading && tab === "memories" && (
          <EntityTable
            rows={data.memories}
            empty="No memories in this scope."
            columns={["id", "type", "text", "confidence"]}
            render={(m) => [
              <span className="mono" key="id">
                {m.id}
              </span>,
              m.memory_type,
              truncate(m.text),
              m.confidence ?? "—",
            ]}
            onOpen={(m) => setDetail({ title: `memory ${m.id}`, value: m })}
            rowKey={(m) => m.id}
          />
        )}

        {!loading && tab === "documents" && (
          <EntityTable
            rows={data.documents}
            empty="No documents in this scope."
            columns={["id", "text", "version"]}
            render={(d) => [
              <span className="mono" key="id">
                {d.id}
              </span>,
              truncate(d.text),
              d.version,
            ]}
            onOpen={(d) => setDetail({ title: `document ${d.id}`, value: d })}
            rowKey={(d) => d.id}
          />
        )}

        {!loading && tab === "files" && (
          <EntityTable
            rows={data.files}
            empty="No files in this scope."
            columns={["id", "name", "media type", "bytes"]}
            render={(f) => [
              <span className="mono" key="id">
                {f.id}
              </span>,
              f.name,
              f.media_type,
              f.size_bytes,
            ]}
            onOpen={(f) => setDetail({ title: `file ${f.id}`, value: f })}
            rowKey={(f) => f.id}
          />
        )}
      </section>

      {detail ? (
        <Drawer
          title={detail.title}
          value={detail.value}
          onClose={() => setDetail(null)}
        />
      ) : null}
    </>
  );
}

interface EntityTableProps<T> {
  rows: T[];
  empty: string;
  columns: string[];
  render: (row: T) => ReactNode[];
  onOpen: (row: T) => void;
  rowKey: (row: T) => string;
}

function EntityTable<T>({
  rows,
  empty,
  columns,
  render,
  onOpen,
  rowKey,
}: EntityTableProps<T>) {
  if (rows.length === 0) return <p className="muted">{empty}</p>;
  return (
    <div className="table-scroll">
      <table className="table">
        <thead>
          <tr>
            {columns.map((c) => (
              <th key={c}>{c}</th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => (
            <tr key={rowKey(row)} onClick={() => onOpen(row)}>
              {render(row).map((value, i) => (
                <td key={columns[i] ?? i}>{value}</td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
