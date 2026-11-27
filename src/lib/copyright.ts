const START_YEAR = 2026;
const SUPPORT_BASE_URL = "https://melvinvarghese.in/support/";

export type CopyrightPlacement = "app_sidebar" | "app_auth" | "opac_footer";

/** Formats the runtime copyright year span (e.g. "2026" or "2026–2028"). */
export function formatCopyrightSpan(year: number = new Date().getFullYear()): string {
  return year > START_YEAR ? `${START_YEAR}–${year}` : `${START_YEAR}`;
}

/** Formats the runtime copyright text using the local clock. */
export function formatCopyrightNotice(year: number = new Date().getFullYear()): string {
  return `© ${formatCopyrightSpan(year)} Melvin Varghese`;
}

/** Builds the support link with static UTM campaign attribution (never sends library data). */
export function supportUrl(placement: CopyrightPlacement): string {
  const url = new URL(SUPPORT_BASE_URL);
  url.searchParams.set("utm_source", "pusthakam");
  url.searchParams.set("utm_medium", placement === "opac_footer" ? "opac" : "app");
  url.searchParams.set("utm_campaign", "copyright_footer");
  url.searchParams.set("utm_content", placement);
  return url.toString();
}
