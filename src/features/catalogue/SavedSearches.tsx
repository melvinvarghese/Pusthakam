import { useTranslation } from "@/lib/translations";
import { useState } from "react";
import { Button } from "@/components/ui/Button";
import { TextField } from "@/components/ui/Field";
import type { SearchQuery } from "@/lib/api/types";

type Filter = Pick<SearchQuery, "text" | "collectionId" | "onlyAvailable" | "includeArchived" | "sort">;
interface Saved { name: string; filter: Filter }

function hasCriteria(filter: Filter) {
  return Boolean(filter.text?.trim() || filter.collectionId || filter.onlyAvailable || filter.includeArchived
    || (filter.sort && filter.sort !== "relevance"));
}

function read(key: string): Saved[] {
  try {
    const items: unknown = JSON.parse(localStorage.getItem(key) ?? "[]");
    if (!Array.isArray(items)) return [];
    return items.filter((item): item is Saved => {
      const f = item?.filter;
      return typeof item?.name === "string" && item.name.trim() && f
        && typeof f.text === "string" && typeof f.onlyAvailable === "boolean"
        && typeof f.includeArchived === "boolean"
        && (f.collectionId === null || (Number.isSafeInteger(f.collectionId) && f.collectionId > 0))
        && ["relevance", "title", "year", "recent"].includes(f.sort);
    });
  } catch { return []; }
}

export function SavedSearches({ userId, filter, onApply }: {
  userId: number; filter: Filter; onApply: (value: Filter) => void;
}) {
  const { t } = useTranslation();
  const key = `pusthakam.saved-searches.${userId}`;
  const [items, setItems] = useState(() => read(key));
  const [name, setName] = useState("");
  const [error, setError] = useState("");
  const selected = items.find((item) => item.name === name.trim());
  function persist(next: Saved[]) {
    try {
      localStorage.setItem(key, JSON.stringify(next));
      setItems(next);
      setError("");
      return true;
    } catch { setError("Could not save searches on this computer."); return false; }
  }
  function save(replace: boolean) {
    const label = name.trim();
    if (!label) { setError("Enter a search name."); return; }
    if (!hasCriteria(filter)) {
      setError("Nothing to save yet. Enter words in Search the catalogue or choose a filter first. The saved search name is only a label.");
      return;
    }
    const existing = items.find((item) => item.name.toLowerCase() === label.toLowerCase());
    if (existing && !replace) { setError("That name already exists. Use Update saved search to replace it."); return; }
    persist([...items.filter((item) => item !== existing), { name: label, filter }]);
  }
  return <div className="space-y-2 border-b border-line p-3">
    <div className="flex flex-wrap items-end gap-2">
      <label>{t("Saved searches")}<select aria-label={t("Saved searches")} className="ml-2 h-9 rounded border border-field-line bg-surface px-2" value={selected?.name ?? ""}
          onChange={(event) => {
            const item = items.find((item) => item.name === event.target.value);
            if (item) { setName(item.name); onApply(item.filter); setError(""); }
          }}>
          <option value="">{t("Choose a saved search")}</option>
          {items.map((item) => <option key={item.name}>{item.name}</option>)}
        </select>
      </label>
      <TextField label={t("Search name")} hint={t("A label for the search above, not search words.")} value={name} maxLength={80} onChange={(event) => setName(event.target.value)} />
      <Button onClick={() => save(false)}>{t("Save search")}</Button>
      {selected && <>
        <Button onClick={() => { onApply(selected.filter); setError(""); }}>{t("Run saved search")}</Button>
        <Button onClick={() => save(true)}>{t("Update saved search")}</Button>
        <Button onClick={() => { persist(items.filter((item) => item !== selected)); setName(""); }}>{t("Remove saved search")}</Button>
      </>}
    </div>
    {selected && !hasCriteria(selected.filter) && <div className="space-y-2">
      <p className="text-sm">{t("This entry saved all titles, without search words or filters.")}</p>
      <Button onClick={() => {
        const repaired = { ...selected, filter: { ...selected.filter, text: selected.name } };
        if (persist(items.map((item) => item === selected ? repaired : item))) onApply(repaired.filter);
      }}>{t("Use saved name as search words")}</Button>
    </div>}
    {selected && <p className="text-sm">{t("Saved search words:")} <strong>{selected.filter.text?.trim() || t("All titles (no search words saved)")}</strong></p>}
    <p className="text-xs text-muted">{t("Enter words and filters in the catalogue search above, then save them with a name. Choose a saved search to run it, or use Run saved search to run it again. Saved for your staff account on this computer.")}</p>
    {error && <p role="alert" className="text-danger">{t(error)}</p>}
  </div>;
}
