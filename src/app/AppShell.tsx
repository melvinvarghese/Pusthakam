import { useTranslation } from "@/lib/translations";
import { useEffect, useRef, useState } from "react";
import { NavLink, Outlet } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { configApi } from "@/lib/api";
import { useSession } from "@/lib/session";
import { Menu, Search } from "lucide-react";
import { Modal } from "@/components/ui/Modal";
import { useVisibleNavGroups } from "./nav";
import { BrandMark } from "@/components/BrandMark";
import { CommandPalette } from "@/components/CommandPalette";
import { OnlineToolsMenu } from "@/components/OnlineToolsMenu";
import { ThemeToggle } from "@/components/ThemeToggle";
import { ErrorBoundary } from "@/components/ErrorBoundary";
import { LiveAnnouncer } from "@/components/LiveAnnouncer";
import { cn } from "@/lib/utils";

function Brand({ libraryName }: { libraryName?: string }) {
  return (
    <div className="flex h-14 shrink-0 items-center gap-3 border-b border-ink-3 px-4">
      <BrandMark />
      <div className="min-w-0">
        <div className="text-[15px] font-bold leading-[18px] text-white">Pusthakam</div>
        <div className="truncate text-xs leading-4 text-rail-mute">{libraryName ?? " "}</div>
      </div>
    </div>
  );
}

function NavRail({ libraryName, mobile, inactive, onNavigate }: { libraryName?: string; mobile?: boolean; inactive?: boolean; onNavigate?: () => void }) {
  const { t } = useTranslation();
  const groups = useVisibleNavGroups();
  return (
    <aside inert={inactive} className={cn("staff-sidebar row-span-2 flex min-h-0 flex-col bg-ink text-rail-text", mobile && "staff-mobile-rail")}>
      <Brand libraryName={libraryName} />
      <nav aria-label={t("Primary")} className="rail-scroll min-h-0 flex-1 overflow-y-auto px-2 pb-3">
        {groups.map((group) => (
          <div key={group.label}>
            <div className="px-2 pb-1 pt-4 text-[11px] font-semibold uppercase leading-4 tracking-[0.08em] text-rail-mute">
              {t(group.label)}
            </div>
            <ul>
              {group.items.map((item) => (
                <li key={item.path}>
                  <NavLink
                    onClick={onNavigate}
                    to={item.path}
                    end={item.path === "/"}
                    title={item.available ? undefined : t("Not available in this version yet")}
                    className={({ isActive }) =>
                      cn(
                        "block min-h-8 rounded-md px-3 py-1.5 leading-5 transition-colors hover:bg-ink-2 hover:text-white",
                        item.available ? "text-rail-text" : "text-rail-mute",
                        isActive && "bg-ink-2 font-semibold text-white",
                      )
                    }
                  >
                    {t(item.label)}
                  </NavLink>
                </li>
              ))}
            </ul>
          </div>
        ))}
      </nav>
      <OnlineToolsMenu />
    </aside>
  );
}

export function AppShell() {
  const { t } = useTranslation();
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const menuButton = useRef<HTMLButtonElement>(null);
  const restoreMenuFocus = useRef(false);
  const closeMobileMenu = () => {
    restoreMenuFocus.current = true;
    setMobileMenuOpen(false);
  };
  useEffect(() => {
    if (!mobileMenuOpen && restoreMenuFocus.current) {
      menuButton.current?.focus();
      restoreMenuFocus.current = false;
    }
  }, [mobileMenuOpen]);
  const { session, signOut } = useSession();
  const config = useQuery({ queryKey: ["config"], queryFn: configApi.overview, retry: false });

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setMobileMenuOpen(false);
        setPaletteOpen((o) => !o);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  return (
    <>
    <div className="staff-shell grid h-full grid-cols-[232px_minmax(0,1fr)] grid-rows-[56px_minmax(0,1fr)]">
      <a
        href="#main"
        className="sr-only focus:not-sr-only focus:absolute focus:left-2 focus:top-2 focus:z-50 focus:rounded-md focus:bg-surface focus:px-3 focus:py-1"
      >
        {t("Skip to content")}
      </a>
      <LiveAnnouncer />
      <NavRail libraryName={config.data?.library?.name} inactive={mobileMenuOpen} />
      <header inert={mobileMenuOpen} className="staff-topbar flex min-w-0 items-center gap-4 border-b border-ink-3 bg-ink px-6">
        <button ref={menuButton} type="button" className="staff-menu-toggle" aria-label={t("Open navigation")} aria-haspopup="dialog" aria-expanded={mobileMenuOpen} onClick={() => setMobileMenuOpen(true)}>
          <Menu aria-hidden="true" className="size-5" />
        </button>
        <button
          type="button"
          aria-label={t("Search or go to a section")}
          onClick={() => setPaletteOpen(true)}
          className="staff-search flex h-9 w-[min(540px,100%)] items-center gap-2 rounded-md border border-ink-3 bg-ink-2 px-2.5 text-rail-field transition-colors hover:border-rail-line"
        >
          <Search aria-hidden="true" className="size-4" strokeWidth={1.75} />
          <span className="flex-1 text-left">{t("Search or go to a section")}</span>
          <span className="staff-search-short">{t("Search")}</span>
          <kbd className="rounded border border-rail-line px-1.5 font-sans text-[11px] leading-[18px] text-rail-text">Ctrl K</kbd>
        </button>
        <div className="staff-account ml-auto flex items-center gap-4">
          <ThemeToggle />
          <div className="flex items-center gap-3 text-white">
            <div className="staff-account-name text-right leading-4">
              <div className="font-medium">{session.displayName}</div>
              <div className="text-xs capitalize text-rail-mute">{t(session.role)}</div>
            </div>
            <button
              type="button"
              onClick={() => void signOut()}
              className="h-8 rounded-md border border-ink-3 px-3 text-rail-text transition-colors hover:border-rail-line hover:text-white"
            >
              {t("Sign out")}
            </button>
          </div>
        </div>
      </header>
      <main id="main" inert={mobileMenuOpen} className="staff-workspace min-h-0 min-w-0 overflow-auto bg-bg">
        <ErrorBoundary>
          <Outlet />
        </ErrorBoundary>
      </main>
      <CommandPalette open={paletteOpen} onClose={() => setPaletteOpen(false)} />
    </div>
    {mobileMenuOpen && <Modal title={t("Navigation")} onClose={closeMobileMenu} footer={<button type="button" className="min-h-11 rounded border border-line-strong px-4" onClick={closeMobileMenu}>{t("Close navigation")}</button>}>
      <p className="text-sm text-muted">{session.displayName} · {t(session.role)}</p>
      <NavRail mobile libraryName={config.data?.library?.name} onNavigate={closeMobileMenu} />
    </Modal>}
    </>
  );
}
