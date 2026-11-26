import { act } from "@testing-library/react";
import { setLanguage } from "@/lib/language";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import { afterEach, expect, it, vi } from "vitest";
import { SessionContext } from "@/lib/session";
import { MembersPage } from "./MembersPage";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
afterEach(() => {
  setLanguage("en");
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
  vi.mocked(invoke).mockReset();
});

it("keeps member data and unsaved registration fields when changing language", async () => {
  vi.stubGlobal("__TAURI_INTERNALS__", {});
  vi.mocked(invoke).mockImplementation(async command => {
    if (command === "config_overview") return { memberCategories: [{ id: 1, name: "Students", archived: false }] } as never;
    return { page: 1, pageSize: 25, total: 1, rows: [{ id: 4, name: "Save", memberNumber: "0001", categoryName: "Students", expiresOn: "2027-01-01" }] } as never;
  });
  render(<QueryClientProvider client={new QueryClient()}>
    <SessionContext.Provider value={{ session: { userId: 1, username: "staff", displayName: "Staff", role: "librarian", permissions: ["members.read", "members.write"] }, signOut: async () => {} }}>
      <MembersPage />
    </SessionContext.Provider>
  </QueryClientProvider>);
  await screen.findByText("Save");
  await userEvent.click(screen.getByRole("button", { name: "Register member" }));
  await userEvent.type(screen.getByRole("textbox", { name: "Name" }), "Catalogue");
  act(() => setLanguage("ml"));
  const dialog = within(screen.getByRole("dialog", { name: "അംഗത്തെ ചേർക്കുക" }));
  expect(dialog.getByRole("textbox", { name: "പേര്" })).toHaveValue("Catalogue");
  expect(dialog.getByRole("option", { name: "Students" })).toBeInTheDocument();
  await userEvent.click(dialog.getByRole("button", { name: "റദ്ദാക്കുക" }));
  expect(screen.getByText("Save")).toBeInTheDocument();
  expect(screen.getByText("0001")).toBeInTheDocument();
});

it("opens loan history for an archived member with read-only access", async () => {
  vi.stubGlobal("__TAURI_INTERNALS__", {});
  const page = { page: 1, pageSize: 25 };
  vi.mocked(invoke)
    .mockResolvedValueOnce({ ...page, total: 0, rows: [] })
    .mockResolvedValueOnce({ ...page, total: 1, rows: [{
      id: 4, name: "Asha Nair", memberNumber: "MEM-101", categoryName: "Student", expiresOn: "2026-10-30",
    }] })
    .mockResolvedValueOnce({ ...page, total: 1, rows: [{
      id: 9, title: "Balyakalasakhi", barcode: "LIB-00101", issuedOn: "2026-10-10",
      dueOn: "2026-10-20", returnedOn: "2026-10-22", overdue: false, importSource: "Old library", sourceStaff: "Former librarian", renewalCount: 2,
    }] });

  render(
    <QueryClientProvider client={new QueryClient()}>
      <SessionContext.Provider value={{
        session: { userId: 1, username: "viewer", displayName: "Viewer", role: "viewer", permissions: ["members.read"] },
        signOut: async () => {},
      }}>
        <MembersPage />
      </SessionContext.Provider>
    </QueryClientProvider>,
  );

  const user = userEvent.setup();
  await screen.findByText("No members registered yet.");
  await user.selectOptions(screen.getByRole("combobox", { name: "Status" }), "archived");
  await user.click(await screen.findByRole("button", { name: "Loans for Asha Nair (MEM-101)" }));
  const dialog = within(screen.getByRole("dialog", { name: "Loans for Asha Nair" }));
  expect(await dialog.findByText("Balyakalasakhi")).toBeInTheDocument();
  expect(dialog.getByText("Returned 2026-10-22")).toBeInTheDocument();
  expect(dialog.getByText("Imported from Old library. Source staff label: Former librarian.")).toBeInTheDocument();
  expect(dialog.getByText("Renewals: 2")).toBeInTheDocument();
  expect(invoke).toHaveBeenLastCalledWith("members_list_loans", { memberId: 4, page: 1 });
});

it("previews a member card and opens the system print action", async () => {
  vi.stubGlobal("__TAURI_INTERNALS__", {});
  const print = vi.spyOn(window, "print").mockImplementation(() => {});
  vi.mocked(invoke).mockImplementation(async (command) => {
    if (command === "config_overview") {
      return { library: { name: "Town Library" }, memberCategories: [] } as never;
    }
    if (command === "members_get_photo") return null as never;
    return { page: 1, pageSize: 25, total: 1, rows: [{
      id: 4, name: "Asha Nair", memberNumber: "MEM-101", categoryName: "Student", expiresOn: "2026-12-30",
    }] } as never;
  });
  render(
    <QueryClientProvider client={new QueryClient()}>
      <SessionContext.Provider value={{
        session: { userId: 2, username: "staff", displayName: "Staff", role: "librarian", permissions: ["members.read", "members.write"] },
        signOut: async () => {},
      }}>
        <MembersPage />
      </SessionContext.Provider>
    </QueryClientProvider>,
  );
  const user = userEvent.setup();
  await user.click(await screen.findByRole("button", { name: "Card for Asha Nair (MEM-101)" }));
  const card = within(screen.getByRole("dialog", { name: "Member card" }));
  expect(card.getByText("Town Library")).toBeInTheDocument();
  expect(card.getByRole("img", { name: "Barcode MEM-101" })).toBeInTheDocument();
  const printButton = card.getByRole("button", { name: "Print card" });
  await waitFor(() => expect(printButton).toBeEnabled());
  await user.click(printButton);
  expect(print).toHaveBeenCalledOnce();
});

it("requires category mapping before committing a Koha patron CSV", async () => {
  vi.stubGlobal("__TAURI_INTERNALS__", {});
  vi.mocked(invoke).mockImplementation(async (command) => {
    switch (command) {
      case "config_overview": return {
        library: { name: "Town Library" },
        memberCategories: [{ id: 3, name: "Student", archived: false }],
      } as never;
      case "members_preview_koha": return {
        ready: 1, skipped: 1, categories: { STU: 1 },
        sample: [{ memberNumber: "0001", name: "Asha Nair", categoryCode: "STU", expiresOn: null }],
        problems: ["Line 3: Card number already exists."],
      } as never;
      case "members_import_koha": return 1 as never;
      default: return { page: 1, pageSize: 25, total: 0, rows: [] } as never;
    }
  });
  render(
    <QueryClientProvider client={new QueryClient()}>
      <SessionContext.Provider value={{
        session: { userId: 2, username: "staff", displayName: "Staff", role: "librarian", permissions: ["members.read", "members.write"] },
        signOut: async () => {},
      }}>
        <MembersPage />
      </SessionContext.Provider>
    </QueryClientProvider>,
  );
  const user = userEvent.setup();
  const open = await screen.findByRole("button", { name: "Import Koha patrons" });
  await user.click(open);
  const dialog = within(await screen.findByRole("dialog", { name: "Import Koha patrons" }, { timeout: 5000 }));
  const csv = "cardnumber,surname,categorycode\n0001,Nair,STU";
  await user.upload(dialog.getByLabelText("Koha patron CSV"), new File([csv], "patrons.csv", { type: "text/csv" }));
  await user.click(dialog.getByRole("button", { name: "Preview CSV" }));
  const importButton = await dialog.findByRole("button", { name: "Import members" });
  expect(importButton).toBeDisabled();
  await user.selectOptions(dialog.getByRole("combobox", { name: "STU (1)" }), "3");
  await user.click(importButton);
  expect(await dialog.findByText("Imported 1 member.")).toBeInTheDocument();
  expect(invoke).toHaveBeenCalledWith("members_import_koha", {
    dataBase64: btoa(csv), mappings: { STU: 3 }, expectedReady: 1,
  });
});

it("opens an archived member fine account using member-read permission", async () => {
  vi.stubGlobal("__TAURI_INTERNALS__", {});
  vi.mocked(invoke).mockImplementation(async (command) => {
    if (command === "members_fine_account") return { rows: [], balances: [], page: 1, pageSize: 25, total: 0 };
    return { rows: [{ id: 4, name: "Asha Nair", memberNumber: "MEM-101", categoryName: "Student", expiresOn: "2026-01-01" }], total: 1, page: 1, pageSize: 25 };
  });
  render(<QueryClientProvider client={new QueryClient()}><SessionContext.Provider value={{ session: { userId: 1, username: "viewer", displayName: "Viewer", role: "viewer", permissions: ["members.read"] }, signOut: async () => {} }}><MembersPage /></SessionContext.Provider></QueryClientProvider>);
  const user=userEvent.setup();
  await user.selectOptions(screen.getByRole("combobox", {name:"Status"}),"archived");
  await user.click(await screen.findByRole("button", {name:"Fines for Asha Nair (MEM-101)"}));
  await screen.findByText("This member has no assessed fines.");
  expect(invoke).toHaveBeenCalledWith("members_fine_account",{memberId:4,page:1});
  expect(screen.queryByRole("button",{name:"Register member"})).not.toBeInTheDocument();
});
