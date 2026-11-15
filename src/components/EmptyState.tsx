import type { LucideIcon } from "lucide-react";
import type { ReactNode } from "react";

interface Props {
  icon?: LucideIcon;
  title: string;
  description?: ReactNode;
  action?: ReactNode;
}

export function EmptyState({ icon: Icon, title, description, action }: Props) {
  return (
    <div className="mx-auto flex max-w-md flex-col items-center gap-3 px-6 py-16 text-center">
      {Icon && (
        <span className="flex size-10 items-center justify-center rounded-md border border-line bg-band text-muted">
          <Icon aria-hidden="true" className="size-5" strokeWidth={1.75} />
        </span>
      )}
      <h2 className="text-base font-semibold">{title}</h2>
      {description && <p className="text-muted">{description}</p>}
      {action}
    </div>
  );
}
