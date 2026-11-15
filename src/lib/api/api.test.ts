import { describe, expect, it } from "vitest";
import { ApiError, appApi, isDesktop, openExternal, toApiError } from ".";

describe("api layer", () => {
  it("is not desktop in jsdom and fails with a clear NOT_DESKTOP error", async () => {
    expect(isDesktop()).toBe(false);
    await expect(appApi.info()).rejects.toMatchObject({ code: "NOT_DESKTOP" });
  });

  it("maps backend error payloads", () => {
    const e = toApiError({ code: "CONFLICT", message: "Accession number already exists." });
    expect(e).toBeInstanceOf(ApiError);
    expect(e.code).toBe("CONFLICT");
    expect(e.message).toBe("Accession number already exists.");
  });

  it("only opens secure web addresses", async () => {
    for (const bad of ["http://example.com", "file:///C:/secret.txt", "javascript:alert(1)", "mailto:a@b.c"]) {
      await expect(openExternal(bad)).rejects.toMatchObject({ code: "VALIDATION" });
    }
  });

  it("maps strings and unknown values without leaking internals", () => {
    expect(toApiError("boom").code).toBe("UNKNOWN");
    expect(toApiError(42).message).toBe("An unexpected error occurred.");
  });
});
