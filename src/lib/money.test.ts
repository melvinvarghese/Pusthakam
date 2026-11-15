import { describe, expect, it } from "vitest";
import { formatMinor, minorToInput, parseMinor } from "./money";

describe("money", () => {
  it("parses plain amounts into minor units without floating point error", () => {
    expect(parseMinor("12")).toBe(1200);
    expect(parseMinor("12.5")).toBe(1250);
    expect(parseMinor("0.07")).toBe(7);
    expect(parseMinor(" 1.10 ")).toBe(110);
    expect(parseMinor("19.99")).toBe(1999);
  });

  it("rejects anything else", () => {
    for (const bad of ["", "abc", "-1", "1.234", "1,50", "1e3", ".5"]) expect(parseMinor(bad)).toBeNull();
  });

  it("formats for inputs and for display", () => {
    expect(minorToInput(1250)).toBe("12.50");
    expect(formatMinor(100, "INR")).toContain("1.00");
    expect(formatMinor(100, "NOPE1")).toBe("NOPE1 1.00");
  });
});
