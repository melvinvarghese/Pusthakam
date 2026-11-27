import type { BrowseCategory } from "@/lib/api/types";

/**
 * Predicts which browse categories the backend's DDC rules will give a record, so the record form
 * can show them before saving. Mirrors `browse_categories.rs`: a rule is a DDC prefix and only DDC
 * (or unspecified-scheme) class numbers are matched.
 */
export function predictCategoryIds(
  classNumber: string,
  schemeCode: string | null | undefined,
  categories: BrowseCategory[],
): number[] {
  const number = classNumber.trim();
  if (!number || (schemeCode && schemeCode !== "ddc")) return [];
  return categories
    .filter((c) => !c.archived && (c.ddcRules ?? "").split(",").some((rule) => rule && number.startsWith(rule)))
    .map((c) => c.id);
}
