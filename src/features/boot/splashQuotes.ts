import quoteCatalogue from "./splashQuoteCatalogue.json";

/** Only source-reviewed entries belong in this runtime catalogue.
 * Unchecked candidates are archived in docs/splash-quote-candidates.json.
 * Source access limitations and edition notes are recorded per entry.
 */
export interface SplashQuote {
  id: string;
  text: string;
  author: string;
  work: string;
  sourceUrl: string;
  locator: string;
  edition: string;
  reviewNote: string;
  reviewedAt: string;
  rightsBasis: string;
  speaker?: string;
}

export const SPLASH_QUOTE_STORAGE_KEY = "pusthakam.last-splash-quote-id";

export const SPLASH_QUOTES: readonly SplashQuote[] = quoteCatalogue;

/** Picks one quote while excluding a valid immediately previous id. */
export function selectSplashQuote(random: () => number = Math.random, previousId: string | null = null): SplashQuote {
  const choices = previousId && SPLASH_QUOTES.length > 1
    ? SPLASH_QUOTES.filter((entry) => entry.id !== previousId)
    : SPLASH_QUOTES;
  return choices[Math.min(Math.floor(random() * choices.length), choices.length - 1)]!;
}

function getStorage(): Storage | null {
  try {
    return typeof window === "undefined" ? null : window.localStorage;
  } catch {
    return null;
  }
}

/** Selects and remembers a quote without allowing storage access to break startup. */
export function selectAndRememberSplashQuote(random: () => number = Math.random): SplashQuote {
  const storage = getStorage();
  let previousId: string | null = null;
  try {
    previousId = storage?.getItem(SPLASH_QUOTE_STORAGE_KEY) ?? null;
  } catch {
    // Privacy-restricted environments may reject storage access; startup remains functional.
  }
  const selected = selectSplashQuote(random, previousId);
  try {
    storage?.setItem(SPLASH_QUOTE_STORAGE_KEY, selected.id);
  } catch {
    // The quote is still displayed even if it cannot be remembered.
  }
  return selected;
}

let launchQuote: SplashQuote | undefined;

/** A webview reload starts a new launch; React checks/remounts reuse its selection. */
export function getLaunchSplashQuote(): SplashQuote {
  launchQuote ??= selectAndRememberSplashQuote();
  return launchQuote;
}
