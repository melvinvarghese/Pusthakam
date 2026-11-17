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
});
