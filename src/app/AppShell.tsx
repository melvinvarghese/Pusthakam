import { useEffect, useState } from "react";
import { NavLink, Outlet } from "react-router-dom";
import { Search } from "lucide-react";
import { NAV_GROUPS } from "./nav";
import { CommandPalette } from "@/components/CommandPalette";
import { ThemeToggle } from "@/components/ThemeToggle";
import { ErrorBoundary } from "@/components/ErrorBoundary";
import { cn } from "@/lib/utils";

function Brand() {
  return (
    <div className="px-4 pb-4 pt-5">
      <div className="text-[15px] font-semibold tracking-[0.18em]">PUSTHAKAM</div>
      {/* Malayalam brand word: optional, falls back to OS fonts; the app never depends on it. */}
      <div lang="ml" className="text-sm text-muted">പുസ്തകം</div>
    </div>
  );
}

export function AppShell() {
  const [paletteOpen, setPaletteOpen] = useState(false);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setPaletteOpen((o) => !o);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  return (
    <div className="flex h-full">
      <a href="#main" className="sr-only focus:not-sr-only focus:absolute focus:left-2 focus:top-2 focus:z-50 focus:rounded focus:bg-surface focus:px-3 focus:py-1">
        Skip to content
      </a>
      <aside className="flex w-60 shrink-0 flex-col border-r border-border bg-sidebar">
        <Brand />
        <nav aria-label="Primary" className="flex-1 overflow-y-auto px-2 pb-4">
          {NAV_GROUPS.map((group) => (
            <div key={group.label} className="mb-4">
              <div className="px-2 pb-1 text-[11px] font-medium uppercase tracking-wider text-muted">{group.label}</div>
              <ul className="space-y-0.5">
                {group.items.map((item) => (
                  <li key={item.path}>
                    <NavLink
                      to={item.path}
                      end={item.path === "/"}
                      className={({ isActive }) =>
                        cn(
                          "flex items-center gap-2.5 rounded px-2 py-1.5 text-[13px] text-fg/85 hover:bg-surface-2",
                          isActive && "bg-accent-soft font-medium text-fg",
                          !item.available && "text-muted",
                        )
                      }
                    >
                      <item.icon aria-hidden="true" className="size-4 shrink-0" />
                      <span className="truncate">{item.label}</span>
                    </NavLink>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </nav>
        <div className="border-t border-border px-4 py-3 text-xs text-muted">A Better Way to Manage Libraries.</div>
      </aside>

      <div className="flex min-w-0 flex-1 flex-col">
        <header className="flex h-12 shrink-0 items-center gap-3 border-b border-border bg-surface px-4">
          <button
            type="button"
            onClick={() => setPaletteOpen(true)}
            className="flex h-8 w-72 items-center gap-2 rounded-md border border-border bg-bg px-2.5 text-muted hover:text-fg"
          >
            <Search aria-hidden="true" className="size-4" />
            <span className="flex-1 text-left">Search or go to…</span>
            <kbd className="rounded border border-border px-1 text-[11px]">Ctrl K</kbd>
          </button>
          <div className="ml-auto"><ThemeToggle /></div>
        </header>
        <main id="main" className="min-h-0 flex-1 overflow-auto">
          <ErrorBoundary>
            <Outlet />
          </ErrorBoundary>
        </main>
      </div>
      <CommandPalette open={paletteOpen} onClose={() => setPaletteOpen(false)} />
    </div>
  );
}
