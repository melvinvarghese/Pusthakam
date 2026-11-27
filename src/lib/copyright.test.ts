import { describe, expect, it } from "vitest";
import { formatCopyrightNotice, supportUrl } from "./copyright";

describe("formatCopyrightNotice", () => {
  it("shows 2026 in the initial release year", () => {
    expect(formatCopyrightNotice(2026)).toBe("© 2026 Melvin Varghese");
  });

  it("shows a 2026–year range in future years", () => {
    expect(formatCopyrightNotice(2028)).toBe("© 2026–2028 Melvin Varghese");
    expect(formatCopyrightNotice(2035)).toBe("© 2026–2035 Melvin Varghese");
  });
});

describe("supportUrl", () => {
  it("includes static UTM attribution for app and OPAC placements", () => {
    const appUrl = new URL(supportUrl("app_sidebar"));
    expect(appUrl.origin + appUrl.pathname).toBe("https://melvinvarghese.in/support/");
    expect(appUrl.searchParams.get("utm_source")).toBe("pusthakam");
    expect(appUrl.searchParams.get("utm_medium")).toBe("app");
    expect(appUrl.searchParams.get("utm_campaign")).toBe("copyright_footer");
    expect(appUrl.searchParams.get("utm_content")).toBe("app_sidebar");

    const opacUrl = new URL(supportUrl("opac_footer"));
    expect(opacUrl.searchParams.get("utm_medium")).toBe("opac");
    expect(opacUrl.searchParams.get("utm_content")).toBe("opac_footer");
  });
});
