import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import { afterEach, expect, it, vi } from "vitest";
import { SessionContext } from "@/lib/session";
import { InventoryPage } from "./InventoryPage";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
afterEach(() => { vi.unstubAllGlobals(); vi.mocked(invoke).mockReset(); });

it("resumes a stock check and records a scanner submission", async () => {
  vi.stubGlobal("__TAURI_INTERNALS__", {});
  const check = { id: 8, location: "Main Library", shelf: "A1", startedAt: "2026-11-17T12:00:00Z",
    closedAt: null, expected: 1, matched: 0, unexpected: 0 };
  let scanned = false;
  vi.mocked(invoke).mockImplementation(async (command) => {
    if (command === "stock_check_list") return { rows: [check], total: 1, page: 1, pageSize: 25 };
    if (command === "stock_check_scan") { scanned = true; return "matched"; }
    if (command === "stock_check_review") return {
      check: { ...check, matched: Number(scanned) },
      entries: { rows: scanned ? [] : [{ barcode: "COPY-8", accessionNo: "008", title: "Shelf fixture" }],
        page: 1, pageSize: 25, total: Number(!scanned) },
    };
    throw new Error(`Unexpected command: ${command}`);
  });
  render(<QueryClientProvider client={new QueryClient()}>
    <SessionContext.Provider value={{
      session: { userId: 1, username: "admin", displayName: "Administrator", role: "administrator",
        permissions: ["catalogue.read", "catalogue.write"] },
      signOut: async () => {},
    }}><InventoryPage /></SessionContext.Provider>
  </QueryClientProvider>);
  const user = userEvent.setup();
  await user.click(await screen.findByRole("button", { name: "Open check" }));
  expect(await screen.findByText("Shelf fixture")).toBeInTheDocument();
  const field = screen.getByRole("textbox", { name: "Scan barcode" });
  await user.type(field, "COPY-8{Enter}");
  expect(await screen.findByText("Matched an expected copy.")).toBeInTheDocument();
  expect(await screen.findByRole("button", { name: "Missing (0)" })).toBeInTheDocument();
  expect(invoke).toHaveBeenCalledWith("stock_check_scan", { id: 8, barcode: "COPY-8" });
  expect(field).toHaveFocus();
});
