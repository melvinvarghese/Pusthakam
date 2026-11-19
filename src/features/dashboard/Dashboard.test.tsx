import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, within } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { MemoryRouter } from "react-router-dom";
import { afterEach, expect, it, vi } from "vitest";
import { SessionContext } from "@/lib/session";
import { Dashboard } from "./Dashboard";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
afterEach(() => { vi.unstubAllGlobals(); vi.mocked(invoke).mockReset(); });

function renderDashboard({
  error = false,
  permissions = ["reports.read"],
  circulation = { onLoan: 2, dueToday: 1, overdue: 2, readyHolds: 3, outstandingFines: 3 },
} = {}) {
  vi.stubGlobal("__TAURI_INTERNALS__", {});
  vi.mocked(invoke).mockImplementation(async (command) => {
    if (command === "app_info") return { version: "0.1.0", schemaVersion: 20, dataDir: "test", databasePath: "test" };
    if (error) throw { code: "DATABASE", message: "Could not read library counts." };
    if (command === "dashboard_overview") {
      return {
        catalogue: { titles: 4, copies: 8, available: 5 },
        activeMembers: 12,
        circulation,
      };
    }
    if (command === "reports_circulation_activity") {
      return [
        { date: "2026-11-18", issued: 4, returned: 2, lost: 0, renewed: 1, reservations: 1 },
        { date: "2026-11-19", issued: 6, returned: 3, lost: 1, renewed: 2, reservations: 2 },
      ];
    }
    if (command === "reports_popular_titles") {
      return [
        { recordId: 1, title: "The Lord of the Rings", archived: false, issues: 7 },
        { recordId: 2, title: "1984", archived: false, issues: 4 },
      ];
    }
    if (command === "reports_catalogue_growth") {
      return [
        { date: "2026-11-18", titlesAdded: 1, copiesAdded: 2 },
        { date: "2026-11-19", titlesAdded: 2, copiesAdded: 3 },
      ];
    }
    throw new Error(`Unexpected command: ${command}`);
  });

  render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <SessionContext.Provider value={{
        session: { userId: 1, username: "admin", displayName: "Administrator", role: "administrator", permissions },
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
  expect(value("Holds ready")).toHaveTextContent("3");
  expect(value("Outstanding fines")).toHaveTextContent("3");
});

it("shows a query error instead of fabricating counts", async () => {
  renderDashboard({ error: true });
  expect(await screen.findByRole("alert")).toHaveTextContent("Could not read library counts.");
  expect(screen.queryByRole("region", { name: "Library at a glance" })).not.toBeInTheDocument();
});

it("shows actionable alerts for overdue items, ready holds, and due today loans with circulation links", async () => {
  renderDashboard({
    circulation: { onLoan: 5, dueToday: 2, overdue: 3, readyHolds: 4, outstandingFines: 1 },
  });

  const alerts = await screen.findByRole("region", { name: "Actionable alerts" });
  expect(within(alerts).getByText(/3 loans are overdue/i)).toBeInTheDocument();
  expect(within(alerts).getByRole("link", { name: /review overdue loans/i })).toHaveAttribute("href", "/circulation");

  expect(within(alerts).getByText(/4 holds are ready for pickup/i)).toBeInTheDocument();
  expect(within(alerts).getByRole("link", { name: /view reservations queue/i })).toHaveAttribute("href", "/circulation");

  expect(within(alerts).getByText(/2 loans are due today/i)).toBeInTheDocument();
  expect(within(alerts).getByRole("link", { name: /open circulation desk/i })).toHaveAttribute("href", "/circulation");
});

it("displays activity, growth and top titles reconciled with date range", async () => {
  renderDashboard();
  const trends = within(await screen.findByRole("region", { name: "Activity & growth trends" }));
  await trends.findByText("The Lord of the Rings");

  // Issues: 4 + 6 = 10, Returns: 2 + 3 = 5, Renewals: 1 + 2 = 3
  const value = (label: string) => trends.getByText(label).nextElementSibling;
  expect(value("Issues")).toHaveTextContent("10");
  expect(value("Returns")).toHaveTextContent("5");
  expect(value("Renewals")).toHaveTextContent("3");

  // Titles: 1 + 2 = 3, Copies: 2 + 3 = 5
  expect(value("New titles")).toHaveTextContent("3");
  expect(value("New copies")).toHaveTextContent("5");

  // Top borrowed titles
  expect(trends.getByText("The Lord of the Rings")).toBeInTheDocument();
  expect(trends.getByText("7 borrows")).toBeInTheDocument();
  expect(trends.getByText("1984")).toBeInTheDocument();
  expect(trends.getByText("4 borrows")).toBeInTheDocument();
});

it("updates date range preset and accepts custom date filters", async () => {
  renderDashboard();
  const trends = within(await screen.findByRole("region", { name: "Activity & growth trends" }));

  // Switch to 7d preset
  fireEvent.click(trends.getByRole("button", { name: "7d" }));
  expect(trends.getByRole("button", { name: "7d" })).toHaveClass("bg-accent");

  // Switch to custom preset and edit dates
  fireEvent.click(trends.getByRole("button", { name: "Custom" }));
  const startDate = trends.getByLabelText("Start date");
  const endDate = trends.getByLabelText("End date");
  expect(startDate).toBeInTheDocument();
  expect(endDate).toBeInTheDocument();

  fireEvent.change(startDate, { target: { value: "2026-11-01" } });
  fireEvent.change(endDate, { target: { value: "2026-11-15" } });
  expect(trends.getByText(/Summary for 2026-11-01 to 2026-11-15/i)).toBeInTheDocument();
});
