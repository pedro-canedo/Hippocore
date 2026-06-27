// Session token persistence. Shares the storage key with the legacy
// vanilla console so a login in either UI is recognised by the other.

const SESSION_KEY = "hippocore.adminSession";
const USER_KEY = "hippocore.adminUser";

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
