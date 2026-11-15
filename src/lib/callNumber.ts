/**
 * Splits a call number into the stacked lines of a spine label and finds its Dewey main class.
 * Pure and offline. Non-Dewey schemes (LC, UDC, custom) still get a stacked label with a neutral band.
 */
export interface ParsedCallNumber {
  /** Up to three lines, top to bottom, as printed on the label. */
  lines: string[];
  /** Main class 0-9 when the number starts with a three-digit Dewey class, otherwise null. */
  deweyClass: number | null;
}

const DEWEY = /^(\d{3})(\.\d+)?(?:\s+(.*))?$/;

export function parseCallNumber(raw: string): ParsedCallNumber {
  const text = raw.trim().replace(/\s+/g, " ");
  if (!text) return { lines: [], deweyClass: null };

  const m = DEWEY.exec(text);
  if (m) {
    const [, cls, decimal, rest] = m;
    const lines = [cls ?? "", ...(decimal ? [decimal] : []), ...(rest ? [rest] : [])];
    return { lines: clampLines(lines), deweyClass: Number((cls ?? "0")[0]) };
  }
  return { lines: clampLines(text.split(" ")), deweyClass: null };
}

/** More than three tokens: the remainder is joined onto the last line. */
function clampLines(lines: string[]): string[] {
  if (lines.length <= 3) return lines;
  return [lines[0] ?? "", lines[1] ?? "", lines.slice(2).join(" ")];
}

/**
 * Suggests a call number for a new copy: the classification number plus the first three letters of the
 * first author's surname, e.g. "894.8123" + "Vaikom Muhammad Basheer" -> "894.8123 BAS". Only a suggestion; staff can edit it.
 */
export function suggestCallNumber(classification: string | null, firstAuthor: string | null): string {
  const cls = (classification ?? "").trim();
  if (!cls) return "";
  const name = (firstAuthor ?? "").trim();
  if (!name) return cls;
  // "Surname, Given" puts the surname first; otherwise the surname is the last word.
  const surname = name.includes(",") ? name.split(",")[0]!.trim() : (name.split(/\s+/).pop() ?? "");
  const letters = surname.replace(/[^\p{L}]/gu, "").slice(0, 3).toUpperCase();
  return letters ? `${cls} ${letters}` : cls;
}
