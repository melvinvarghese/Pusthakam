import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import { afterEach, describe, expect, it, vi } from "vitest";
import { SessionContext } from "@/lib/session";
import { LabelSheetDialog } from "@/features/catalogue/LabelSheetDialog";
import { MemberCard } from "@/features/members/MemberCard";
import { DueSlip } from "@/features/circulation/DueSlip";
import { CirculationPage } from "@/features/circulation/CirculationPage";
import { InventoryPage } from "@/features/inventory/InventoryPage";
import type { CopyRow } from "@/lib/api/types";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
  vi.mocked(invoke).mockReset();
});

describe("Hardware printing and USB scanner acceptance (PRN-HARDWARE)", () => {
  it("prints copy label sheet with barcodes and metadata on OS printer", async () => {
    vi.stubGlobal("__TAURI_INTERNALS__", {});
    const printSpy = vi.spyOn(window, "print").mockImplementation(() => {
      const printSheet = document.querySelector("body > .label-sheet-print");
      expect(printSheet).toBeInTheDocument();
      expect(printSheet).toHaveTextContent("Accession ACC-001");
      expect(printSheet).toHaveTextContent("The Hobbit");
      expect(printSheet).toHaveTextContent("823 TOL");
      window.dispatchEvent(new Event("afterprint"));
    });

    const copies: CopyRow[] = [
      { id: 1, recordId: 10, title: "The Hobbit", accessionNo: "ACC-001", barcode: "ACC-001", callNumber: "823 TOL", location: "General", shelf: "G1", collection: "General", acquisitionDate: "2026-01-01", priceMinor: 25000, status: "available" },
      { id: 2, recordId: 10, title: "The Hobbit", accessionNo: "ACC-002", barcode: "ACC-002", callNumber: "823 TOL", location: "General", shelf: "G1", collection: "General", acquisitionDate: "2026-01-01", priceMinor: 25000, status: "available" },
    ];

    render(
      <QueryClientProvider client={new QueryClient()}>
        <LabelSheetDialog copies={copies} onClose={vi.fn()} />
      </QueryClientProvider>,
    );

    expect(screen.getByRole("heading", { name: "Copy label sheet" })).toBeInTheDocument();
    expect(screen.getByText("2 labels. Use plain paper or full-sheet label stock.")).toBeInTheDocument();

    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: "Print labels" }));
    expect(printSpy).toHaveBeenCalledOnce();
  });

  it("prints member card with barcode on OS printer", async () => {
    vi.stubGlobal("__TAURI_INTERNALS__", {});
    vi.mocked(invoke).mockImplementation(async (command) => {
      if (command === "members_get_photo") return null;
      return null;
    });

    const printSpy = vi.spyOn(window, "print").mockImplementation(() => {
      const area = document.querySelector(".print-area");
      expect(area).toHaveTextContent("St. Mary's Library");
      expect(area).toHaveTextContent("Deepa Nair");
      expect(area).toHaveTextContent("General · Valid through 2027-12-31");
      expect(area?.querySelector("svg")).toBeInTheDocument();
    });

    const member = {
      id: 1,
      memberNumber: "M-1001",
      name: "Deepa Nair",
      categoryId: 1,
      categoryName: "General",
      email: "deepa@example.com",
      phone: "9876543210",
      address: "123 Main St",
      joinedOn: "2026-01-01",
      expiresOn: "2027-12-31",
      archived: false,
    };

    render(
      <QueryClientProvider client={new QueryClient()}>
        <MemberCard member={member} libraryName="St. Mary's Library" onClose={vi.fn()} />
      </QueryClientProvider>,
    );

    expect(screen.getByRole("heading", { name: "Member card" })).toBeInTheDocument();
    expect(await screen.findByText("Deepa Nair")).toBeInTheDocument();

    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: "Print card" }));
    expect(printSpy).toHaveBeenCalledOnce();
  });

  it("prints due slip with loan details and library heading on OS printer", async () => {
    vi.stubGlobal("__TAURI_INTERNALS__", {});
    vi.mocked(invoke).mockImplementation(async (command) => {
      if (command === "config_overview") {
        return {
          library: { name: "District Central Library", email: "info@lib.gov", phone: "12345", currency: "INR" },
          policy: { loanDays: 14, renewLimit: 2, finePerDayMinor: 100, graceDays: 1 },
        };
      }
      return null;
    });

    const printSpy = vi.spyOn(window, "print").mockImplementation(() => {
      const area = document.querySelector(".print-area");
      expect(area).toHaveTextContent("District Central Library");
      expect(area).toHaveTextContent("Loan issue");
      expect(area).toHaveTextContent("Deepa Nair");
      expect(area).toHaveTextContent("A Passage to India");
      expect(area).toHaveTextContent("2026-12-05");
    });

    const loan = {
      id: 10,
      loanId: 10,
      memberId: 1,
      memberName: "Deepa Nair",
      memberNumber: "M-1001",
      itemId: 42,
      barcode: "ACC-0042",
      accessionNo: "ACC-0042",
      title: "A Passage to India",
      callNumber: "823 FOR",
      issuedOn: "2026-11-20",
      dueOn: "2026-12-05",
      renewals: 0,
    };

    render(
      <QueryClientProvider client={new QueryClient()}>
        <DueSlip loan={loan} renewed={false} onClose={vi.fn()} />
      </QueryClientProvider>,
    );

    expect(screen.getByRole("heading", { name: "Loan due slip" })).toBeInTheDocument();
    expect(await screen.findByText("A Passage to India")).toBeInTheDocument();

    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: "Print slip" }));
    expect(printSpy).toHaveBeenCalledOnce();
  });

  it("resolves USB keyboard-wedge barcode scanner input across circulation fields", async () => {
    vi.stubGlobal("__TAURI_INTERNALS__", {});
    vi.mocked(invoke).mockImplementation(async (command, _args) => {
      if (command === "circulation_prepare_issue") {
        return {
          allowed: true,
          memberId: 1,
          memberName: "Deepa Nair",
          memberNumber: "M-1001",
          itemId: 42,
          barcode: "ACC-0042",
          title: "War and Peace",
          dueOn: "2026-12-04",
          finesOwedMinor: 0,
        };
      }
      if (command === "circulation_overdue_list") return [];
      if (command === "circulation_reservation_list") return [];
      return null;
    });

    render(
      <QueryClientProvider client={new QueryClient()}>
        <SessionContext.Provider value={{
          session: { userId: 1, username: "librarian", displayName: "Librarian", role: "circulation", permissions: ["circulation.issue", "members.read"] },
          signOut: async () => {},
        }}>
          <CirculationPage />
        </SessionContext.Provider>
      </QueryClientProvider>,
    );

    const user = userEvent.setup();
    const memberInput = screen.getByRole("textbox", { name: "Member number" });
    const bookBarcode = screen.getByRole("textbox", { name: "Book barcode or accession number" });

    // Step 1: Scanner scans member card with Enter wedge
    await user.type(memberInput, "M-1001{Enter}");

    // Focus automatically shifts to book barcode
    await waitFor(() => {
      expect(bookBarcode).toHaveFocus();
    });

    // Step 2: Scanner scans book barcode with Enter wedge
    await user.type(bookBarcode, "ACC-0042{Enter}");

    // Issue preview is prepared and presented
    expect(await screen.findByRole("heading", { name: "Confirm issue" })).toBeInTheDocument();
    expect(screen.getByText("Deepa Nair")).toBeInTheDocument();
    expect(screen.getByText("War and Peace")).toBeInTheDocument();
    expect(screen.getByText("2026-12-04")).toBeInTheDocument();
  });

  it("records USB keyboard-wedge scanner input ending with Enter during stock check", async () => {
    vi.stubGlobal("__TAURI_INTERNALS__", {});
    const check = { id: 8, location: "Main Library", shelf: "A1", startedAt: "2026-11-17T12:00:00Z", closedAt: null, expected: 1, matched: 0, unexpected: 0 };
    let scanned = false;
    vi.mocked(invoke).mockImplementation(async (command) => {
      if (command === "stock_check_list") return { rows: [check], total: 1, page: 1, pageSize: 25 };
      if (command === "stock_check_scan") { scanned = true; return "matched"; }
      if (command === "stock_check_review") return {
        check: { ...check, matched: Number(scanned) },
        entries: { rows: scanned ? [] : [{ barcode: "ACC-0042", accessionNo: "0042", title: "A Passage to India" }], page: 1, pageSize: 25, total: Number(!scanned) },
      };
      return null;
    });

    render(
      <QueryClientProvider client={new QueryClient()}>
        <SessionContext.Provider value={{
          session: { userId: 1, username: "admin", displayName: "Administrator", role: "administrator", permissions: ["catalogue.read", "catalogue.write"] },
          signOut: async () => {},
        }}>
          <InventoryPage />
        </SessionContext.Provider>
      </QueryClientProvider>,
    );

    const user = userEvent.setup();
    await user.click(await screen.findByRole("button", { name: "Open check" }));
    expect(await screen.findByText("A Passage to India")).toBeInTheDocument();

    const field = screen.getByRole("textbox", { name: "Scan barcode" });

    // USB scanner wedge sends rapid barcode keystrokes terminated with Enter
    await user.type(field, "ACC-0042{Enter}");

    expect(await screen.findByText("Matched an expected copy.")).toBeInTheDocument();
    expect(invoke).toHaveBeenCalledWith("stock_check_scan", { id: 8, barcode: "ACC-0042" });
    expect(field).toHaveFocus();
  });
});

it("prints spine labels with call numbers and no barcode graphics",async()=>{
 vi.stubGlobal("__TAURI_INTERNALS__",{});vi.mocked(invoke).mockResolvedValue(null);
 const print=vi.spyOn(window,"print").mockImplementation(()=>{
  const sheet=document.querySelector("body > .label-sheet-print");
  expect(sheet).toHaveTextContent("823 TOL");expect(sheet?.querySelector("svg")).toBeNull();
  window.dispatchEvent(new Event("afterprint"));
 });
 const copy={id:1,recordId:1,title:"Book",accessionNo:"A1",barcode:"B1",callNumber:"823 TOL",status:"available",collection:null,location:null,shelf:null,acquisitionDate:null,priceMinor:null} as CopyRow;
 render(<QueryClientProvider client={new QueryClient()}><LabelSheetDialog copies={[copy]} onClose={vi.fn()} /></QueryClientProvider>);
 const user=userEvent.setup();await user.selectOptions(screen.getByRole("combobox",{name:"Label style"}),"spine");
 await user.click(screen.getByRole("button",{name:"Print labels"}));expect(print).toHaveBeenCalledOnce();
});
