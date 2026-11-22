import { describe, expect, it } from "vitest";
import {
  SPLASH_QUOTES,
  SPLASH_QUOTE_STORAGE_KEY,
  selectSplashQuote,
} from "./splashQuotes";

describe("splash quote catalogue", () => {
  it("contains exactly 113 distinct, attributed quotations", () => {
    expect(SPLASH_QUOTES).toHaveLength(113);
    expect(new Set(SPLASH_QUOTES.map((quote) => quote.id)).size).toBe(113);
    expect(new Set(SPLASH_QUOTES.map((quote) => quote.text)).size).toBe(113);

    for (const quote of SPLASH_QUOTES) {
      expect(quote.text.length).toBeGreaterThan(10);
      expect(quote.author).not.toEqual("");
      expect(quote.work).not.toEqual("");
      expect(quote.sourceUrl).toMatch(/^https:\/\//);
    }
  });

  it("does not select the immediately previous quote when an alternative exists", () => {
    const previousId = SPLASH_QUOTES[0]!.id;
    const selected = selectSplashQuote(() => 0, previousId);

    expect(selected.id).not.toBe(previousId);
    expect(selected).toEqual(SPLASH_QUOTES[1]);
  });

  it("uses the deterministic selection index after excluding the previous quote", () => {
    expect(selectSplashQuote(() => 0.5).id).toBe(SPLASH_QUOTES[56]!.id);
  });

  it("exports a stable local storage key", () => {
    expect(SPLASH_QUOTE_STORAGE_KEY).toBe("pusthakam.last-splash-quote-id");
  });
});
