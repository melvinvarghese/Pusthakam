import { describe, expect, it } from "vitest";
import { ALL_NAV_ITEMS, NAV_GROUPS } from "./nav";

describe("navigation config", () => {
  it("matches the product's four groups in order", () => {
    expect(NAV_GROUPS.map((g) => g.label)).toEqual(["Main", "Collections", "Operations", "Administration"]);
  });

  it("has unique paths and labels", () => {
    expect(new Set(ALL_NAV_ITEMS.map((i) => i.path)).size).toBe(ALL_NAV_ITEMS.length);
    expect(new Set(ALL_NAV_ITEMS.map((i) => i.label)).size).toBe(ALL_NAV_ITEMS.length);
  });

  it("only marks modules available when they are actually built", () => {
    expect(ALL_NAV_ITEMS.filter((i) => i.available).map((i) => i.label)).toEqual([
      "Dashboard", "Catalogue", "Circulation", "Members", "Accession Register", "Inventory", "Classification", "Periodicals", "Acquisitions", "Reports", "Barcode & Printing", "Staff & Permissions", "Library Settings", "Database Backups", "Audit Log",
    ]);
  });
});
