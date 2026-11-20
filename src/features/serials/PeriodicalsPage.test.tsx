import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import { afterEach, expect, it, vi } from "vitest";
import { SessionContext } from "@/lib/session";
import PeriodicalsPage from "./PeriodicalsPage";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
afterEach(() => {
  vi.unstubAllGlobals();
  vi.mocked(invoke).mockReset();
});

function renderWithClient(permissions = ["serials.read", "serials.write"]) {
  return render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <SessionContext.Provider
        value={{
          session: {
            userId: 1,
            username: "admin",
            displayName: "Administrator",
            role: "administrator",
            permissions,
          },
          signOut: async () => {},
        }}
      >
        <PeriodicalsPage />
      </SessionContext.Provider>
    </QueryClientProvider>
  );
}

it("renders subscriptions and handles subscription creation", async () => {
  vi.stubGlobal("__TAURI_INTERNALS__", {});
  const user = userEvent.setup();

  const mockSubs = [
    {
      id: 1,
      title: "Nature",
      issn: "0028-0836",
      publisher: "Springer Nature",
      frequency: "weekly",
      vendorId: null,
      vendorName: null,
      budgetId: null,
      budgetName: null,
      costMinor: 240000,
      currency: "INR",
      startDate: "2026-01-01",
      endDate: "2026-12-31",
      status: "active",
      notes: "Main library subscription",
      archived: false,
      totalIssuesCount: 52,
      receivedIssuesCount: 4,
      missingIssuesCount: 1,
      expectedIssuesCount: 47,
      createdAt: "2026-11-20T08:00:00Z",
    },
  ];

  vi.mocked(invoke).mockImplementation(async (command, _args: unknown) => {
    if (command === "config_overview") return { library: { currency: "INR" } };
    if (command === "serials_list_subscriptions") return mockSubs;
    if (command === "serials_save_subscription") return 2;
    throw new Error(`Unexpected command: ${command}`);
  });

  renderWithClient();

  expect(await screen.findByText("Nature")).toBeInTheDocument();
  expect(screen.getByText("Springer Nature")).toBeInTheDocument();
  expect(screen.getByText("weekly")).toBeInTheDocument();
  expect(screen.getByText("4")).toBeInTheDocument();
  expect(screen.getByText("(1 missing)")).toBeInTheDocument();

  // Open New Subscription modal
  const newBtn = screen.getByRole("button", { name: "New Subscription" });
  await user.click(newBtn);

  expect(await screen.findByRole("dialog")).toBeInTheDocument();
  expect(screen.getByRole("heading", { name: "New Periodical Subscription" })).toBeInTheDocument();

  const titleInput = screen.getByLabelText("Publication Title");
  await user.type(titleInput, "Science");

  const saveBtn = screen.getByRole("button", { name: "Save Subscription" });
  await user.click(saveBtn);

  expect(invoke).toHaveBeenCalledWith("serials_save_subscription", {
    input: expect.objectContaining({
      title: "Science",
      frequency: "monthly",
    }),
  });
});

it("views issues and handles check-in workflow", async () => {
  vi.stubGlobal("__TAURI_INTERNALS__", {});
  const user = userEvent.setup();

  const mockSubs = [
    {
      id: 1,
      title: "National Geographic",
      issn: "0027-9358",
      publisher: "NGS",
      frequency: "monthly",
      vendorId: null,
      vendorName: null,
      budgetId: null,
      budgetName: null,
      costMinor: 120000,
      currency: "INR",
      startDate: "2026-01-01",
      endDate: "2026-12-31",
      status: "active",
      notes: null,
      archived: false,
      totalIssuesCount: 12,
      receivedIssuesCount: 0,
      missingIssuesCount: 0,
      expectedIssuesCount: 12,
      createdAt: "2026-11-20T08:00:00Z",
    },
  ];

  const mockIssues = [
    {
      id: 101,
      subscriptionId: 1,
      subscriptionTitle: "National Geographic",
      issueNumber: "Vol 249 No 1",
      expectedDate: "2026-01-01",
      receivedDate: null,
      status: "expected",
      barcode: null,
      accessionNo: null,
      physicalItemId: null,
      claimCount: 0,
      claimDate: null,
      notes: null,
      receivedBy: null,
      createdAt: "2026-11-20T08:00:00Z",
    },
  ];

  vi.mocked(invoke).mockImplementation(async (command, _args: unknown) => {
    if (command === "config_overview") {
      return {
        library: { currency: "INR" },
        itemCategories: [{ id: 1, name: "Periodical", archived: false }],
        locations: [{ id: 1, name: "Reading Room", archived: false }],
      };
    }
    if (command === "serials_list_subscriptions") return mockSubs;
    if (command === "serials_list_issues") return mockIssues;
    if (command === "serials_check_in_issue") {
      return {
        issueId: 101,
        status: "received",
        receivedDate: "2026-01-02",
        accessionNo: "PER-0001",
        barcode: "PER-0001",
        copyId: 50,
      };
    }
    throw new Error(`Unexpected command: ${command}`);
  });

  renderWithClient();

  // Switch to Issue Check-In tab
  const issuesTab = screen.getByRole("tab", { name: "Issue Check-In" });
  await user.click(issuesTab);

  expect(await screen.findByText("Vol 249 No 1")).toBeInTheDocument();
  expect(screen.getByText("National Geographic")).toBeInTheDocument();

  // Click Check In button
  const checkInBtn = screen.getByRole("button", { name: "Check In" });
  await user.click(checkInBtn);

  expect(await screen.findByRole("dialog")).toBeInTheDocument();
  expect(screen.getByRole("heading", { name: "Check In Issue: Vol 249 No 1" })).toBeInTheDocument();

  // Check accession copy
  const accessionCheckbox = screen.getByLabelText(/Add copy to Accession Register/);
  await user.click(accessionCheckbox);

  const confirmBtn = screen.getByRole("button", { name: "Confirm Check In" });
  await user.click(confirmBtn);

  expect(invoke).toHaveBeenCalledWith("serials_check_in_issue", {
    input: expect.objectContaining({
      issueId: 101,
      accessionCopy: true,
    }),
  });

  expect(await screen.findByText("Issue checked in successfully!")).toBeInTheDocument();
  expect(screen.getByText("Accession No: PER-0001")).toBeInTheDocument();
});

it("navigates to missing and claims tab and records a claim", async () => {
  vi.stubGlobal("__TAURI_INTERNALS__", {});
  const user = userEvent.setup();

  const mockMissing = [
    {
      id: 202,
      subscriptionId: 1,
      subscriptionTitle: "The Hindu",
      issueNumber: "Vol. 148 No. 270",
      expectedDate: "2026-11-18",
      receivedDate: null,
      status: "missing",
      barcode: null,
      accessionNo: null,
      physicalItemId: null,
      claimCount: 0,
      claimDate: null,
      notes: null,
      receivedBy: null,
      createdAt: "2026-11-20T08:00:00Z",
    },
  ];

  vi.mocked(invoke).mockImplementation(async (command, _args: unknown) => {
    if (command === "config_overview") return { library: { currency: "INR" } };
    if (command === "serials_list_subscriptions") return [];
    if (command === "serials_list_missing_issues") return mockMissing;
    if (command === "serials_update_issue_status") return undefined;
    throw new Error(`Unexpected command: ${command}`);
  });

  renderWithClient();

  // Switch to Missing & Claims tab
  const missingTab = screen.getByRole("tab", { name: "Missing & Claims" });
  await user.click(missingTab);

  expect(await screen.findByText("The Hindu")).toBeInTheDocument();
  expect(screen.getByText("Vol. 148 No. 270")).toBeInTheDocument();
  expect(screen.getByText("2026-11-18")).toBeInTheDocument();
  expect(screen.getByText("None")).toBeInTheDocument();

  // Click Send Claim
  const claimBtn = screen.getByRole("button", { name: "Send Claim" });
  await user.click(claimBtn);

  expect(invoke).toHaveBeenCalledWith("serials_update_issue_status", {
    issueId: 202,
    status: "claimed",
  });
});

it("displays due renewal reminders and views renewal history modal", async () => {
  vi.stubGlobal("__TAURI_INTERNALS__", {});
  const user = userEvent.setup();

  const mockSub = {
    id: 10,
    title: "IEEE Software",
    issn: "0740-7459",
    publisher: "IEEE",
    frequency: "bimonthly",
    vendorId: null,
    vendorName: null,
    budgetId: null,
    budgetName: null,
    costMinor: 35000,
    currency: "INR",
    startDate: "2026-01-01",
    endDate: "2026-11-25",
    status: "active",
    notes: null,
    archived: false,
    totalIssuesCount: 6,
    receivedIssuesCount: 5,
    missingIssuesCount: 0,
    expectedIssuesCount: 1,
    createdAt: "2026-01-01T00:00:00Z",
  };

  const mockRenewals = [
    {
      id: 1,
      subscriptionId: 10,
      startDate: "2026-01-01",
      endDate: "2026-12-31",
      costMinor: 35000,
      currency: "INR",
      budgetId: null,
      budgetName: null,
      notes: "Yearly renewal term",
      createdAt: "2026-01-02T10:00:00Z",
    },
  ];

  vi.mocked(invoke).mockImplementation(async (command, _args: unknown) => {
    if (command === "config_overview") return { library: { currency: "INR" } };
    if (command === "serials_list_subscriptions") return [mockSub];
    if (command === "serials_list_expiring") return [{ subscription: mockSub, daysRemaining: 5, isOverdue: false }];
    if (command === "serials_list_renewals") return mockRenewals;
    throw new Error(`Unexpected command: ${command}`);
  });

  renderWithClient();

  // Due reminder banner should be rendered
  expect(await screen.findByRole("alert")).toBeInTheDocument();
  expect(screen.getByText(/Due for Renewal:/)).toBeInTheDocument();
  expect(screen.getByText(/1 subscription is expiring within 30 days/)).toBeInTheDocument();

  // Open renewal history modal
  const renewalsBtn = screen.getByRole("button", { name: "Renewals" });
  await user.click(renewalsBtn);

  expect(await screen.findByRole("heading", { name: /Renewal History: IEEE Software/ })).toBeInTheDocument();
  expect(screen.getByText("Yearly renewal term")).toBeInTheDocument();
  expect(screen.getByText("2026-01-01 to 2026-12-31")).toBeInTheDocument();

  // Close modal
  const closeBtn = screen.getByRole("button", { name: "Close" });
  await user.click(closeBtn);
  expect(screen.queryByRole("heading", { name: /Renewal History: IEEE Software/ })).not.toBeInTheDocument();
});

it("views expenditure and renewals tab with date range reconciliation", async () => {
  vi.stubGlobal("__TAURI_INTERNALS__", {});
  const user = userEvent.setup();

  const mockSummary = {
    fromDate: "2026-01-01",
    toDate: "2026-12-31",
    totalSpendMinor: 85000,
    currency: "INR",
    itemCount: 2,
    items: [
      {
        id: 1,
        subscriptionId: 10,
        title: "ACM Computing Surveys",
        issn: "0360-0300",
        itemType: "initial",
        date: "2026-01-01",
        startDate: "2026-01-01",
        endDate: "2026-12-31",
        costMinor: 50000,
        currency: "INR",
        budgetId: null,
        budgetName: null,
        notes: null,
      },
      {
        id: 2,
        subscriptionId: 11,
        title: "IEEE Software",
        issn: "0740-7459",
        itemType: "renewal",
        date: "2026-06-01",
        startDate: "2026-06-01",
        endDate: "2027-05-31",
        costMinor: 35000,
        currency: "INR",
        budgetId: null,
        budgetName: null,
        notes: "Term renewal",
      },
    ],
  };

  vi.mocked(invoke).mockImplementation(async (command, _args: unknown) => {
    if (command === "config_overview") return { library: { currency: "INR" } };
    if (command === "serials_list_subscriptions") return [];
    if (command === "serials_list_expiring") return [];
    if (command === "serials_get_expenditure") return mockSummary;
    throw new Error(`Unexpected command: ${command}`);
  });

  renderWithClient();

  // Switch to Expenditure & Renewals tab
  const expTab = screen.getByRole("tab", { name: "Expenditure & Renewals" });
  await user.click(expTab);

  expect(await screen.findByRole("heading", { name: "Spending Reconciliation Ledger" })).toBeInTheDocument();
  expect(screen.getByText("Total Serials Expenditure")).toBeInTheDocument();
  expect(screen.getByText("New Subscriptions Spend")).toBeInTheDocument();
  expect(screen.getByText("Renewals Spend")).toBeInTheDocument();

  // Check ledger items
  expect(screen.getByText("ACM Computing Surveys")).toBeInTheDocument();
  expect(screen.getByText("Initial Sub")).toBeInTheDocument();
  expect(screen.getByText("IEEE Software")).toBeInTheDocument();
  expect(screen.getByText("Renewal")).toBeInTheDocument();
  expect(screen.getByText("Term renewal")).toBeInTheDocument();
});
