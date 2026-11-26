import { useTranslation } from "@/lib/translations";
import { Monitor, Moon, Sun } from "lucide-react";
import { useTheme, type ThemePreference } from "@/lib/theme";
import { cn } from "@/lib/utils";

const OPTIONS: { value: ThemePreference; label: string; icon: typeof Sun }[] = [
  { value: "light", label: "Light theme", icon: Sun },
  { value: "system", label: "Match system theme", icon: Monitor },
  { value: "dark", label: "Dark theme", icon: Moon },
];
const ARROW_DIRECTION: Partial<Record<string, number>> = {
  ArrowLeft: -1, ArrowUp: -1, ArrowRight: 1, ArrowDown: 1,
};

/** Segmented control that sits on the ink top bar. */
export function ThemeToggle() {
  const { t } = useTranslation();
  const { preference, setPreference } = useTheme();
  return (
    <div role="radiogroup" aria-label={t("Theme")} className="flex rounded-md border border-ink-3 p-0.5">
      {OPTIONS.map(({ value, label, icon: Icon }, index) => (
        <button
          key={value}
          type="button"
          role="radio"
          aria-checked={preference === value}
          tabIndex={preference === value ? 0 : -1}
          aria-label={t(label)}
          title={t(label)}
          onClick={() => setPreference(value)}
          onKeyDown={(event) => {
            const direction = ARROW_DIRECTION[event.key];
            if (!direction) return;
            event.preventDefault();
            const next = (index + direction + OPTIONS.length) % OPTIONS.length;
            const option = OPTIONS[next];
            if (!option) return;
            setPreference(option.value);
            event.currentTarget.parentElement?.querySelectorAll<HTMLButtonElement>('[role="radio"]')[next]?.focus();
          }}
          className={cn(
            "flex size-7 items-center justify-center rounded text-rail-mute transition-colors hover:text-white",
            preference === value && "bg-ink-3 text-white",
          )}
        >
          <Icon aria-hidden="true" className="size-4" strokeWidth={1.75} />
        </button>
      ))}
    </div>
  );
}
