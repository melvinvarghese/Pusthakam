import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import type { ConfigOverview, CopyQuery, CopyRecord, CopyRow, RecordDetail, Session } from "@/lib/api/types";

const cat = vi.hoisted(() => ({
  search: vi.fn(), getRecord: vi.fn(), saveRecord: vi.fn(), findDuplicates: vi.fn(), archiveRecord: vi.fn(),
  addCopies: vi.fn(), updateCopy: vi.fn(), setCopyStatus: vi.fn(), listCopies: vi.fn(), accessionFormat: vi.fn(),
  setAccessionFormat: vi.fn(), exportCopies: vi.fn(),
}));
const cfg = vi.hoisted(() => ({ overview: vi.fn() }));
vi.mock("@/lib/api", async (orig) => ({ ...(await orig<typeof import("@/lib/api")>()), catalogueApi: cat, configApi: cfg }));

import { ApiError } from "@/lib/api";
import { SessionContext } from "@/lib/session";
import { AccessionPage } from "./AccessionPage";
import { CataloguePage } from "./CataloguePage";
import { RecordPage } from "./RecordPage";

const CONFIG: ConfigOverview = {
  library: { name: "Demo", profile: "school", address: null, contact: null, currency: "INR", locale: "en" },
  classificationSchemes: [{ id: 1, code: "ddc", name: "Dewey Decimal Classification" }],
  memberCategories: [],
  itemCategories: [{ id: 5, name: "General", description: null, archived: false }],
  collections: [{ id: 7, name: "Fiction", description: null, archived: false }],
  locations: [{ id: 3, name: "Main Library", description: null, archived: false }],
  shelves: [{ id: 9, locationId: 3, code: "A1", description: null, archived: false }],
  policies: [],
};

const COPY: CopyRecord = {
  id: 1, recordId: 1, accessionNo: "004217", barcode: "004217", itemCategoryId: 5, collectionId: 7, locationId: 3, shelfId: 9,
  callNumber: "894.8123 BAS", acquisitionDate: "2026-11-02", acquisitionSource: "purchase", priceMinor: 25000, vendor: null,
  invoiceRef: null, notes: null, status: "available",
};

const RECORD: RecordDetail = {
  id: 1, title: "Balyakalasakhi", subtitle: null, edition: null, publisher: "DC Books", publicationPlace: null, publicationYear: 1944,
  language: null, physicalDescription: null, pagination: null, series: null, abstract: null, notes: null, isbn10: null,
  isbn13: "9780306406157", classificationSchemeId: 1, classificationNumber: "894.8123", keywords: null, archived: false,
  contributors: [{ name: "Vaikom Muhammad Basheer", role: "author" }], subjects: ["Malayalam fiction"], copies: [COPY],
};

const SUMMARY = {
  id: 1, title: "Balyakalasakhi", subtitle: null, authors: "Vaikom Muhammad Basheer", publisher: "DC Books", publicationYear: 1944,
  isbn13: "9780306406157", classificationNumber: "894.8123", totalCopies: 3, availableCopies: 2, archived: false,
};

const ADMIN: Session = { userId: 1, username: "a", displayName: "A", role: "librarian", permissions: ["catalogue.read", "catalogue.write"] };
const CLERK: Session = { userId: 2, username: "c", displayName: "C", role: "circulation", permissions: ["catalogue.read"] };

function renderAt(path: string, session: Session = ADMIN) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <QueryClientProvider client={qc}>
      <SessionContext.Provider value={{ session, signOut: async () => {} }}>
        <MemoryRouter initialEntries={[path]}>
          <Routes>
            <Route path="/catalogue" element={<CataloguePage />} />
            <Route path="/catalogue/:id" element={<RecordPage />} />
            <Route path="/accession" element={<AccessionPage />} />
          </Routes>
        </MemoryRouter>
      </SessionContext.Provider>
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  [...Object.values(cat), ...Object.values(cfg)].forEach((f) => f.mockReset());
  cfg.overview.mockResolvedValue(CONFIG);
  cat.search.mockResolvedValue({ total: 1, page: 1, pageSize: 25, rows: [SUMMARY] });
  cat.getRecord.mockResolvedValue(RECORD);
  cat.findDuplicates.mockResolvedValue([]);
  cat.accessionFormat.mockResolvedValue({ prefix: "", padding: 6, next: 4220 });
});

describe("CataloguePage", () => {
  it("lists records with authors, copies and a class label", async () => {
    renderAt("/catalogue");
    expect(await screen.findByRole("link", { name: "Balyakalasakhi" })).toHaveAttribute("href", "/catalogue/1");
    expect(screen.getByText(/Vaikom Muhammad Basheer/)).toBeInTheDocument();
    expect(screen.getByText("2 of 3 available")).toBeInTheDocument();
    expect(screen.getByRole("img", { name: "Call number 894.8123" })).toBeInTheDocument();
  });

  it("searches as the user types, with the backend doing the work", async () => {
    renderAt("/catalogue");
    await screen.findByRole("link", { name: "Balyakalasakhi" });
    await userEvent.type(screen.getByRole("searchbox", { name: "Search the catalogue" }), "balya");
    await waitFor(() => expect(cat.search).toHaveBeenLastCalledWith(expect.objectContaining({ text: "balya", page: 1, pageSize: 25 })));
  });

  it("opens the only match when a scanner presses Enter", async () => {
    renderAt("/catalogue");
    await screen.findByRole("link", { name: "Balyakalasakhi" });
    cat.getRecord.mockResolvedValue(RECORD);
    await userEvent.type(screen.getByRole("searchbox", { name: "Search the catalogue" }), "004217{Enter}");
    expect(await screen.findByRole("heading", { name: "Balyakalasakhi" })).toBeInTheDocument();
  });

  it("explains an empty result and an empty catalogue", async () => {
    cat.search.mockResolvedValue({ total: 0, page: 1, pageSize: 25, rows: [] });
    renderAt("/catalogue");
    expect(await screen.findByText(/catalogue is empty/i)).toBeInTheDocument();
    await userEvent.type(screen.getByRole("searchbox"), "zzz");
    expect(await screen.findByText(/No records match/)).toBeInTheDocument();
  });

  it("hides Add record from read-only staff", async () => {
    renderAt("/catalogue", CLERK);
    await screen.findByRole("link", { name: "Balyakalasakhi" });
    expect(screen.queryByRole("button", { name: "Add record" })).not.toBeInTheDocument();
  });

  it("shows the backend's message when searching fails", async () => {
    cat.search.mockRejectedValue(new ApiError({ code: "AUTH_FAILED", message: "Please sign in." }));
    renderAt("/catalogue");
    expect(await screen.findByRole("alert")).toHaveTextContent("Please sign in.");
  });
});

describe("Add record", () => {
  async function fillAndSave() {
    await userEvent.click(await screen.findByRole("button", { name: "Add record" }));
    const dialog = screen.getByRole("dialog", { name: "Add record" });
    await userEvent.type(within(dialog).getByLabelText("Title"), "Randamoozham");
    await userEvent.type(within(dialog).getByLabelText("Name 1"), "M. T. Vasudevan Nair");
    await userEvent.type(within(dialog).getByLabelText("Year"), "1984");
    await userEvent.type(within(dialog).getByLabelText("Subjects"), "Mahabharata{Enter}Novels");
    await userEvent.click(within(dialog).getByRole("button", { name: "Save record" }));
    return dialog;
  }

  it("sends the record exactly as the backend expects", async () => {
    cat.saveRecord.mockResolvedValue(9);
    renderAt("/catalogue");
    await fillAndSave();
    await waitFor(() => expect(cat.saveRecord).toHaveBeenCalledTimes(1));
    expect(cat.saveRecord.mock.calls[0]?.[0]).toMatchObject({
      id: null, title: "Randamoozham", subtitle: null, publicationYear: 1984, isbn: null,
      contributors: [{ name: "M. T. Vasudevan Nair", role: "author" }], subjects: ["Mahabharata", "Novels"],
    });
    expect(cat.findDuplicates).toHaveBeenCalledWith(expect.objectContaining({ title: "Randamoozham", firstAuthor: "M. T. Vasudevan Nair", publicationYear: 1984 }));
  });

  it("warns about duplicates and only saves after the librarian confirms", async () => {
    cat.findDuplicates.mockResolvedValue([{ id: 4, title: "Randamoozham", authors: "M. T. Vasudevan Nair", publicationYear: 1984, reason: "Same title, author and year" }]);
    cat.saveRecord.mockResolvedValue(9);
    renderAt("/catalogue");
    const dialog = await fillAndSave();
    expect(await within(dialog).findByText(/looks like a record you already have/)).toBeInTheDocument();
    expect(cat.saveRecord).not.toHaveBeenCalled();
    await userEvent.click(within(dialog).getByRole("button", { name: "Save as a new record" }));
    await waitFor(() => expect(cat.saveRecord).toHaveBeenCalledTimes(1));
  });

  it("needs a title and a numeric year before calling the backend", async () => {
    renderAt("/catalogue");
    await userEvent.click(await screen.findByRole("button", { name: "Add record" }));
    const dialog = screen.getByRole("dialog");
    await userEvent.click(within(dialog).getByRole("button", { name: "Save record" }));
    expect(within(dialog).getByText("Enter the title.")).toBeInTheDocument();
    await userEvent.type(within(dialog).getByLabelText("Title"), "X");
    await userEvent.type(within(dialog).getByLabelText("Year"), "abc");
    await userEvent.click(within(dialog).getByRole("button", { name: "Save record" }));
    expect(within(dialog).getByText(/must be a number/)).toBeInTheDocument();
    expect(cat.findDuplicates).not.toHaveBeenCalled();
  });

  it("shows the backend's message, for example an invalid ISBN", async () => {
    cat.saveRecord.mockRejectedValue(new ApiError({ code: "VALIDATION", message: "That ISBN is not valid. Check the digits." }));
    renderAt("/catalogue");
    const dialog = await fillAndSave();
    expect(await within(dialog).findByText("That ISBN is not valid. Check the digits.")).toBeInTheDocument();
  });
});

describe("RecordPage", () => {
  it("shows details, contributors, subjects and every copy", async () => {
    renderAt("/catalogue/1");
    expect(await screen.findByRole("heading", { name: "Balyakalasakhi" })).toBeInTheDocument();
    expect(screen.getByText("Vaikom Muhammad Basheer")).toBeInTheDocument();
    expect(screen.getByText("Malayalam fiction")).toBeInTheDocument();
    const copies = screen.getByRole("region", { name: /Copies/ });
    expect(within(copies).getByText("004217")).toBeInTheDocument();
    expect(within(copies).getByText("Available")).toBeInTheDocument();
    expect(within(copies).getByText(/Main Library · A1/)).toBeInTheDocument();
    expect(within(copies).getByText(/₹250\.00/)).toBeInTheDocument();
  });

  it("adds copies with a suggested call number and the price in minor units", async () => {
    cat.addCopies.mockResolvedValue([COPY]);
    renderAt("/catalogue/1");
    await userEvent.click(await screen.findByRole("button", { name: "Add copies" }));
    const dialog = screen.getByRole("dialog", { name: /Add copies of/ });
    expect(within(dialog).getByLabelText("Call number")).toHaveValue("894.8123 BAS");
    const count = within(dialog).getByLabelText("Number of copies");
    await userEvent.clear(count);
    await userEvent.type(count, "3");
    await userEvent.type(within(dialog).getByLabelText(/Price/), "249.50");
    await userEvent.click(within(dialog).getByRole("button", { name: "Add copies" }));
    await waitFor(() => expect(cat.addCopies).toHaveBeenCalledTimes(1));
    expect(cat.addCopies.mock.calls[0]).toEqual([1, expect.objectContaining({
      count: 3, accessionNumbers: null, barcodes: null, itemCategoryId: 5, priceMinor: 24950, callNumber: "894.8123 BAS", acquisitionSource: "purchase",
    })]);
  });

  it("requires one accession number per copy when typing them by hand", async () => {
    renderAt("/catalogue/1");
    await userEvent.click(await screen.findByRole("button", { name: "Add copies" }));
    const dialog = screen.getByRole("dialog");
    await userEvent.clear(within(dialog).getByLabelText("Number of copies"));
    await userEvent.type(within(dialog).getByLabelText("Number of copies"), "2");
    await userEvent.click(within(dialog).getByLabelText(/type the accession numbers myself/));
    await userEvent.type(within(dialog).getByLabelText("Accession numbers"), "A-1");
    await userEvent.click(within(dialog).getByRole("button", { name: "Add copies" }));
    expect(within(dialog).getByText(/one accession number on each line/i)).toBeInTheDocument();
    expect(cat.addCopies).not.toHaveBeenCalled();
  });

  it("changes a copy's status and shows the conflict message from the backend", async () => {
    cat.setCopyStatus.mockRejectedValue(new ApiError({ code: "CONFLICT", message: "This copy is on loan or reserved. Return it first." }));
    renderAt("/catalogue/1");
    await userEvent.click(await screen.findByRole("button", { name: "Status" }));
    const dialog = screen.getByRole("dialog");
    await userEvent.selectOptions(within(dialog).getByLabelText("Status"), "damaged");
    await userEvent.click(within(dialog).getByRole("button", { name: "Change status" }));
    expect(cat.setCopyStatus).toHaveBeenCalledWith(1, "damaged", null);
    expect(await within(dialog).findByRole("alert")).toHaveTextContent("Return it first");
  });

  it("opens a printable label with a barcode", async () => {
    renderAt("/catalogue/1");
    await userEvent.click(await screen.findByRole("button", { name: "Label" }));
    const dialog = screen.getByRole("dialog", { name: "Label for 004217" });
    expect(within(dialog).getByRole("img", { name: "Barcode 004217" })).toBeInTheDocument();
    expect(within(dialog).getByRole("button", { name: "Print label" })).toBeInTheDocument();
  });

  it("hides editing from read-only staff but still shows labels", async () => {
    renderAt("/catalogue/1", CLERK);
    await screen.findByRole("heading", { name: "Balyakalasakhi" });
    for (const name of ["Edit record", "Archive", "Add copies", "Edit", "Status"]) {
      expect(screen.queryByRole("button", { name })).not.toBeInTheDocument();
    }
    expect(screen.getByRole("button", { name: "Label" })).toBeInTheDocument();
  });

  it("explains a missing record", async () => {
    cat.getRecord.mockRejectedValue(new ApiError({ code: "NOT_FOUND", message: "That record no longer exists." }));
    renderAt("/catalogue/99");
    expect(await screen.findByRole("alert")).toHaveTextContent("no longer exists");
  });
});

describe("AccessionPage", () => {
  it("lists copies and filters by status through the backend", async () => {
    cat.listCopies.mockResolvedValue({
      total: 1, page: 1, pageSize: 25,
      rows: [{ id: 1, recordId: 1, accessionNo: "004217", barcode: "004217", title: "Balyakalasakhi", callNumber: "894.8123 BAS", status: "damaged", collection: "Fiction", location: "Main Library", shelf: "A1", acquisitionDate: null, priceMinor: 25000 }],
    });
    renderAt("/accession");
    expect(await screen.findByText("004217")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Balyakalasakhi" })).toHaveAttribute("href", "/catalogue/1");
    await userEvent.selectOptions(screen.getByLabelText("Status"), "damaged");
    await waitFor(() => expect(cat.listCopies).toHaveBeenLastCalledWith(expect.objectContaining({ status: "damaged", page: 1 })));
    await userEvent.click(screen.getByRole("button", { name: "Inventory sheet for this page" }));
    const sheet = within(screen.getByRole("dialog", { name: "Inventory sheet" }));
    expect(sheet.getByRole("cell", { name: "004217" })).toBeInTheDocument();
    expect(sheet.getByRole("cell", { name: "Main Library / A1" })).toBeInTheDocument();
    expect(sheet.getByRole("cell", { name: "Damaged" })).toBeInTheDocument();
    await userEvent.click(sheet.getByRole("button", { name: "Close" }));
    cat.exportCopies.mockRejectedValue(new ApiError({ code: "IO", message: "Export unavailable." }));
    await userEvent.click(screen.getByRole("button", { name: "Export matching copies" }));
    expect(cat.exportCopies).toHaveBeenCalledWith(expect.objectContaining({ status: "damaged" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Export unavailable.");
  });

  it("prints selected copies across pages and clears the print sheet afterward", async () => {
    const first: CopyRow = {
      id: 1, recordId: 1, accessionNo: "004217", barcode: "004217", title: "Balyakalasakhi",
      callNumber: "894.8123 BAS", status: "available", collection: null, location: null, shelf: null,
      acquisitionDate: null, priceMinor: null,
    };
    const second = { ...first, id: 2, accessionNo: "004218", barcode: "പുസ്തകം", callNumber: null };
    cat.listCopies.mockImplementation(async ({ page }: CopyQuery) => ({
      total: 2, page, pageSize: 1, rows: [page === 1 ? first : second],
    }));
    const print = vi.spyOn(window, "print").mockImplementation(() => {
      const printable = document.querySelector<HTMLDivElement>("body > .label-sheet-print");
      if (!printable) throw new Error("Print sheet is missing.");
      expect(within(printable).getByText("Accession 004217")).toBeInTheDocument();
      expect(within(printable).getByText("Accession 004218")).toBeInTheDocument();
      expect(within(printable).getByRole("img", { name: "Barcode 004217" }).querySelector("g rect")).not.toBeNull();
      window.dispatchEvent(new Event("afterprint"));
    });
    try {
      renderAt("/accession", CLERK);
      await screen.findByRole("checkbox", { name: "Select copy 004217" });
      expect(screen.getByRole("button", { name: "Preview labels" })).toBeDisabled();
      await userEvent.click(screen.getByRole("checkbox", { name: "Select copies on this page" }));
      await userEvent.click(screen.getByRole("button", { name: /next/i }));
      await userEvent.click(await screen.findByRole("checkbox", { name: "Select copy 004218" }));
      await userEvent.click(screen.getByRole("button", { name: "Preview labels (2)" }));
      const preview = within(screen.getByRole("dialog", { name: "Copy label sheet" }));
      expect(preview.getByRole("img", { name: "Barcode 004217" })).toBeInTheDocument();
      expect(preview.getByText("Barcode പുസ്തകം (text only)")).toBeInTheDocument();
      await userEvent.click(preview.getByRole("button", { name: "Print labels" }));
      expect(print).toHaveBeenCalledOnce();
      expect(document.querySelector(".label-sheet-print")).toBeNull();
      await userEvent.click(preview.getByRole("button", { name: "Close" }));
      await userEvent.click(screen.getByRole("button", { name: "Clear selection" }));
      expect(screen.getByRole("button", { name: "Preview labels" })).toBeDisabled();
    } finally {
      print.mockRestore();
    }
  });
});
