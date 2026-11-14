import { Monitor, Moon, Sun } from "lucide-react";
import { useTheme, type ThemePreference } from "@/lib/theme";
import { cn } from "@/lib/utils";

const OPTIONS: { value: ThemePreference; label: string; icon: typeof Sun }[] = [
  { value: "light", label: "Light theme", icon: Sun },
  { value: "system", label: "Match system theme", icon: Monitor },
  { value: "dark", label: "Dark theme", icon: Moon },
];

export function ThemeToggle() {
  const { preference, setPreference } = useTheme();
  return (
    <div role="radiogroup" aria-label="Theme" className="flex rounded-md border border-border bg-surface p-0.5">
      {OPTIONS.map(({ value, label, icon: Icon }) => (
        <button
          key={value}
          type="button"
          role="radio"
          aria-checked={preference === value}
          aria-label={label}
          title={label}
          onClick={() => setPreference(value)}
          className={cn(
            "flex size-7 items-center justify-center rounded text-muted transition-colors hover:text-fg",
            preference === value && "bg-accent-soft text-fg",
          )}
        >
          <Icon aria-hidden="true" className="size-4" />
        </button>
      ))}
    </div>
  );
}
