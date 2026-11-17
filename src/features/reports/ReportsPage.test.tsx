import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import { afterEach, expect, it, vi } from "vitest";
import { SessionContext } from "@/lib/session";
import ReportsPage from "./ReportsPage";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
afterEach(() => { vi.unstubAllGlobals(); vi.restoreAllMocks(); });

it("exports the displayed report rows as a dated CSV", async () => {
  vi.stubGlobal("__TAURI_INTERNALS__", {});
  vi.mocked(invoke).mockResolvedValue([{
    date: "2026-11-16", issued: 3, returned: 2, lost: 0, renewed: 1, reservations: 4,
  }]);
  const createObjectURL = vi.fn((_blob: Blob) => "blob:test");
  vi.stubGlobal("URL", Object.assign(class extends URL {}, { createObjectURL, revokeObjectURL: vi.fn() }));
  const click = vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(function (this: HTMLAnchorElement) {
    expect(this.download).toMatch(/^circulation-\d{4}-\d{2}-\d{2}-to-\d{4}-\d{2}-\d{2}\.csv$/);
  });
  render(
    <QueryClientProvider client={new QueryClient()}>
      <SessionContext.Provider value={{
        session: { userId: 1, username: "viewer", displayName: "Viewer", role: "viewer", permissions: ["reports.read"] },
        signOut: async () => {},
      }}>
        <ReportsPage />
      </SessionContext.Provider>
    </QueryClientProvider>,
  );
  expect(await screen.findByRole("rowheader", { name: "2026-11-16" })).toBeInTheDocument();
  await userEvent.click(screen.getByRole("button", { name: "Export CSV" }));
  expect(click).toHaveBeenCalledOnce();
  const blob = createObjectURL.mock.calls[0]?.[0];
  if (!blob) throw new Error("CSV export did not create a file.");
  const csv = await new Promise<string>((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result));
    reader.onerror = reject;
    reader.readAsText(blob);
  });
  expect(csv).toBe("Date,Issued,Returned,Closed as lost,Renewed,Reservations placed\r\n2026-11-16,3,2,0,1,4\r\n");
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
