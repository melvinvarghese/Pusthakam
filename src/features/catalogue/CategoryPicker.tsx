import { useTranslation } from "@/lib/translations";
import type { BrowseCategory } from "@/lib/api/types";
import { cn } from "@/lib/utils";

interface Props {
  categories: BrowseCategory[];
  selected: number[];
  onChange: (ids: number[]) => void;
  hint: string;
}

/** Toggle chips for the member-facing browse categories shown in the public catalogue. */
export function CategoryPicker({ categories, selected, onChange, hint }: Props) {
  const { t } = useTranslation();
  const toggle = (id: number) =>
    onChange(selected.includes(id) ? selected.filter((x) => x !== id) : [...selected, id]);
  return (
    <fieldset className="space-y-1.5">
      <legend className="text-[13px] font-medium">{t("Categories in the public catalogue")}</legend>
      {categories.length === 0 ? (
        <p className="text-xs text-muted">{t("No categories yet. Add them under Classification, Browse categories.")}</p>
      ) : (
        <div className="flex flex-wrap gap-1.5">
          {categories.map((c) => {
            const on = selected.includes(c.id);
            return (
              <button
                key={c.id}
                type="button"
                aria-pressed={on}
                onClick={() => toggle(c.id)}
                className={cn(
                  "h-7 rounded-full border px-3 text-xs transition-colors",
                  on ? "border-primary bg-primary font-semibold text-primary-fg" : "border-line-strong bg-surface text-fg hover:bg-band",
                )}
              >
                {c.name}
              </button>
            );
          })}
        </div>
      )}
      <p className="text-xs text-muted">{hint}</p>
    </fieldset>
  );
}
