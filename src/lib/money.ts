/** Money is stored as integer minor units (paise, cents). These helpers convert for display and input. */

/** "12.50" or "12" -> 1250. Returns null for anything that is not a non-negative amount with at most 2 decimals. */
export function parseMinor(text: string): number | null {
  const t = text.trim();
  if (!/^\d+(\.\d{1,2})?$/.test(t)) return null;
  const [whole = "0", frac = ""] = t.split(".");
  return Number(whole) * 100 + Number(frac.padEnd(2, "0"));
}

/** 1250 -> "12.50" (plain, for text inputs). */
export function minorToInput(minor: number): string {
  return (minor / 100).toFixed(2);
}

/** 1250, "INR" -> "₹12.50" in the user's locale. Falls back to "INR 12.50" for unknown codes. */
export function formatMinor(minor: number, currency: string, locale = "en"): string {
  try {
    return new Intl.NumberFormat(locale, { style: "currency", currency }).format(minor / 100);
  } catch {
    return `${currency} ${minorToInput(minor)}`;
  }
}
