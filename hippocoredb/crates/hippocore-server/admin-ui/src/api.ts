// Typed client for the Hippocore admin API.
//
// All admin endpoints (except /admin/login and /health) require the
// `x-admin-session` header obtained from a successful login.

export interface LoginResponse {
  session: string;
  username: string;
}

export interface AdminConfig {
  api_key_set: boolean;
  api_key_length: number;
  data_dir: string;
  admin_user_set: boolean;
}

export interface BootstrapStats {
  tenants: number;
  collections: number;
  documents: number;
  chunks: number;
  memories: number;
  records: number;
  files: number;
  graph_edges: number;
  indexed_entries: number;
  wal_entries: number;
  disk_bytes: number;
  audit_records: number;
  audit_log_bytes: number;
}

export interface Tenant {
  id: string;
  name: string;
  [key: string]: unknown;
}

export interface Collection {
  name: string;
  tenant_id: string;
  description: string;
  [key: string]: unknown;
}

export interface Bootstrap {
  config: AdminConfig;
  stats: BootstrapStats;
  tenants: Tenant[];
}

export interface RecordRow {
  id: string;
  table: string;
  collection: string;
  payload: Record<string, unknown>;
  created_at: number;
  updated_at: number;
  version: number;
  [key: string]: unknown;
}

export interface MemoryRow {
  id: string;
  collection: string;
  memory_type: string;
  text: string;
  confidence: number | null;
  created_at: number;
  [key: string]: unknown;
}

export interface DocumentRow {
  id: string;
  collection: string;
  text: string;
  version: number;
  created_at: number;
  updated_at: number;
  [key: string]: unknown;
}

export interface FileRow {
  id: string;
  collection: string;
  name: string;
  media_type: string;
  size_bytes: number;
  created_at: number;
  [key: string]: unknown;
}

export interface SqlResult {
  command: string;
  row_count: number;
  rows: RecordRow[];
}

export class ApiError extends Error {
  readonly status: number;
  constructor(status: number, message: string) {
    super(message);
    this.status = status;
    this.name = "ApiError";
  }
}

async function parseError(res: Response): Promise<never> {
  let message = `HTTP ${res.status}`;
  try {
    const body = (await res.json()) as { error?: string };
    if (body && typeof body.error === "string") message = body.error;
  } catch {
    // non-JSON error body; keep the status message
  }
  throw new ApiError(res.status, message);
}

export async function health(): Promise<boolean> {
  try {
    const res = await fetch("/health");
    return res.ok;
  } catch {
    return false;
  }
}

export async function login(
  username: string,
  password: string,
): Promise<LoginResponse> {
  const res = await fetch("/admin/login", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ username, password }),
  });
  if (!res.ok) return parseError(res);
  return (await res.json()) as LoginResponse;
}

export async function bootstrap(session: string): Promise<Bootstrap> {
  const res = await fetch("/admin/bootstrap", {
    headers: { "x-admin-session": session },
  });
  if (!res.ok) return parseError(res);
  return (await res.json()) as Bootstrap;
}

function authHeaders(session: string): HeadersInit {
  return { "content-type": "application/json", "x-admin-session": session };
}

export async function listTenants(session: string): Promise<Tenant[]> {
  const res = await fetch("/admin/tenants", {
    headers: { "x-admin-session": session },
  });
  if (!res.ok) return parseError(res);
  return (await res.json()) as Tenant[];
}

export async function createTenant(
  session: string,
  id: string,
  name: string,
): Promise<Tenant> {
  const res = await fetch("/admin/tenants", {
    method: "POST",
    headers: authHeaders(session),
    body: JSON.stringify({ id, name }),
  });
  if (!res.ok) return parseError(res);
  return (await res.json()) as Tenant;
}

export async function listCollections(
  session: string,
  tenantId: string,
): Promise<Collection[]> {
  const params = new URLSearchParams({ tenant_id: tenantId });
  const res = await fetch(`/admin/collections?${params.toString()}`, {
    headers: { "x-admin-session": session },
  });
  if (!res.ok) return parseError(res);
  return (await res.json()) as Collection[];
}

export async function createCollection(
  session: string,
  tenantId: string,
  name: string,
  description: string,
): Promise<Collection> {
  const res = await fetch(
    `/admin/tenants/${encodeURIComponent(tenantId)}/collections`,
    {
      method: "POST",
      headers: authHeaders(session),
      body: JSON.stringify({ name, description }),
    },
  );
  if (!res.ok) return parseError(res);
  return (await res.json()) as Collection;
}

async function listEntities<T>(
  session: string,
  path: string,
  tenantId: string,
  collection: string,
): Promise<T[]> {
  const params = new URLSearchParams({ tenant_id: tenantId });
  if (collection) params.set("collection", collection);
  const res = await fetch(`${path}?${params.toString()}`, {
    headers: { "x-admin-session": session },
  });
  if (!res.ok) return parseError(res);
  return (await res.json()) as T[];
}

export function listRecords(
  session: string,
  tenantId: string,
  collection: string,
): Promise<RecordRow[]> {
  return listEntities<RecordRow>(session, "/admin/records", tenantId, collection);
}

export function listMemories(
  session: string,
  tenantId: string,
  collection: string,
): Promise<MemoryRow[]> {
  return listEntities<MemoryRow>(
    session,
    "/admin/memories",
    tenantId,
    collection,
  );
}

export function listDocuments(
  session: string,
  tenantId: string,
  collection: string,
): Promise<DocumentRow[]> {
  return listEntities<DocumentRow>(
    session,
    "/admin/documents",
    tenantId,
    collection,
  );
}

export function listFiles(
  session: string,
  tenantId: string,
  collection: string,
): Promise<FileRow[]> {
  return listEntities<FileRow>(session, "/admin/files", tenantId, collection);
}

export async function runSql(
  session: string,
  tenantId: string,
  sql: string,
): Promise<SqlResult> {
  const res = await fetch("/admin/sql", {
    method: "POST",
    headers: authHeaders(session),
    body: JSON.stringify({ tenant_id: tenantId, sql }),
  });
  if (!res.ok) return parseError(res);
  return (await res.json()) as SqlResult;
}
