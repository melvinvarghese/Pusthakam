import type { HTMLAttributes, ReactNode } from "react";
import { cn } from "@/lib/utils";

export function Panel({ className, ...rest }: HTMLAttributes<HTMLElement>) {
  return <section className={cn("staff-panel min-w-0 rounded-lg border border-line bg-surface", className)} {...rest} />;
}

interface HeaderProps {
  id?: string;
  title: string;
  aside?: ReactNode;
}

/** Panel title row. The title is a real h2 so screen readers can jump between panels. */
export function PanelHeader({ id, title, aside }: HeaderProps) {
  return (
    <div className="flex items-center justify-between gap-3 border-b border-line px-4 py-3">
      <h2 id={id} className="text-sm font-semibold">{title}</h2>
      {aside && <div className="text-xs text-muted">{aside}</div>}
    </div>
  );
}
