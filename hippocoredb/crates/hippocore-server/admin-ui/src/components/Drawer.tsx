interface DrawerProps {
  title: string;
  value: unknown;
  onClose: () => void;
}

export function Drawer({ title, value, onClose }: DrawerProps) {
  return (
    <div className="drawer-overlay" onClick={onClose}>
      <aside
        className="drawer glass"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-label={title}
      >
        <header className="drawer-head">
          <strong>{title}</strong>
          <button className="btn ghost" onClick={onClose}>
            Close
          </button>
        </header>
        <pre className="json">{JSON.stringify(value, null, 2)}</pre>
      </aside>
    </div>
  );
}
