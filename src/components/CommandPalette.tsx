import { useEffect, useMemo, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Search } from "lucide-react";
import { ALL_NAV_ITEMS } from "@/app/nav";
import { cn } from "@/lib/utils";

interface Props {
  open: boolean;
  onClose: () => void;
}

/** Ctrl/Cmd+K quick navigation. Data search joins this palette when the catalogue ships (Phase 3). */
export function CommandPalette({ open, onClose }: Props) {
  const navigate = useNavigate();
  const [query, setQuery] = useState("");
  const [active, setActive] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const results = useMemo(() => {
    const q = query.trim().toLowerCase();
    return q ? ALL_NAV_ITEMS.filter((i) => i.label.toLowerCase().includes(q)) : ALL_NAV_ITEMS;
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
    <div className="fixed inset-0 z-50 flex items-start justify-center bg-black/40 pt-[15vh]" onMouseDown={onClose}>
      <div
        role="dialog"
        aria-modal="true"
        aria-label="Quick navigation"
        className="w-[min(32rem,90vw)] overflow-hidden rounded-md border border-border bg-surface shadow-xl"
        onMouseDown={(e) => e.stopPropagation()}
        onKeyDown={onKeyDown}
      >
        <div className="flex items-center gap-2 border-b border-border px-3">
          <Search aria-hidden="true" className="size-4 text-muted" />
          <input
            ref={inputRef}
            value={query}
            onChange={(e) => { setQuery(e.target.value); setActive(0); }}
            placeholder="Go to…"
            aria-label="Go to section"
            role="combobox"
            aria-expanded="true"
            aria-controls="palette-list"
            className="h-11 flex-1 bg-transparent outline-none placeholder:text-muted"
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
              className={cn("flex cursor-pointer items-center gap-2 rounded px-3 py-2", i === active && "bg-accent-soft")}
            >
              <item.icon aria-hidden="true" className="size-4 text-muted" />
              <span>{item.label}</span>
              {!item.available && <span className="ml-auto text-xs text-muted">Phase {item.phase}</span>}
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}
