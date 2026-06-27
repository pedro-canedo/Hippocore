import { useRef, useState, type DragEvent } from "react";

interface DropZoneProps {
  accept: string;
  disabled?: boolean;
  onFile: (file: File) => void;
}

const TYPES = [".pdf", ".txt", ".md", ".csv", ".json"];

export function DropZone({ accept, disabled = false, onFile }: DropZoneProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const [over, setOver] = useState(false);

  function handleDrop(event: DragEvent<HTMLDivElement>) {
    event.preventDefault();
    setOver(false);
    if (disabled) return;
    const file = event.dataTransfer.files?.[0];
    if (file) onFile(file);
  }

  function handleDragOver(event: DragEvent<HTMLDivElement>) {
    event.preventDefault();
    if (!disabled) setOver(true);
  }

  return (
    <div
      className={`drop-zone${over ? " over" : ""}${disabled ? " disabled" : ""}`}
      onDrop={handleDrop}
      onDragOver={handleDragOver}
      onDragLeave={() => setOver(false)}
      onClick={() => !disabled && inputRef.current?.click()}
      role="button"
      tabIndex={0}
    >
      <input
        ref={inputRef}
        type="file"
        accept={accept}
        hidden
        onChange={(e) => {
          const file = e.target.files?.[0];
          if (file) onFile(file);
          e.target.value = "";
        }}
      />
      <p className="drop-hint">
        {disabled
          ? "Select a collection first"
          : "Drop a file here, or click to browse"}
      </p>
      <div className="drop-types">
        {TYPES.map((t) => (
          <span className="file-type" key={t}>
            {t}
          </span>
        ))}
      </div>
    </div>
  );
}
