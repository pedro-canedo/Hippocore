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

export interface Bootstrap {
  config: AdminConfig;
  stats: BootstrapStats;
  tenants: Tenant[];
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
