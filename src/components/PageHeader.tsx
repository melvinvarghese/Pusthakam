import type { ReactNode } from "react";

interface Props {
  title: string;
  description?: string;
  actions?: ReactNode;
}

export function PageHeader({ title, description, actions }: Props) {
  return (
    <header className="staff-page-heading flex flex-wrap items-end justify-between gap-3 sm:gap-6">
      <div className="min-w-0">
        <h1 className="text-xl font-bold leading-7 tracking-tight">{title}</h1>
        {description && <p className="text-muted">{description}</p>}
      </div>
      {actions && <div className="flex max-w-full flex-wrap items-center gap-2">{actions}</div>}
    </header>
  );
}
