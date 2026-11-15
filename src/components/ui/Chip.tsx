import type { ReactNode } from "react";
import { cn } from "@/lib/utils";

type Tone = "neutral" | "overdue" | "fine";

const TONES: Record<Tone, string> = {
  neutral: "border-line-strong text-fg",
  overdue: "border-[1.5px] border-danger bg-surface font-bold text-danger",
  fine: "border-warn/40 bg-warn-bg font-semibold text-warn",
};

/** Status chip. Red and amber are reserved for overdue and fines; never decorative. */
export function Chip({ tone = "neutral", children }: { tone?: Tone; children: ReactNode }) {
  return (
    <span className={cn("inline-block whitespace-nowrap rounded border px-2 text-xs font-medium leading-5", TONES[tone])}>
      {children}
    </span>
  );
}
