import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ConfigOverview, ImportPreview, ImportReport } from "@/lib/api/types";

const imp = vi.hoisted(() => ({ preview: vi.fn(), commit: vi.fn(), cancel: vi.fn() }));
const cfg = vi.hoisted(() => ({ overview: vi.fn() }));
vi.mock("@/lib/api", async (orig) => ({ ...(await orig<typeof import("@/lib/api")>()), importApi: imp, configApi: cfg }));

import { ApiError } from "@/lib/api";
import { fileToBase64 } from "@/lib/files";
import { ImportDialog } from "./ImportDialog";

const CONFIG: ConfigOverview = {
  library: { name: "Demo", profile: "school", address: null, contact: null, currency: "INR", locale: "en" },
  classificationSchemes: [], memberCategories: [], collections: [], locations: [], shelves: [], policies: [],
  itemCategories: [{ id: 5, name: "General", description: null, archived: false }],
};

const PREVIEW: ImportPreview = {
  token: "tok1", fileName: "export.mrc", format: "marc", profile: "koha",
  totals: { records: 2, copies: 3, withWarnings: 1, withErrors: 1, duplicates: 1, copiesSkipped: 1 },
  newLookups: { locations: ["MAIN"], collections: ["FICTION"], itemCategories: ["BK"] },
  sample: [
    { row: 1, title: "Balyakalasakhi", contributors: [{ name: "Basheer", role: "author" }], isbn: null, copies: [{ barcode: "1", accession: null, callNumber: null, location: "MAIN", shelf: null, collection: null, itemCategory: "BK", acquisitionDate: null, priceMinor: null, status: null, skipReason: null }], warnings: ["A record with this ISBN is already in the catalogue."], errors: [], duplicateOf: 3 },
    { row: 2, title: "", contributors: [], isbn: null, copies: [], warnings: [], errors: ["No title. This record cannot be imported."], duplicateOf: null },
  ],
  parseErrors: ["Record 3: the directory is damaged."],
  headers: null, mapping: null, mappingFields: null,
};

const REPORT: ImportReport = {
  createdRecords: 1, mergedRecords: 0, skippedRecords: 1, failedRecords: 1, copiesAdded: 2, copiesSkipped: 1, createdLookups: 3,
  problems: [{ row: 2, message: "No title. This record cannot be imported." }],
};

function renderDialog(onClose = vi.fn()) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(<QueryClientProvider client={qc}><ImportDialog onClose={onClose} /></QueryClientProvider>);
  return onClose;
}

const mrc = () => new File([new Uint8Array([0x30, 0x30, 0x1d])], "export.mrc");

beforeEach(() => {
  [...Object.values(imp), ...Object.values(cfg)].forEach((f) => f.mockReset());
  cfg.overview.mockResolvedValue(CONFIG);
  imp.cancel.mockResolvedValue(undefined);
});

describe("fileToBase64", () => {
  it("encodes bytes, including ones above 127", async () => {
    expect(await fileToBase64(new File([new Uint8Array([0, 1, 200, 255])], "x"))).toBe("AAHI/w==");
  });
});

describe("ImportDialog", () => {
  it("explains how to export from Koha and offers every source", () => {
    renderDialog();
    expect(screen.getByRole("dialog", { name: "Import records" })).toBeInTheDocument();
    expect(screen.getByText(/including items/i)).toBeInTheDocument();
    expect(within(screen.getByLabelText("Where is the data from?")).getAllByRole("option")).toHaveLength(3);
    expect(screen.getByRole("button", { name: "Preview" })).toBeDisabled();
  });

  it("previews a Koha file, shows problems before saving, then imports with the chosen options", async () => {
    imp.preview.mockResolvedValue(PREVIEW);
    imp.commit.mockResolvedValue(REPORT);
    renderDialog();
    await userEvent.upload(screen.getByLabelText("File"), mrc());
    await userEvent.click(screen.getByRole("button", { name: "Preview" }));

    expect(imp.preview).toHaveBeenCalledWith(expect.objectContaining({ fileName: "export.mrc", format: "marc", profile: "koha", dataBase64: "MDAd", mapping: null }));
    expect(await screen.findByText("Check before importing: export.mrc")).toBeInTheDocument();
    expect(screen.getByText("No title. This record cannot be imported.")).toBeInTheDocument();
    expect(screen.getByText(/directory is damaged/)).toBeInTheDocument();
    expect(screen.getByText(/New in your library/)).toHaveTextContent("locations MAIN");
    expect(imp.commit).not.toHaveBeenCalled();

    await userEvent.selectOptions(screen.getByLabelText(/already in the catalogue \(same ISBN\)/), "merge");
    await userEvent.click(screen.getByRole("button", { name: "Import 1 record" }));
    expect(imp.commit).toHaveBeenCalledWith("tok1", { duplicatePolicy: "merge", createMissing: true, defaultItemCategoryId: 5, barcodeAsAccession: true });

    expect(await screen.findByRole("dialog", { name: "Import finished" })).toBeInTheDocument();
    expect(screen.getByText("Row 2: No title. This record cannot be imported.")).toBeInTheDocument();
    expect(screen.getByText(/3 new locations/)).toBeInTheDocument();
  });

  it("recognises a CSV by its name and lets the librarian fix a column", async () => {
    const csvPreview: ImportPreview = {
      ...PREVIEW, format: "csv", profile: "generic", fileName: "books.csv", parseErrors: [],
      headers: ["Heading", "Writer"], mapping: { title: "Heading" },
      mappingFields: ["title", "author"],
    };
    imp.preview.mockResolvedValueOnce(csvPreview).mockResolvedValueOnce({ ...csvPreview, mapping: { title: "Heading", author: "Writer" } });
    renderDialog();
    await userEvent.upload(screen.getByLabelText("File"), new File(["Heading,Writer\nA,B\n"], "books.csv"));
    expect(screen.getByLabelText("Where is the data from?")).toHaveValue("csv");
    await userEvent.click(screen.getByRole("button", { name: "Preview" }));
    await userEvent.selectOptions(await screen.findByLabelText("Column for Author"), "Writer");
    await waitFor(() => expect(imp.preview).toHaveBeenCalledTimes(2));
    expect(imp.preview.mock.calls[1]?.[0]).toMatchObject({ format: "csv", mapping: { title: "Heading", author: "Writer" } });
  });

  it("shows the backend's message when a file cannot be read", async () => {
    imp.preview.mockRejectedValue(new ApiError({ code: "VALIDATION", message: "The file is empty." }));
    renderDialog();
    await userEvent.upload(screen.getByLabelText("File"), mrc());
    await userEvent.click(screen.getByRole("button", { name: "Preview" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("The file is empty.");
  });

  it("keeps the preview open and shows the message if the import is refused", async () => {
    imp.preview.mockResolvedValue(PREVIEW);
    imp.commit.mockRejectedValue(new ApiError({ code: "VALIDATION", message: "Choose an item category for copies that do not name one." }));
    renderDialog();
    await userEvent.upload(screen.getByLabelText("File"), mrc());
    await userEvent.click(screen.getByRole("button", { name: "Preview" }));
    await userEvent.click(await screen.findByRole("button", { name: "Import 1 record" }));
    // The preview already shows an alert about unreadable records; the refusal appears beside it.
    expect(await screen.findByText(/Choose an item category for copies/)).toHaveAttribute("role", "alert");
    expect(screen.getByText("Check before importing: export.mrc")).toBeInTheDocument();
  });

  it("discards the waiting preview when cancelled", async () => {
    imp.preview.mockResolvedValue(PREVIEW);
    const onClose = renderDialog();
    await userEvent.upload(screen.getByLabelText("File"), mrc());
    await userEvent.click(screen.getByRole("button", { name: "Preview" }));
    await screen.findByText("Check before importing: export.mrc");
    await userEvent.click(screen.getByRole("button", { name: "Back" }));
    await userEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(onClose).toHaveBeenCalled();
  });
});
