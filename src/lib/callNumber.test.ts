import { describe, expect, it } from "vitest";
import { parseCallNumber, suggestCallNumber } from "./callNumber";

describe("suggestCallNumber", () => {
  it("adds three letters of the author's surname", () => {
    expect(suggestCallNumber("894.8123", "Vaikom Muhammad Basheer")).toBe("894.8123 BAS");
    expect(suggestCallNumber("823", "Roy, Arundhati")).toBe("823 ROY");
    expect(suggestCallNumber("629.1092", "A. P. J. Abdul Kalam")).toBe("629.1092 KAL");
  });

  it("falls back gracefully", () => {
    expect(suggestCallNumber("500", null)).toBe("500");
    expect(suggestCallNumber("", "Someone")).toBe("");
    expect(suggestCallNumber(null, null)).toBe("");
    expect(suggestCallNumber("100", "12345")).toBe("100");
  });
});

describe("parseCallNumber", () => {
  it("stacks a full Dewey number and finds the main class", () => {
    expect(parseCallNumber("894.8123 BAS")).toEqual({ lines: ["894", ".8123", "BAS"], deweyClass: 8 });
    expect(parseCallNumber("629.1092 KAL")).toEqual({ lines: ["629", ".1092", "KAL"], deweyClass: 6 });
  });

  it("handles a class with no decimal part", () => {
    expect(parseCallNumber("823 ROY")).toEqual({ lines: ["823", "ROY"], deweyClass: 8 });
    expect(parseCallNumber("004")).toEqual({ lines: ["004"], deweyClass: 0 });
  });

  it("keeps non-Dewey schemes neutral", () => {
    expect(parseCallNumber("PR9199.3 .R69 2001")).toEqual({ lines: ["PR9199.3", ".R69", "2001"], deweyClass: null });
  });

  it("joins extra tokens onto the third line", () => {
    expect(parseCallNumber("A B C D").lines).toEqual(["A", "B", "C D"]);
  });

  it("normalises whitespace and handles empty input", () => {
    expect(parseCallNumber("  894.8123   BAS ").lines).toEqual(["894", ".8123", "BAS"]);
    expect(parseCallNumber("   ")).toEqual({ lines: [], deweyClass: null });
  });

  it("does not treat short numbers as Dewey classes", () => {
    expect(parseCallNumber("89 X").deweyClass).toBeNull();
  });
});
