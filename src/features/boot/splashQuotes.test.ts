import { createHash } from "node:crypto";
import sourceAudit from "../../../docs/splash-source-audit-2026-11-24.json";
import { afterEach, describe, expect, it, vi } from "vitest";
import {
  SPLASH_QUOTES,
  SPLASH_QUOTE_STORAGE_KEY,
  selectSplashQuote,
  selectAndRememberSplashQuote,
} from "./splashQuotes";

describe("splash quote catalogue", () => {
  afterEach(() => { vi.restoreAllMocks(); localStorage.clear(); });

  it("only ships reviewed entries with distinct text and auditable sources", () => {
    expect(SPLASH_QUOTES).toHaveLength(100);
    expect(new Set(SPLASH_QUOTES.map((quote) => quote.id)).size).toBe(SPLASH_QUOTES.length);
    expect(new Set(SPLASH_QUOTES.map((quote) => quote.text)).size).toBe(SPLASH_QUOTES.length);

    for (const quote of SPLASH_QUOTES) {
      expect(quote.text.length).toBeGreaterThan(10);
      expect(quote.author).not.toEqual("");
      expect(quote.work).not.toEqual("");
      expect(quote.sourceUrl).toMatch(/^https:\/\//);
      expect(quote.locator).not.toEqual("");
      expect(quote.edition).not.toEqual("");
      expect(quote.reviewNote).not.toEqual("");
      expect(quote.rightsBasis).not.toEqual("");
      expect(quote.reviewedAt).toMatch(/^\d{4}-\d{2}-\d{2}$/);
      expect(quote.text.split(/\s+/).length).toBeLessThanOrEqual(25);
    }
  });

  it("keeps the source audit synchronized without treating matches as image signoff", () => {
    expect(sourceAudit.entries).toHaveLength(SPLASH_QUOTES.length);
    expect(new Set(sourceAudit.entries.map(entry => entry.id)).size).toBe(SPLASH_QUOTES.length);
    for (const quote of SPLASH_QUOTES) {
      const audit = sourceAudit.entries.find(entry => entry.id === quote.id);
      expect(audit).toBeDefined();
      expect(audit?.textSha256).toBe(createHash("sha256").update(quote.text).digest("hex"));
      expect(audit && ("reviewedSourceUrl" in audit ? audit.reviewedSourceUrl : audit.sourceUrl)).toBe(quote.sourceUrl);
    }
    expect(SPLASH_QUOTES.find(quote => quote.id === "morley-world")?.text).toBe("there\u2019s all heaven and earth in a book, a real book I mean");
  });

  it("does not select the immediately previous quote when an alternative exists", () => {
    const previousId = SPLASH_QUOTES[0]!.id;
    const selected = selectSplashQuote(() => 0, previousId);

    expect(selected.id).not.toBe(previousId);
    expect(selected).toEqual(SPLASH_QUOTES[1]);
  });

  it("uses the deterministic selection index after excluding the previous quote", () => {
    expect(selectSplashQuote(() => 0.5).id).toBe(SPLASH_QUOTES[Math.floor(SPLASH_QUOTES.length / 2)]!.id);
  });

  it("remembers the selected id and excludes it on the next launch", () => {
    const first = selectAndRememberSplashQuote(() => 0);
    expect(localStorage.getItem(SPLASH_QUOTE_STORAGE_KEY)).toBe(first.id);
    expect(selectAndRememberSplashQuote(() => 0).id).not.toBe(first.id);
  });

  it("ignores an unknown previously stored id", () => {
    localStorage.setItem(SPLASH_QUOTE_STORAGE_KEY, "removed-candidate");
    expect(selectAndRememberSplashQuote(() => 0)).toEqual(SPLASH_QUOTES[0]);
  });

  it("survives blocked storage reads and writes", () => {
    vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => { throw new Error("blocked"); });
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => { throw new Error("full"); });
    expect(selectAndRememberSplashQuote(() => 0)).toEqual(SPLASH_QUOTES[0]);
  });

  it("survives an unavailable storage property", () => {
    vi.spyOn(window, "localStorage", "get").mockImplementation(() => { throw new Error("unavailable"); });
    expect(selectAndRememberSplashQuote(() => 0)).toEqual(SPLASH_QUOTES[0]);
  });

  it("reuses one remembered selection across StrictMode initializers and remounts", async () => {
    vi.resetModules();
    const { getLaunchSplashQuote } = await import("./splashQuotes");
    const random = vi.spyOn(Math, "random").mockReturnValue(0);
    const write = vi.spyOn(Storage.prototype, "setItem");
    const first = getLaunchSplashQuote();
    expect(getLaunchSplashQuote()).toBe(first);
    expect(getLaunchSplashQuote()).toBe(first);
    expect(random).toHaveBeenCalledTimes(1);
    expect(write).toHaveBeenCalledTimes(1);
    expect(localStorage.getItem(SPLASH_QUOTE_STORAGE_KEY)).toBe(first.id);
  });

  it("exports a stable local storage key", () => {
    expect(SPLASH_QUOTE_STORAGE_KEY).toBe("pusthakam.last-splash-quote-id");
  });
});
