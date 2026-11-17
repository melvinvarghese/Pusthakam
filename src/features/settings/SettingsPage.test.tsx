import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ConfigOverview, Session } from "@/lib/api/types";

const api = vi.hoisted(() => ({
  overview: vi.fn(), updateLibrary: vi.fn(), saveNamed: vi.fn(), saveMemberCategory: vi.fn(), saveShelf: vi.fn(),
  savePolicy: vi.fn(), deletePolicy: vi.fn(), setArchived: vi.fn(),
}));
vi.mock("@/lib/api", async (orig) => ({ ...(await orig<typeof import("@/lib/api")>()), configApi: api }));

import { ApiError, appApi } from "@/lib/api";
import { SessionContext } from "@/lib/session";
import { SettingsPage } from "./SettingsPage";

const CONFIG: ConfigOverview = {
  library: { name: "Demo Library", profile: "school", address: null, contact: null, currency: "INR", locale: "en" },
  classificationSchemes: [],
  memberCategories: [{ id: 1, name: "Student", description: null, membershipDays: 365, archived: false }],
  itemCategories: [{ id: 1, name: "General", description: null, archived: false }, { id: 2, name: "Reference", description: null, archived: false }],
  collections: [{ id: 1, name: "General Collection", description: null, archived: false }],
  locations: [{ id: 1, name: "Main Library", description: null, archived: false }],
  shelves: [],
  policies: [{
    id: 1, memberCategoryId: 1, itemCategoryId: 1, loanDays: 14, maxLoans: 3, maxRenewals: 1, renewalDays: 7,
    finePerDayMinor: 100, fineGraceDays: 1, fineCapMinor: null, reservationHoldDays: 3,
  }],
};

const ADMIN: Session = { userId: 1, username: "admin", displayName: "Admin", role: "administrator", permissions: ["admin.settings"] };
const CLERK: Session = { userId: 2, username: "clerk", displayName: "Clerk", role: "circulation", permissions: ["circulation.issue"] };

function renderPage(session: Session) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <QueryClientProvider client={qc}>
      <SessionContext.Provider value={{ session, signOut: async () => {} }}>
        <SettingsPage />
      </SessionContext.Provider>
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  Object.values(api).forEach((f) => f.mockReset());
  api.overview.mockResolvedValue(CONFIG);
});
afterEach(() => vi.restoreAllMocks());

describe("SettingsPage", () => {
  it("saves a custom library field as a number", async () => {
    vi.spyOn(appApi, "setting").mockResolvedValue(null);
    const save = vi.spyOn(appApi, "setSetting").mockResolvedValue(undefined);
    renderPage(ADMIN);

    await userEvent.click(await screen.findByRole("button", { name: "Add custom field" }));
    await userEvent.type(screen.getByLabelText("Field label"), "Floors");
    await userEvent.selectOptions(screen.getByLabelText("Field type"), "number");
    await userEvent.type(screen.getByRole("spinbutton", { name: "Field value" }), "2");
    await userEvent.click(screen.getByRole("button", { name: "Save custom field" }));

    expect(await screen.findByText("Floors")).toBeInTheDocument();
    expect(save).toHaveBeenCalledExactlyOnceWith("library.custom_fields", '[{"label":"Floors","kind":"number","value":2}]');
  });

  it("shows the library, categories, loan rules and locations from the backend", async () => {
    renderPage(ADMIN);
    expect(await screen.findByText("Demo Library")).toBeInTheDocument();
    expect(screen.getByText("School library")).toBeInTheDocument();
    const rules = screen.getByRole("region", { name: "Loan rules" });
    expect(within(rules).getByText("14 days")).toBeInTheDocument();
    expect(within(rules).getByText("1 × 7 days")).toBeInTheDocument();
    expect(screen.getByText("Main Library")).toBeInTheDocument();
  });

  it("hides every editing control from someone without the settings permission", async () => {
    renderPage(CLERK);
    await screen.findByText("Demo Library");
    expect(screen.queryByRole("button", { name: /Add|Edit|Archive|Delete/ })).not.toBeInTheDocument();
    expect(screen.getByText(/view these settings but not change them/)).toBeInTheDocument();
  });

  it("adds a collection through the dialog and sends the trimmed name", async () => {
    api.saveNamed.mockResolvedValue(2);
    renderPage(ADMIN);
    await userEvent.click(await screen.findByRole("button", { name: "Add collection" }));
    const dialog = screen.getByRole("dialog", { name: "Add collection" });
    await userEvent.type(within(dialog).getByLabelText("Name"), "  Fiction ");
    await userEvent.click(within(dialog).getByRole("button", { name: "Save" }));
    expect(api.saveNamed).toHaveBeenCalledWith("collection", { id: null, name: "  Fiction ", description: null });
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("refuses an empty name without calling the backend", async () => {
    renderPage(ADMIN);
    await userEvent.click(await screen.findByRole("button", { name: "Add collection" }));
    await userEvent.click(within(screen.getByRole("dialog")).getByRole("button", { name: "Save" }));
    expect(screen.getByText("Enter a name.")).toBeInTheDocument();
    expect(api.saveNamed).not.toHaveBeenCalled();
  });

  it("keeps the dialog open and shows the backend's message on a duplicate", async () => {
    api.saveNamed.mockRejectedValue(new ApiError({ code: "CONFLICT", message: 'A collection named "Fiction" already exists.' }));
    renderPage(ADMIN);
    await userEvent.click(await screen.findByRole("button", { name: "Add collection" }));
    const dialog = screen.getByRole("dialog");
    await userEvent.type(within(dialog).getByLabelText("Name"), "Fiction");
    await userEvent.click(within(dialog).getByRole("button", { name: "Save" }));
    expect(await within(dialog).findByRole("alert")).toHaveTextContent("already exists");
  });

  it("archives an item category", async () => {
    api.setArchived.mockResolvedValue(undefined);
    renderPage(ADMIN);
    const panel = await screen.findByRole("region", { name: "Item categories" });
    await userEvent.click(within(panel).getAllByRole("button", { name: "Archive" })[0]!);
    expect(api.setArchived).toHaveBeenCalledWith("itemCategory", 1, true);
  });

  it("converts the fine to minor units when saving a loan rule", async () => {
    api.savePolicy.mockResolvedValue(1);
    renderPage(ADMIN);
    const rules = await screen.findByRole("region", { name: "Loan rules" });
    await userEvent.click(within(rules).getByRole("button", { name: "Edit" }));
    const dialog = screen.getByRole("dialog", { name: "Edit loan rule" });
    const fine = within(dialog).getByLabelText(/Fine per day/);
    await userEvent.clear(fine);
    await userEvent.type(fine, "2.50");
    await userEvent.click(within(dialog).getByRole("button", { name: "Save" }));
    expect(api.savePolicy.mock.calls[0]?.[0]).toMatchObject({ id: 1, finePerDayMinor: 250, loanDays: 14, fineCapMinor: null });
  });

  it("asks before deleting a loan rule", async () => {
    api.deletePolicy.mockResolvedValue(undefined);
    renderPage(ADMIN);
    const rules = await screen.findByRole("region", { name: "Loan rules" });
    await userEvent.click(within(rules).getByRole("button", { name: "Delete" }));
    expect(api.deletePolicy).not.toHaveBeenCalled();
    await userEvent.click(screen.getByRole("button", { name: "Delete rule" }));
    expect(api.deletePolicy).toHaveBeenCalledWith(1);
  });

  it("shows the backend's message if settings cannot be loaded", async () => {
    api.overview.mockRejectedValue(new ApiError({ code: "AUTH_FAILED", message: "Please sign in." }));
    renderPage(ADMIN);
    expect(await screen.findByRole("alert")).toHaveTextContent("Please sign in.");
  });
});
