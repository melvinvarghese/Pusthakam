import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, within } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { MemoryRouter } from "react-router-dom";
import { afterEach, expect, it, vi } from "vitest";
import { SessionContext } from "@/lib/session";
import { Dashboard } from "./Dashboard";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
afterEach(() => { vi.unstubAllGlobals(); vi.mocked(invoke).mockReset(); });

function renderDashboard(error = false) {
  vi.stubGlobal("__TAURI_INTERNALS__", {});
  vi.mocked(invoke).mockImplementation(async (command) => {
    if (command === "app_info") return { version: "0.1.0", schemaVersion: 15, dataDir: "test", databasePath: "test" };
    if (error) throw { code: "DATABASE", message: "Could not read library counts." };
    return { catalogue: { titles: 4, copies: 8, available: 5 }, activeMembers: 12,
      circulation: { onLoan: 2, dueToday: 0, overdue: 1, readyHolds: 1, outstandingFines: 3 } };
  });
  render(
    <QueryClientProvider client={new QueryClient()}>
      <SessionContext.Provider value={{
        session: { userId: 1, username: "admin", displayName: "Administrator", role: "administrator", permissions: [] },
        signOut: async () => {},
      }}>
        <MemoryRouter><Dashboard /></MemoryRouter>
      </SessionContext.Provider>
    </QueryClientProvider>,
  );
}

it("shows database-backed stock, membership and circulation counts", async () => {
  renderDashboard();
  const counts = within(await screen.findByRole("region", { name: "Library at a glance" }));
  const value = (label: string) => counts.getByText(label).nextElementSibling;
  expect(value("Available copies")).toHaveTextContent("5");
  expect(value("Active members")).toHaveTextContent("12");
  expect(value("Holds ready")).toHaveTextContent("1");
  expect(value("Outstanding fines")).toHaveTextContent("3");
});

it("shows a query error instead of fabricating counts", async () => {
  renderDashboard(true);
  expect(await screen.findByRole("alert")).toHaveTextContent("Could not read library counts.");
  expect(screen.queryByRole("region", { name: "Library at a glance" })).not.toBeInTheDocument();
});
