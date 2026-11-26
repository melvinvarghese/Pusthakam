import { useTranslation } from "@/lib/translations";
import { useEffect, useRef, useState } from "react";
import { ChevronUp, ExternalLink } from "lucide-react";
import { ONLINE_TOOLS, toolUrl } from "@/features/online-tools/tools";
import { ApiError, openExternal } from "@/lib/api";
import { cn } from "@/lib/utils";

/** Rail footer item that opens a small list of optional web tools. Closes on Escape or an outside click. */
export function OnlineToolsMenu() {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const root = useRef<HTMLDivElement>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  const firstLink = useRef<HTMLAnchorElement>(null);

  useEffect(() => {
    if (!open) return;
    firstLink.current?.focus();
    const onDown = (e: MouseEvent) => {
      if (root.current && !root.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", onDown);
    return () => document.removeEventListener("mousedown", onDown);
  }, [open]);

  const close = () => {
    setOpen(false);
    setError(null);
    trigger.current?.focus();
  };

  const launch = async (e: React.MouseEvent, url: string) => {
    e.preventDefault();
    setError(null);
    try {
      await openExternal(url);
      setOpen(false);
    } catch (err) {
      setError(err instanceof ApiError ? err.message : "Could not open the link.");
    }
  };

  return (
    <div
      ref={root}
      className="relative border-t border-ink-3 p-2"
      onKeyDown={(e) => {
        if (e.key === "Escape" && open) {
          e.preventDefault();
          close();
        }
      }}
    >
      <button
        ref={trigger}
        type="button"
        aria-haspopup="dialog"
        aria-expanded={open}
        onClick={() => (open ? close() : setOpen(true))}
        className={cn(
          "flex h-8 w-full items-center justify-between rounded-md px-3 text-rail-text transition-colors hover:bg-ink-2 hover:text-white",
          open && "bg-ink-2 text-white",
        )}
      >
        <span>{t("Online Tools")}</span>
        <ChevronUp aria-hidden="true" className={cn("size-4 transition-transform", !open && "rotate-180")} strokeWidth={1.75} />
      </button>

      {open && (
        <div
          role="dialog"
          aria-label={t("Online Tools")}
          className="palette-in absolute bottom-full left-2 z-40 mb-1 w-[340px] rounded-lg border border-line-strong bg-surface p-1 text-fg shadow-[0_16px_40px_-8px_rgb(13_20_32/0.45)]"
        >
          <div className="px-3 pb-1 pt-2">
            <h2 className="text-sm font-semibold">{t("Online Tools")}</h2>
            <p className="text-xs text-muted">{t("Open in your web browser. These need an internet connection.")}</p>
          </div>
          <ul>
            {ONLINE_TOOLS.map((tool, i) => (
              <li key={tool.id}>
                <a
                  ref={i === 0 ? firstLink : undefined}
                  href={toolUrl(tool)}
                  target="_blank"
                  rel="noopener noreferrer"
                  onClick={(e) => launch(e, toolUrl(tool))}
                  className="block rounded-md px-3 py-2 hover:bg-band"
                >
                  <span className="flex items-center justify-between gap-2 font-medium">
                    {tool.name}
                    <ExternalLink aria-hidden="true" className="size-3.5 text-muted" strokeWidth={1.75} />
                  </span>
                  <span className="block text-xs text-muted">{tool.description}</span>
                </a>
              </li>
            ))}
          </ul>
          {error && <p role="alert" className="px-3 pb-2 text-xs text-danger">{t(error)}</p>}
        </div>
      )}
    </div>
  );
}
