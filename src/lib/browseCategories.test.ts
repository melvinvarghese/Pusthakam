import { describe, expect, it } from "vitest";
import type { BrowseCategory } from "@/lib/api/types";
import { predictCategoryIds } from "./browseCategories";

const cat = (id: number, name: string, ddcRules: string | null, archived = false): BrowseCategory => ({
  id, name, ddcRules, archived, sortOrder: id, isDefault: true, recordCount: 0,
});
const CATS = [
  cat(1, "Malayalam Fiction", "894.8123"),
  cat(2, "Physics", "53"),
  cat(3, "History", "90,93,94"),
  cat(4, "Children's Books", null),
  cat(5, "Old", "53", true),
];

describe("predictCategoryIds", () => {
  it("matches DDC prefixes", () => {
    expect(predictCategoryIds("894.8123 BAS", "ddc", CATS)).toEqual([1]);
    expect(predictCategoryIds(" 530.1", "ddc", CATS)).toEqual([2]);
    expect(predictCategoryIds("954.83", null, CATS)).toEqual([]);
    expect(predictCategoryIds("940", undefined, CATS)).toEqual([3]);
  });
  it("ignores other schemes, blanks and archived categories", () => {
    expect(predictCategoryIds("530", "udc", CATS)).toEqual([]);
    expect(predictCategoryIds("  ", "ddc", CATS)).toEqual([]);
  });
});
