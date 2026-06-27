// The set of pages the TypeScript console can render. Kept as a plain union so
// view switching needs no router library while the migration is in progress.
export type View = "dashboard" | "tenants" | "collections" | "explorer";

export const NAV_ITEMS: ReadonlyArray<{ view: View; label: string }> = [
  { view: "dashboard", label: "Dashboard" },
  { view: "tenants", label: "Tenants" },
  { view: "collections", label: "Collections" },
  { view: "explorer", label: "Data Explorer" },
];
