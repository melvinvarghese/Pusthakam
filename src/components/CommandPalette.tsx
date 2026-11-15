import { useEffect, useMemo, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Search } from "lucide-react";
import { NAV_GROUPS } from "@/app/nav";
import { cn } from "@/lib/utils";

interface Props {
  open: boolean;
  onClose: () => void;
}

const ENTRIES = NAV_GROUPS.flatMap((g) => g.items.map((item) => ({ ...item, group: g.label })));

/** Ctrl/Cmd+K quick navigation. Record search joins this palette when the catalogue ships (Phase 3). */
export function CommandPalette({ open, onClose }: Props) {
  const navigate = useNavigate();
  const [query, setQuery] = useState("");
  const [active, setActive] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const results = useMemo(() => {
    const q = query.trim().toLowerCase();
    return q ? ENTRIES.filter((i) => i.label.toLowerCase().includes(q)) : ENTRIES;
  }, [query]);

  useEffect(() => {
    if (open) {
      setQuery("");
      setActive(0);
      inputRef.current?.focus();
    }
  }, [open]);

  if (!open) return null;

  const go = (path: string) => {
    navigate(path);
    onClose();
  };

  const onKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      e.preventDefault();
      onClose();
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      setActive((a) => Math.min(a + 1, results.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setActive((a) => Math.max(a - 1, 0));
    } else if (e.key === "Enter") {
      e.preventDefault();
      const item = results[active];
      if (item) go(item.path);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center bg-ink/60 pt-[14vh]" onMouseDown={onClose}>
      <div
        role="dialog"
        aria-modal="true"
        aria-label="Quick navigation"
        className="palette-in w-[min(32rem,90vw)] overflow-hidden rounded-lg border border-line-strong bg-surface shadow-[0_16px_40px_-8px_rgb(13_20_32/0.45)]"
        onMouseDown={(e) => e.stopPropagation()}
        onKeyDown={onKeyDown}
      >
        <div className="flex items-center gap-2 border-b border-line px-3">
          <Search aria-hidden="true" className="size-4 text-muted" strokeWidth={1.75} />
          <input
            ref={inputRef}
            value={query}
            onChange={(e) => { setQuery(e.target.value); setActive(0); }}
            placeholder="Go to a section"
            aria-label="Go to section"
            role="combobox"
            aria-expanded="true"
            aria-controls="palette-list"
            className="h-11 flex-1 bg-transparent text-sm outline-none placeholder:text-muted focus-visible:outline-none"
          />
        </div>
        <ul id="palette-list" role="listbox" className="max-h-72 overflow-auto p-1">
          {results.length === 0 && <li className="px-3 py-6 text-center text-muted">No matching sections</li>}
          {results.map((item, i) => (
            <li
              key={item.path}
              role="option"
              aria-selected={i === active}
              onMouseEnter={() => setActive(i)}
              onClick={() => go(item.path)}
              className={cn("flex cursor-pointer items-center gap-2 rounded-md px-3 py-2", i === active && "bg-band")}
            >
              <span className={cn(!item.available && "text-muted")}>{item.label}</span>
              <span className="ml-auto text-xs text-muted">{item.available ? item.group : `Phase ${item.phase}`}</span>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}
