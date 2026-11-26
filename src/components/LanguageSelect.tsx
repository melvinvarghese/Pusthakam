import { useId } from "react";
import { setLanguage, type Language } from "@/lib/language";
import { useTranslation } from "@/lib/translations";

/** Per-device preference: changing it never mutates library records. */
export function LanguageSelect() {
  const id = useId();
  const { language, t } = useTranslation();
  return (
    <div className="space-y-1">
      <label htmlFor={id} className="block text-sm font-medium">Language / ഭാഷ</label>
      <select id={id} value={language} onChange={e => setLanguage(e.target.value as Language)}
        className="min-h-11 w-full rounded-md border border-field-line bg-surface px-3 text-base text-fg">
        <option value="en" lang="en">English</option>
        <option value="ml" lang="ml">മലയാളം</option>
      </select>
      <p className="text-xs text-muted">{t("Choose the language for this device.")}</p>
    </div>
  );
}
