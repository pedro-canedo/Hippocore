// Session token persistence. Shares the storage key with the legacy
// vanilla console so a login in either UI is recognised by the other.

const SESSION_KEY = "hippocore.adminSession";
const USER_KEY = "hippocore.adminUser";
const TENANT_KEY = "hippocore.tenant";

export function loadSession(): string {
  return localStorage.getItem(SESSION_KEY) ?? "";
}

export function loadUsername(): string {
  return localStorage.getItem(USER_KEY) ?? "admin";
}

export function saveSession(session: string, username: string): void {
  localStorage.setItem(SESSION_KEY, session);
  localStorage.setItem(USER_KEY, username);
}

export function clearSession(): void {
  localStorage.removeItem(SESSION_KEY);
}

export function loadActiveTenant(): string {
  return localStorage.getItem(TENANT_KEY) ?? "";
}

export function saveActiveTenant(tenantId: string): void {
  if (tenantId) localStorage.setItem(TENANT_KEY, tenantId);
  else localStorage.removeItem(TENANT_KEY);
}

// Active collection is remembered per tenant in a small JSON map.
const COLLECTION_MAP_KEY = "hippocore.console.collectionByTenant";

function readCollectionMap(): Record<string, string> {
  try {
    const raw = localStorage.getItem(COLLECTION_MAP_KEY);
    return raw ? (JSON.parse(raw) as Record<string, string>) : {};
  } catch {
    return {};
  }
}

export function loadActiveCollection(tenantId: string): string {
  if (!tenantId) return "";
  return readCollectionMap()[tenantId] ?? "";
}

export function saveActiveCollection(tenantId: string, collection: string): void {
  if (!tenantId) return;
  const map = readCollectionMap();
  if (collection) map[tenantId] = collection;
  else delete map[tenantId];
  localStorage.setItem(COLLECTION_MAP_KEY, JSON.stringify(map));
}
