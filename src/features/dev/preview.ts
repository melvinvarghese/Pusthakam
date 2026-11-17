/**
 * Development-only stand-in for the Rust backend, used by `?preview` (see src/lib/preview.ts).
 * Every record here is SYNTHETIC sample data for looking at screens; it is never bundled into production.
 */
import type { ConfigOverview, Session } from "@/lib/api/types";

const SESSION: Session = {
  userId: 1, username: "preview", displayName: "Rekha Nair", role: "administrator",
  permissions: ["admin.settings", "catalogue.read", "catalogue.write", "members.read"],
};

const SUMMARIES = [
  { id: 1, title: "Balyakalasakhi", subtitle: null, authors: "Vaikom Muhammad Basheer", publisher: "DC Books", publicationYear: 1944, isbn13: "9780306406157", classificationNumber: "894.8123", totalCopies: 3, availableCopies: 2, archived: false },
  { id: 2, title: "The God of Small Things", subtitle: null, authors: "Arundhati Roy", publisher: "IndiaInk", publicationYear: 1997, isbn13: null, classificationNumber: "823", totalCopies: 2, availableCopies: 0, archived: false },
  { id: 3, title: "Wings of Fire", subtitle: "An autobiography", authors: "A. P. J. Abdul Kalam", publisher: "Universities Press", publicationYear: 1999, isbn13: null, classificationNumber: "629.1092", totalCopies: 1, availableCopies: 1, archived: false },
  { id: 4, title: "A Brief History of Time", subtitle: null, authors: "Stephen Hawking", publisher: null, publicationYear: null, isbn13: null, classificationNumber: "523.1", totalCopies: 0, availableCopies: 0, archived: false },
];

const COPIES = [
  { id: 1, recordId: 1, accessionNo: "004217", barcode: "004217", itemCategoryId: 1, collectionId: 1, locationId: 1, shelfId: 1, callNumber: "894.8123 BAS", acquisitionDate: "2026-11-02", acquisitionSource: "purchase", priceMinor: 25000, vendor: null, invoiceRef: null, notes: null, status: "available" },
  { id: 2, recordId: 1, accessionNo: "004218", barcode: "004218", itemCategoryId: 1, collectionId: 1, locationId: 1, shelfId: 1, callNumber: "894.8123 BAS", acquisitionDate: "2026-11-02", acquisitionSource: "purchase", priceMinor: 25000, vendor: null, invoiceRef: null, notes: null, status: "on_loan" },
  { id: 3, recordId: 1, accessionNo: "004219", barcode: "004219", itemCategoryId: 1, collectionId: 1, locationId: 1, shelfId: 2, callNumber: "894.8123 BAS", acquisitionDate: "2026-11-02", acquisitionSource: "donation", priceMinor: null, vendor: "Friends of the Library", invoiceRef: null, notes: null, status: "damaged" },
];

const RECORD = {
  id: 1, title: "Balyakalasakhi", subtitle: null, edition: "5th", publisher: "DC Books", publicationPlace: "Kottayam", publicationYear: 1944,
  language: "Malayalam", physicalDescription: null, pagination: "120 p.", series: null, abstract: "A classic love story.", notes: null,
  isbn10: "0306406152", isbn13: "9780306406157", classificationSchemeId: 1, classificationNumber: "894.8123", keywords: "classic",
  archived: false, contributors: [{ name: "Vaikom Muhammad Basheer", role: "author" }], subjects: ["Love stories", "Malayalam fiction"], copies: COPIES,
};

const CONFIG: ConfigOverview = {
  library: { name: "Demo Library", profile: "school", address: "Kochi, Kerala", contact: null, currency: "INR", locale: "en" },
  classificationSchemes: [
    { id: 1, code: "ddc", name: "Dewey Decimal Classification" },
    { id: 2, code: "udc", name: "Universal Decimal Classification" },
    { id: 3, code: "lcc", name: "Library of Congress Classification" },
  ],
  memberCategories: [
    { id: 1, name: "Staff", description: null, membershipDays: 365, archived: false },
    { id: 2, name: "Student", description: "Classes 1 to 12", membershipDays: 365, archived: false },
    { id: 3, name: "Teacher", description: null, membershipDays: 365, archived: false },
  ],
  itemCategories: [
    { id: 1, name: "General", description: null, archived: false },
    { id: 2, name: "Reference", description: "Not lent out", archived: false },
  ],
  collections: [
    { id: 1, name: "General Collection", description: null, archived: false },
    { id: 2, name: "Reference Collection", description: null, archived: false },
    { id: 3, name: "Old Periodicals", description: null, archived: true },
  ],
  locations: [{ id: 1, name: "Main Library", description: null, archived: false }],
  shelves: [
    { id: 1, locationId: 1, code: "A1", description: "Fiction", archived: false },
    { id: 2, locationId: 1, code: "B3", description: null, archived: false },
  ],
  policies: [
    { id: 1, memberCategoryId: 1, itemCategoryId: 1, loanDays: 14, maxLoans: 3, maxRenewals: 1, renewalDays: 7, finePerDayMinor: 100, fineGraceDays: 1, fineCapMinor: null, reservationHoldDays: 3 },
    { id: 2, memberCategoryId: 2, itemCategoryId: 1, loanDays: 14, maxLoans: 3, maxRenewals: 1, renewalDays: 7, finePerDayMinor: 100, fineGraceDays: 1, fineCapMinor: null, reservationHoldDays: 3 },
    { id: 3, memberCategoryId: 3, itemCategoryId: 1, loanDays: 30, maxLoans: 10, maxRenewals: 2, renewalDays: 14, finePerDayMinor: 0, fineGraceDays: 0, fineCapMinor: null, reservationHoldDays: 3 },
  ],
};

export async function previewCall(command: string, mode: string, args?: Record<string, unknown>): Promise<unknown> {
  switch (command) {
    case "setup_status": return { configured: mode !== "setup", profiles: [] };
    case "current_session":
      if (mode === "login") return null;
      if (mode === "circulation") return { ...SESSION, permissions: [...SESSION.permissions, "circulation.issue"] };
      return SESSION;
    case "circulation_prepare_issue":
      if (args?.memberNumber !== "DEMO-001" || args?.barcode !== "004217") {
        throw { code: "NOT_FOUND", message: "For this preview, use member DEMO-001 and copy 004217." };
      }
      return { memberId: 1, itemId: COPIES[0]?.id, memberName: "Demo Reader", title: SUMMARIES[0]?.title, dueOn: "2026-12-01" };
    case "login": case "complete_setup": return SESSION;
    case "config_overview": return CONFIG;
    case "catalogue_search": return { total: SUMMARIES.length, page: 1, pageSize: 25, rows: SUMMARIES };
    case "catalogue_get_record": return RECORD;
    case "catalogue_find_duplicates": return [];
    case "catalogue_list_copies":
      return { total: COPIES.length, page: 1, pageSize: 25, rows: COPIES.map((c) => ({ ...c, title: "Balyakalasakhi", collection: "General Collection", location: "Main Library", shelf: c.shelfId === 1 ? "A1" : "B3" })) };
    case "import_preview":
      return {
        token: "preview", fileName: "koha-export.mrc", format: "marc", profile: "koha",
        totals: { records: 1240, copies: 1893, withWarnings: 37, withErrors: 2, duplicates: 12, copiesSkipped: 4 },
        newLookups: { locations: ["MAIN", "ANNEX"], collections: ["FICTION", "REF"], itemCategories: ["BK", "REF"] },
        sample: [
          { row: 1, title: "Balyakalasakhi", contributors: [{ name: "Basheer, Vaikom Muhammad", role: "author" }], isbn: "9780306406157", copies: [{ barcode: "004217" }, { barcode: "004218" }], warnings: [], errors: [], duplicateOf: null },
          { row: 2, title: "Wings of Fire", contributors: [{ name: "Kalam, A. P. J.", role: "author" }], isbn: null, copies: [{ barcode: "004219" }], warnings: ["Copy skipped: Barcode 004219 is already in the catalogue."], errors: [], duplicateOf: null },
          { row: 3, title: "", contributors: [], isbn: null, copies: [], warnings: [], errors: ["No title. This record cannot be imported."], duplicateOf: null },
        ],
        parseErrors: [], headers: null, mapping: null, mappingFields: null,
      };
    case "accession_format_get": return { prefix: "", padding: 6, next: 4220 };
    case "app_info":
      return { version: "0.1.0", schemaVersion: 11, dataDir: "C:\\Users\\Preview\\AppData\\Roaming\\app.pusthakam.desktop", databasePath: "C:\\Users\\Preview\\AppData\\Roaming\\app.pusthakam.desktop\\pusthakam.db" };
    case "dashboard_overview":
      return {
        catalogue: { titles: SUMMARIES.length, copies: COPIES.length, available: COPIES.filter((copy) => copy.status === "available").length },
        activeMembers: 0, circulation: null,
      };
    default: return null; // saves and archives succeed without changing anything
  }
}
