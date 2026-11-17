import { useId, type InputHTMLAttributes, type ReactNode, type SelectHTMLAttributes } from "react";
import { cn } from "@/lib/utils";

const CONTROL =
  "h-9 w-full rounded-md border border-field-line bg-surface px-3 text-[13px] text-fg placeholder:text-muted disabled:opacity-60";

interface Shared {
  label: string;
  hint?: string;
  error?: string | null;
}

function Wrap({ id, label, hint, error, children }: Shared & { id: string; children: ReactNode }) {
  return (
    <div className="space-y-1">
      <label htmlFor={id} className="block text-[13px] font-medium">{label}</label>
      {children}
      {hint && !error && <p id={`${id}-hint`} className="text-xs text-muted">{hint}</p>}
      {error && <p id={`${id}-err`} role="alert" className="text-xs text-danger">{error}</p>}
    </div>
  );
}

function describedBy(id: string, hint?: string, error?: string | null) {
  return error ? `${id}-err` : hint ? `${id}-hint` : undefined;
}

export function TextField({ label, hint, error, className, ...rest }: Shared & InputHTMLAttributes<HTMLInputElement>) {
  const id = useId();
  return (
    <Wrap id={id} label={label} hint={hint} error={error}>
      <input
        id={id}
        aria-invalid={error ? true : undefined}
        aria-describedby={describedBy(id, hint, error)}
        className={cn(CONTROL, error && "border-danger", className)}
        {...rest}
      />
    </Wrap>
  );
}

export function SelectField({
  label, hint, error, className, children, ...rest
}: Shared & SelectHTMLAttributes<HTMLSelectElement>) {
  const id = useId();
  return (
    <Wrap id={id} label={label} hint={hint} error={error}>
      <select
        id={id}
        aria-invalid={error ? true : undefined}
        aria-describedby={describedBy(id, hint, error)}
        className={cn(CONTROL, error && "border-danger", className)}
        {...rest}
      >
        {children}
      </select>
    </Wrap>
  );
}

export function CheckField({
  label, hint, ...rest
}: { label: string; hint?: string } & Omit<InputHTMLAttributes<HTMLInputElement>, "type">) {
  const id = useId();
  return (
    <div className="flex items-start gap-2">
      <input id={id} type="checkbox" className="mt-0.5 size-4 accent-[var(--primary)]" {...rest} />
      <div>
        <label htmlFor={id} className="text-[13px] font-medium">{label}</label>
        {hint && <p className="text-xs text-muted">{hint}</p>}
      </div>
    </div>
  );
}
