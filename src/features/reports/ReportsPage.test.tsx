import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import { afterEach, expect, it, vi } from "vitest";
import { SessionContext } from "@/lib/session";
import ReportsPage from "./ReportsPage";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
afterEach(() => { vi.unstubAllGlobals(); vi.restoreAllMocks(); vi.mocked(invoke).mockReset(); });

function renderReports() {
  vi.stubGlobal("__TAURI_INTERNALS__", {});
  return render(
    <QueryClientProvider client={new QueryClient()}>
      <SessionContext.Provider value={{
        session: { userId: 1, username: "viewer", displayName: "Viewer", role: "viewer", permissions: ["reports.read"] },
        signOut: async () => {},
      }}>
        <ReportsPage />
      </SessionContext.Provider>
    </QueryClientProvider>,
  );
}

async function exportCsv(filename: RegExp) {
  const createObjectURL = vi.fn((_blob: Blob) => "blob:test");
  vi.stubGlobal("URL", Object.assign(class extends URL {}, { createObjectURL, revokeObjectURL: vi.fn() }));
  const click = vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(function (this: HTMLAnchorElement) {
    expect(this.download).toMatch(filename);
  });
  await userEvent.click(screen.getByRole("button", { name: "Export CSV" }));
  expect(click).toHaveBeenCalledOnce();
  const blob = createObjectURL.mock.calls[0]?.[0];
  if (!blob) throw new Error("CSV export did not create a file.");
  return new Promise<string>((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result));
    reader.onerror = reject;
    reader.readAsText(blob);
  });
}

it("exports the displayed circulation rows and totals as a dated CSV", async () => {
  vi.mocked(invoke).mockResolvedValue([
    { date: "2026-11-16", issued: 3, returned: 2, lost: 0, renewed: 1, reservations: 4 },
    { date: "2026-11-17", issued: 2, returned: 1, lost: 1, renewed: 0, reservations: 0 },
  ]);
  renderReports();
  expect(await screen.findByRole("rowheader", { name: "2026-11-16" })).toBeInTheDocument();
  expect(screen.getByRole("row", { name: "Total 5 3 1 1 4" })).toBeInTheDocument();
  expect(await exportCsv(/^circulation-\d{4}-\d{2}-\d{2}-to-\d{4}-\d{2}-\d{2}\.csv$/)).toBe(
    "Date,Issued,Returned,Closed as lost,Renewed,Reservations placed\r\n2026-11-16,3,2,0,1,4\r\n2026-11-17,2,1,1,0,0\r\nTotal,5,3,1,1,4\r\n",
  );
  const print = vi.spyOn(window, "print").mockImplementation(() => {
    expect(document.querySelector("body > .report-print table")).toHaveTextContent("2026-11-16");
    window.dispatchEvent(new Event("afterprint"));
  });
  await userEvent.click(screen.getByRole("button", { name: "Print report" }));
  expect(print).toHaveBeenCalledOnce();
  expect(document.querySelector(".report-print")).toBeNull();

  vi.mocked(invoke).mockResolvedValueOnce([
    { currency: "INR", assessed: "250", paid: "125", waived: "25", outstanding: "100" },
    { currency: "USD", assessed: "9007199254740993", paid: "0", waived: "0", outstanding: "9007199254740993" },
  ]);
  await userEvent.selectOptions(screen.getByRole("combobox", { name: "Report" }), "fines");
  const summary = await screen.findByRole("table", { name: "All-time fine summary by currency" });
  expect(summary).toHaveTextContent("INR2.501.250.251.00");
  expect(summary).toHaveTextContent("90071992547409.93");
  expect(screen.queryByLabelText("Start date")).not.toBeInTheDocument();

  vi.mocked(invoke).mockResolvedValueOnce([
    { date: "2026-11-16", titlesAdded: 2, copiesAdded: 5 },
    { date: "2026-11-17", titlesAdded: 0, copiesAdded: 1 },
  ]);
  await userEvent.selectOptions(screen.getByRole("combobox", { name: "Report" }), "growth");
  expect(await screen.findByRole("table", { name: /Titles and copies added/ })).toBeInTheDocument();
  expect(screen.getByRole("row", { name: "Total 2 6" })).toBeInTheDocument();
  expect(screen.getByLabelText("Start date")).toBeInTheDocument();

  vi.mocked(invoke).mockResolvedValueOnce([
    { categoryId: 1, category: "Students", archivedCategory: false, current: 3, expired: 2, archived: 1, total: 6 },
    { categoryId: 2, category: "Former staff", archivedCategory: true, current: 0, expired: 0, archived: 2, total: 2 },
  ]);
  await userEvent.selectOptions(screen.getByRole("combobox", { name: "Report" }), "members");
  expect(await screen.findByRole("table", { name: "Members by category" })).toBeInTheDocument();
  expect(screen.getByRole("row", { name: "All categories 3 2 3 8" })).toBeInTheDocument();
  expect(screen.getByText("Archived category")).toBeInTheDocument();
  expect(screen.queryByLabelText("Start date")).not.toBeInTheDocument();
});

it("disables export and print after a failed refresh until a retry succeeds", async () => {
  const rows = [{ date: "2026-11-16", issued: 1, returned: 0, lost: 0, renewed: 0, reservations: 0 }];
  vi.mocked(invoke).mockResolvedValueOnce(rows)
    .mockRejectedValueOnce({ code: "INTERNAL", message: "Report refresh failed." })
    .mockResolvedValueOnce(rows);
  renderReports();

  const user = userEvent.setup();
  const run = screen.getByRole("button", { name: "Run report" });
  const download = screen.getByRole("button", { name: "Export CSV" });
  const print = screen.getByRole("button", { name: "Print report" });
  await screen.findByRole("rowheader", { name: "2026-11-16" });
  expect(download).toBeEnabled();
  expect(print).toBeEnabled();

  await user.click(run);
  expect(await screen.findByRole("alert")).toHaveTextContent("Report refresh failed.");
  expect(download).toBeDisabled();
  expect(print).toBeDisabled();

  await user.click(run);
  await waitFor(() => expect(download).toBeEnabled());
  expect(print).toBeEnabled();
  expect(screen.queryByRole("alert")).not.toBeInTheDocument();
});

it.each([
  {
    kind: "popular", table: /Top 25 borrowed titles/, filename: /^most-borrowed-.*\.csv$/,
    rows: [{ recordId: 1, title: ' =SUM(1,"x")', archived: true, issues: 3 }],
    expected: "Title,Issues,Catalogue status\r\n\"' =SUM(1,\"\"x\"\")\",3,Archived\r\n",
  },
  {
    kind: "stock", table: /Stock by location/, filename: /^stock-by-location-.*\.csv$/,
    rows: [
      { locationId: 1, location: "＝Main", archivedLocation: true, available: 1, onLoan: 1, held: 1, lost: 0, damaged: 0, other: 0, total: 3 },
      { locationId: null, location: null, archivedLocation: false, available: 0, onLoan: 0, held: 0, lost: 1, damaged: 1, other: 1, total: 3 },
    ],
    expected: "Location,Location status,Available,On loan,Held,Lost,Damaged,In repair or transit,Total\r\n\"'＝Main\",Archived,1,1,1,0,0,0,3\r\n\"Unassigned\",,0,0,0,1,1,1,3\r\nAll locations,,1,1,1,1,1,1,6\r\n",
  },
  {
    kind: "fines", table: /All-time fine summary/, filename: /^fine-summary-.*\.csv$/,
    rows: [
      { currency: "INR", assessed: "250", paid: "125", waived: "25", outstanding: "100" },
      { currency: "USD", assessed: "9007199254740993", paid: "0", waived: "0", outstanding: "9007199254740993" },
    ],
    expected: "Currency,Assessed,Paid,Waived,Outstanding\r\n\"INR\",2.50,1.25,0.25,1.00\r\n\"USD\",90071992547409.93,0.00,0.00,90071992547409.93\r\n",
  },
  {
    kind: "growth", table: /Titles and copies added/, filename: /^catalogue-growth-.*\.csv$/,
    rows: [{ date: "2026-11-16", titlesAdded: 2, copiesAdded: 5 }, { date: "2026-11-17", titlesAdded: 0, copiesAdded: 1 }],
    expected: "Date,Titles added,Copies added\r\n2026-11-16,2,5\r\n2026-11-17,0,1\r\nTotal,2,6\r\n",
  },
  {
    kind: "members", table: /Members by category/, filename: /^members-by-category-.*\.csv$/,
    rows: [
      { categoryId: 1, category: "@Students", archivedCategory: false, current: 3, expired: 2, archived: 1, total: 6 },
      { categoryId: 2, category: "Former staff", archivedCategory: true, current: 0, expired: 0, archived: 2, total: 2 },
    ],
    expected: "Category,Category status,Current,Expired,Archived,Total\r\n\"'@Students\",Active,3,2,1,6\r\n\"Former staff\",Archived,0,0,2,2\r\nAll categories,,3,2,3,8\r\n",
  },
  {
    kind: "overdues", table: /Overdue loans/, filename: /^overdue-loans-.*\.csv$/,
    rows: [{ loanId: 1, memberId: 2, memberName: "Alice", memberBarcode: "M001", itemId: 3, accessionNo: "A001", barcode: "B001", title: "Hamlet", dueDate: "2026-11-01", daysOverdue: 18 }],
    expected: "Due date,Days overdue,Member,Member barcode,Accession no,Barcode,Title\r\n2026-11-01,18,\"Alice\",\"M001\",\"A001\",\"B001\",\"Hamlet\"\r\n",
  },
  {
    kind: "accession", table: /Accession list/, filename: /^accession-list-.*\.csv$/,
    rows: [{ itemId: 1, accessionNo: "A001", barcode: "B001", title: "Hamlet", acquisitionDate: "2026-11-10", acquisitionSource: "purchase", priceMinor: 1500, location: "Main", status: "available" }],
    expected: "Accession no,Barcode,Title,Acquisition date,Source,Price,Location,Status\r\n\"A001\",\"B001\",\"Hamlet\",2026-11-10,\"purchase\",15.00,\"Main\",\"available\"\r\n",
  },
  {
    kind: "expenditure", table: /Expenditure by acquisition source/, filename: /^expenditure-.*\.csv$/,
    rows: [
      { acquisitionSource: "purchase", copies: 5, totalMinor: 7500 },
      { acquisitionSource: "donation", copies: 2, totalMinor: 0 },
    ],
    expected: "Acquisition source,Copies,Total cost\r\n\"purchase\",5,75.00\r\n\"donation\",2,0.00\r\nAll sources,7,75.00\r\n",
  },
  {
    kind: "loss", table: /Loss and damage/, filename: /^loss-damage-.*\.csv$/,
    rows: [{ loanId: 1, returnCondition: "lost", returnedDate: "2026-11-15", memberId: 2, memberName: "Bob", itemId: 3, accessionNo: "A002", title: "Othello" }],
    expected: "Returned date,Condition,Member,Accession no,Title\r\n2026-11-15,\"lost\",\"Bob\",\"A002\",\"Othello\"\r\n",
  },
])("exports the $kind report", async ({ kind, table, filename, rows, expected }) => {
  vi.mocked(invoke).mockResolvedValueOnce([]).mockResolvedValue(rows);
  renderReports();
  await userEvent.selectOptions(screen.getByRole("combobox", { name: "Report" }), kind);
  expect(await screen.findByRole("table", { name: table })).toBeInTheDocument();
  expect(await exportCsv(filename)).toBe(expected);
});

it("exports report to XLSX and PDF matching row counts and totals", async () => {
  vi.mocked(invoke).mockImplementation(async (cmd, args) => {
    if (cmd === "reports_circulation_activity") {
      return [
        { date: "2026-11-16", issued: 3, returned: 2, lost: 0, renewed: 1, reservations: 4 },
        { date: "2026-11-17", issued: 2, returned: 1, lost: 1, renewed: 0, reservations: 0 },
      ];
    }
    if (cmd === "reports_export_file") {
      const a = args as { format: string; fileName: string; rows: string[][] };
      expect(a.rows).toHaveLength(3); // 2 data rows + 1 totals row
      expect(a.rows[2]?.[0]).toBe("Total");
      return {
        fileName: a.fileName,
        mimeType: a.format === "xlsx" ? "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" : "application/pdf",
        dataBase64: "UEsD",
      };
    }
    return [];
  });

  const createObjectURL = vi.fn((_blob: Blob) => "blob:test");
  vi.stubGlobal("URL", Object.assign(class extends URL {}, { createObjectURL, revokeObjectURL: vi.fn() }));
  const downloads: string[] = [];
  vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(function (this: HTMLAnchorElement) {
    downloads.push(this.download);
  });

  renderReports();
  expect(await screen.findByRole("rowheader", { name: "2026-11-16" })).toBeInTheDocument();

  // Export XLSX
  await userEvent.click(screen.getByRole("button", { name: "Export XLSX" }));
  expect(downloads).toHaveLength(1);
  expect(downloads[0]).toMatch(/^circulation-.*\.xlsx$/);

  // Export PDF
  await userEvent.click(screen.getByRole("button", { name: "Export PDF" }));
  expect(downloads).toHaveLength(2);
  expect(downloads[1]).toMatch(/^circulation-.*\.pdf$/);
});

it("supports toggling simple report design mode for preview, printing and export", async () => {
  vi.mocked(invoke).mockImplementation(async (cmd, args) => {
    if (cmd === "reports_circulation_activity") {
      return [{ date: "2026-11-16", issued: 3, returned: 2, lost: 0, renewed: 1, reservations: 4 }];
    }
    if (cmd === "reports_export_file") {
      const a = args as { format: string; simpleDesign?: boolean };
      expect(a.simpleDesign).toBe(true);
      return { fileName: "report.pdf", mimeType: "application/pdf", dataBase64: "JVBERi0xLjQK" };
    }
    return [];
  });

  const createObjectURL = vi.fn((_blob: Blob) => "blob:test");
  vi.stubGlobal("URL", Object.assign(class extends URL {}, { createObjectURL, revokeObjectURL: vi.fn() }));
  vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(() => {});

  const { container } = renderReports();
  expect(await screen.findByRole("rowheader", { name: "2026-11-16" })).toBeInTheDocument();

  const checkbox = screen.getByRole("checkbox", { name: "Use simple report design" });
  expect(checkbox).not.toBeChecked();
  expect(container.querySelector(".report-polished")).toBeInTheDocument();

  // Toggle simple report design
  await userEvent.click(checkbox);
  expect(checkbox).toBeChecked();
  expect(container.querySelector(".report-simple")).toBeInTheDocument();

  // Print in simple design mode
  const print = vi.spyOn(window, "print").mockImplementation(() => {
    const printEl = document.querySelector("body > .report-print");
    expect(printEl).toHaveClass("report-simple");
    window.dispatchEvent(new Event("afterprint"));
  });
  await userEvent.click(screen.getByRole("button", { name: "Print report" }));
  expect(print).toHaveBeenCalledOnce();

  // Export PDF with simpleDesign
  await userEvent.click(screen.getByRole("button", { name: "Export PDF" }));
  expect(vi.mocked(invoke)).toHaveBeenCalledWith("reports_export_file", expect.objectContaining({ simpleDesign: true }));
});
