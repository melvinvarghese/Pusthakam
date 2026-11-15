import { useEffect, useId, useRef, type ReactNode } from "react";

interface Props {
  title: string;
  onClose: () => void;
  children: ReactNode;
  footer?: ReactNode;
  /** Wider dialog for long forms. */
  wide?: boolean;
}

const FOCUSABLE = 'input:not([disabled]), select:not([disabled]), textarea:not([disabled]), button:not([disabled]), a[href]';

/** Modal dialog: focus moves in, Tab stays inside, Escape or a backdrop click closes, focus returns on close. */
export function Modal({ title, onClose, children, footer, wide }: Props) {
  const titleId = useId();
  const box = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const previous = document.activeElement as HTMLElement | null;
    box.current?.querySelector<HTMLElement>(FOCUSABLE)?.focus();
    return () => previous?.focus();
  }, []);

  const onKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      e.preventDefault();
      onClose();
      return;
    }
    if (e.key !== "Tab" || !box.current) return;
    const items = [...box.current.querySelectorAll<HTMLElement>(FOCUSABLE)];
    const first = items[0];
    const last = items[items.length - 1];
    if (!first || !last) return;
    if (e.shiftKey && document.activeElement === first) {
      e.preventDefault();
      last.focus();
    } else if (!e.shiftKey && document.activeElement === last) {
      e.preventDefault();
      first.focus();
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center overflow-auto bg-ink/60 p-6 pt-[10vh]" onMouseDown={onClose}>
      <div
        ref={box}
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        className={`palette-in ${wide ? "w-[min(46rem,100%)]" : "w-[min(30rem,100%)]"} rounded-lg border border-line-strong bg-surface shadow-[0_16px_40px_-8px_rgb(13_20_32/0.45)]`}
        onMouseDown={(e) => e.stopPropagation()}
        onKeyDown={onKeyDown}
      >
        <h2 id={titleId} className="border-b border-line px-5 py-3 text-sm font-semibold">{title}</h2>
        <div className="space-y-4 p-5">{children}</div>
        {footer && <div className="flex justify-end gap-2 border-t border-line px-5 py-3">{footer}</div>}
      </div>
    </div>
  );
}
