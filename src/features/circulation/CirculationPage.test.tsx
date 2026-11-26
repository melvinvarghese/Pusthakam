import { act } from "@testing-library/react";
import { setLanguage } from "@/lib/language";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { SessionContext } from "@/lib/session";
import { circulationApi } from "@/lib/api";
import { CirculationPage } from "./CirculationPage";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
beforeEach(() => { vi.spyOn(circulationApi, "listOpenLoans").mockResolvedValue({rows:[],page:1,pageSize:25,total:0}); });

afterEach(() => {
  setLanguage("en");
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
  vi.mocked(invoke).mockReset();
});

function renderCirculation(admin = false, catalogue = false) {
  vi.stubGlobal("__TAURI_INTERNALS__", {});
  render(
    <QueryClientProvider client={new QueryClient()}>
      <SessionContext.Provider value={{
        session: { userId: 1, username: "clerk", displayName: "Clerk", role: "circulation", permissions: ["circulation.issue", "members.read", ...(catalogue ? ["catalogue.read"] : []), ...(admin ? ["admin.settings"] : [])] },
        signOut: async () => {},
      }}>
        <CirculationPage />
      </SessionContext.Provider>
    </QueryClientProvider>,
  );
}

it("renews in Malayalam without changing the barcode or confirmation payload", async () => {
  const preview = { loanId: 17, memberName: "Members", title: "Catalogue", previousDueOn: "2026-11-20", dueOn: "2026-11-27" };
  vi.mocked(invoke).mockResolvedValueOnce(preview).mockResolvedValueOnce(undefined);
  renderCirculation();
  await userEvent.click(screen.getByRole("button", { name: "Renew" }));
  await userEvent.type(screen.getByRole("textbox", { name: "Book barcode or accession number" }), "000123");
  act(() => setLanguage("ml"));
  expect(screen.getByRole("textbox", { name: "പുസ്തകത്തിന്റെ ബാർകോഡ് അല്ലെങ്കിൽ അക്സഷൻ നമ്പർ" })).toHaveValue("000123");
  await userEvent.click(screen.getByRole("button", { name: "നീട്ടേണ്ട കാലാവധി പരിശോധിക്കുക" }));
  const dialog = within(await screen.findByRole("dialog", { name: "കാലാവധി നീട്ടുന്നത് ഉറപ്പാക്കുക" }));
  expect(dialog.getByText("Catalogue")).toBeInTheDocument();
  expect(dialog.getByText("Members")).toBeInTheDocument();
  await userEvent.click(dialog.getByRole("button", { name: "കാലാവധി നീട്ടുക" }));
  expect(invoke).toHaveBeenCalledWith("circulation_prepare_renewal", { barcode: "000123" });
  expect(invoke).toHaveBeenLastCalledWith("circulation_renew", { confirmation: { loanId: 17, previousDueOn: "2026-11-20", dueOn: "2026-11-27" } });
  expect(await screen.findByRole("status")).toHaveTextContent("Catalogue എന്ന പുസ്തകത്തിന്റെ കാലാവധി നീട്ടി.");
  act(() => setLanguage("en"));
  expect(screen.getByRole("status")).toHaveTextContent("Renewed Catalogue. Now due 2026-11-27.");
});

it("returns keyboard focus to the member lookup button when dismissed", async () => {
  renderCirculation();
  const user = userEvent.setup();
  const button = screen.getByRole("button", { name: "Find member by name" });
  button.focus();
  await user.keyboard("{Enter}");
  expect(screen.getByRole("combobox", { name: "Member name" })).toHaveFocus();
  await user.keyboard("{Escape}");
  expect(button).toHaveFocus();
});

it("looks up a borrower by name and fills the selected member number", async () => {
  vi.mocked(invoke).mockResolvedValueOnce({
    rows: [{ id: 1, name: "Asha Nair", memberNumber: "MEM-1" }, { id: 2, name: "Asha Nair", memberNumber: "MEM-2" }],
    total: 2, page: 1, pageSize: 25,
  });
  renderCirculation();
  const user = userEvent.setup();
  await user.click(screen.getByRole("button", { name: "Find member by name" }));
  const dialog = within(screen.getByRole("dialog", { name: "Find member by name" }));
  await user.type(dialog.getByRole("combobox", { name: "Member name" }), "Asha");
  await user.click(await dialog.findByRole("option", { name: "Asha Nair (MEM-2)" }));
  expect(invoke).toHaveBeenCalledExactlyOnceWith("members_list", { search: "Asha", page: 1, archived: false });
  expect(screen.getByRole("textbox", { name: "Member number" })).toHaveValue("MEM-2");
});

it("reviews the old and new due dates before confirming a renewal", async () => {
  const preview = {
    loanId: 1,
    memberName: "Asha Nair",
    title: "Balyakalasakhi",
    previousDueOn: "2026-11-20",
    dueOn: "2026-11-27",
  };
  vi.mocked(invoke).mockResolvedValueOnce(preview).mockResolvedValueOnce(undefined)
    .mockResolvedValueOnce({ library: { name: "Town Library" } });
  renderCirculation();

  const user = userEvent.setup();
  await user.click(screen.getByRole("button", { name: "Renew" }));
  await user.type(screen.getByRole("textbox", { name: "Book barcode or accession number" }), "LIB-00101{Enter}");
  const dialog = within(await screen.findByRole("dialog", { name: "Confirm renewal" }));
  expect(dialog.getByText(preview.previousDueOn)).toBeInTheDocument();
  expect(dialog.getByText(preview.dueOn)).toBeInTheDocument();
  expect(invoke).toHaveBeenCalledExactlyOnceWith("circulation_prepare_renewal", { barcode: "LIB-00101" });

  await user.click(dialog.getByRole("button", { name: "Renew loan" }));
  expect(await screen.findByRole("status")).toHaveTextContent("Renewed Balyakalasakhi. Now due 2026-11-27.");
  expect(invoke).toHaveBeenLastCalledWith("circulation_renew", {
    confirmation: { loanId: preview.loanId, previousDueOn: preview.previousDueOn, dueOn: preview.dueOn },
  });
  const print = vi.spyOn(window, "print").mockImplementation(() => {});
  await user.click(screen.getByRole("button", { name: "View due slip" }));
  const slip = within(screen.getByRole("dialog", { name: "Loan due slip" }));
  expect(await slip.findByText("Town Library")).toBeInTheDocument();
  expect(slip.getByText(preview.dueOn)).toBeInTheDocument();
  await user.click(slip.getByRole("button", { name: "Print slip" }));
  expect(print).toHaveBeenCalledOnce();
});

it("refreshes the overdue list after a confirmed return", async () => {
  const page = { page: 1, pageSize: 25 };
  vi.mocked(invoke)
    .mockResolvedValueOnce({ ...page, total: 1, rows: [{
      loanId: 9, memberName: "Asha Nair", memberNumber: "MEM-101",
      title: "Balyakalasakhi", barcode: "LIB-00101", dueOn: "2026-11-13", daysOverdue: 3,
    }] })
    .mockResolvedValueOnce({
      loanId: 9, memberName: "Asha Nair", title: "Balyakalasakhi", dueOn: "2026-11-13",
      estimatedFine: { amountMinor: 250, currency: "INR" },
    })
    .mockResolvedValueOnce(undefined)
    .mockResolvedValueOnce({ ...page, total: 0, rows: [] });
  renderCirculation();

  const user = userEvent.setup();
  await user.click(screen.getByRole("button", { name: "Overdue" }));
  expect(await screen.findByText("Asha Nair")).toBeInTheDocument();
  await user.click(screen.getByRole("button", { name: "Return copy LIB-00101" }));
  const dialog = within(await screen.findByRole("dialog", { name: "Confirm return" }));
  expect(dialog.getByText(/Estimated overdue fine/)).toHaveTextContent("2.50");
  await user.click(dialog.getByRole("button", { name: "Return copy" }));
  expect(await screen.findByText("No overdue loans.")).toBeInTheDocument();
  expect(invoke).toHaveBeenNthCalledWith(2, "circulation_prepare_return", { barcode: "LIB-00101" });
  expect(invoke).toHaveBeenNthCalledWith(3, "circulation_return", { loanId: 9 });
  expect(invoke).toHaveBeenNthCalledWith(4, "circulation_list_overdue", { page: 1 });
});

it("places a barcode reservation and refreshes the queue", async () => {
  const page = { page: 1, pageSize: 25 };
  vi.mocked(invoke)
    .mockResolvedValueOnce({ ...page, total: 0, rows: [] })
    .mockResolvedValueOnce(undefined)
    .mockResolvedValueOnce({ ...page, total: 1, rows: [{
      id: 9, title: "Balyakalasakhi", barcode: "LIB-00101", memberName: "Asha Nair",
      memberNumber: "MEM-101", status: "pending", expiresAt: null,
    }] });
  renderCirculation();

  const user = userEvent.setup();
  await user.click(screen.getByRole("button", { name: "Reservations" }));
  expect(await screen.findByText("No active reservations.")).toBeInTheDocument();
  await user.type(screen.getByRole("textbox", { name: "Member number" }), "MEM-101{Enter}");
  expect(screen.getByRole("textbox", { name: "Book barcode or accession number" })).toHaveFocus();
  await user.keyboard("LIB-00101{Enter}");
  expect(await screen.findByText("Waiting")).toBeInTheDocument();
  expect(screen.getByText("Asha Nair")).toBeInTheDocument();
  expect(invoke).toHaveBeenNthCalledWith(2, "circulation_place_reservation", {
    memberNumber: "MEM-101", barcode: "LIB-00101",
  });
});

it("records a partial payment against the displayed balance", async () => {
  const fine = {
    id: 9, title: "Balyakalasakhi", memberName: "Asha Nair", memberNumber: "MEM-101",
    amountMinor: 250, paidMinor: 0, waivedMinor: 0, outstandingMinor: 250, currency: "INR",
  };
  vi.mocked(invoke)
    .mockResolvedValueOnce({ page: 1, pageSize: 25, total: 1, rows: [fine] })
    .mockResolvedValueOnce(undefined)
    .mockResolvedValueOnce({ page: 1, pageSize: 25, total: 1, rows: [{ ...fine, paidMinor: 125, outstandingMinor: 125 }] });
  renderCirculation();

  const user = userEvent.setup();
  await user.click(screen.getByRole("button", { name: "Fines" }));
  await user.click(await screen.findByRole("button", { name: "Record payment" }, { timeout: 5000 }));
  const dialog = within(screen.getByRole("dialog", { name: "Record payment" }));
  const amount = dialog.getByRole("textbox", { name: "Amount (INR)" });
  expect(amount).toHaveValue("2.50");
  await user.clear(amount);
  await user.type(amount, "1.25");
  await user.click(dialog.getByRole("button", { name: "Record payment" }));
  expect(await screen.findByText("Payment recorded.")).toBeInTheDocument();
  expect(invoke).toHaveBeenNthCalledWith(2, "circulation_settle_fine", {
    fineId: 9, amountMinor: 125, expectedBalance: 250, entry: { kind: "payment", method: "cash" },
  });
  expect(screen.queryByRole("button", { name: "Waive" })).not.toBeInTheDocument();
  vi.mocked(invoke).mockResolvedValueOnce({ page: 1, pageSize: 25, total: 1, rows: [{
    id: 17, kind: "payment", method: "cash", amountMinor: 125, staffName: "Clerk", recordedAt: "2026-11-17T03:00:00Z",
  }] }).mockResolvedValueOnce({ library: { name: "Main Library" } });
  await user.click(screen.getByRole("button", { name: "Fine history for MEM-101" }));
  await user.click(await screen.findByRole("button", { name: "Receipt #17" }));
  const receipt = within(screen.getByRole("dialog", { name: "Fine payment receipt" }));
  expect(await receipt.findByText("Main Library")).toBeInTheDocument();
  expect(receipt.getByText(/1\.25/)).toBeInTheDocument();
  expect(receipt.getByText("Cash")).toBeInTheDocument();
  const print = vi.spyOn(window, "print").mockImplementation(() => {});
  await user.click(receipt.getByRole("button", { name: "Print receipt" }));
  expect(print).toHaveBeenCalledOnce();
});

it("selects a reservation member from live name suggestions and uses the chosen number", async () => {
  vi.mocked(invoke).mockImplementation(async (command) => {
    if (command === "circulation_list_reservations") return { rows: [], total: 0, page: 1, pageSize: 25 };
    if (command === "members_list") return { rows: [{ id: 4, name: "Asha Nair", memberNumber: "MEM-004" }], total: 1, page: 1, pageSize: 25 };
    return undefined;
  });
  renderCirculation();
  const user = userEvent.setup();
  await user.click(screen.getByRole("button", { name: "Reservations" }));
  await screen.findByText("No active reservations.");
  await user.click(screen.getByRole("button", { name: "Find member by name" }));
  await user.type(screen.getByRole("combobox", { name: "Member name" }), "Ash");
  await user.click(await screen.findByRole("option", { name: "Asha Nair (MEM-004)" }));
  expect(screen.getByRole("textbox", { name: "Member number" })).toHaveValue("MEM-004");
  await user.type(screen.getByRole("textbox", { name: "Book barcode or accession number" }), "BOOK-004");
  await user.click(screen.getByRole("button", { name: "Place reservation" }));
  expect(invoke).toHaveBeenCalledWith("circulation_place_reservation", { memberNumber: "MEM-004", barcode: "BOOK-004" });
});

it("opens the admin loan preview and keeps it hidden for circulation staff", async () => {
  renderCirculation();
  expect(screen.queryByRole("button", { name: "Preview loan import" })).not.toBeInTheDocument();
});

it("opens the loan CSV preview for an administrator", async () => {
  renderCirculation(true);
  await userEvent.setup().click(screen.getByRole("button", { name: "Preview loan import" }));
  expect(await screen.findByRole("dialog", { name: "Preview loan import" })).toBeInTheDocument();
});

it("keeps renewal import hidden for circulation-only staff", () => {
  renderCirculation();
  expect(screen.queryByRole("button", { name: "Import renewal history" })).not.toBeInTheDocument();
});
it("opens the administrator renewal import", async () => {
  renderCirculation(true);
  await userEvent.setup().click(screen.getByRole("button", { name: "Import renewal history" }));
  expect(await screen.findByRole("dialog", { name: "Import renewal history" })).toBeInTheDocument();
});

it("hides financial-history import from circulation-only staff",()=>{
  renderCirculation();expect(screen.queryByRole("button",{name:"Import reservations and fines"})).not.toBeInTheDocument();
});
it("opens the five-part migration dialog for an administrator",async()=>{
  vi.mocked(invoke).mockResolvedValueOnce([]);
  renderCirculation(true);await userEvent.setup().click(screen.getByRole("button",{name:"Import reservations and fines"}));
  expect(await screen.findByRole("dialog",{name:"Import reservations and financial history"})).toBeInTheDocument();
});

it("fills a selected book barcode and requires the normal review step", async () => {
 vi.mocked(invoke).mockImplementation(async (command) => {
  if(command === "catalogue_search") return {rows:[{id:9,title:"Book",authors:"Author",availableCopies:1}],total:1,page:1,pageSize:25};
  if(command === "catalogue_get_record") return {archived:false,copies:[{id:10,accessionNo:"ACC-10",barcode:"BC-10",status:"available"}]};
  if(command === "circulation_prepare_issue") return {memberName:"Reader",title:"Book",dueOn:"2026-12-01",memberId:1,itemId:10};
 });
 renderCirculation(false,true); const user=userEvent.setup();
 await user.type(screen.getByRole("textbox",{name:"Member number"}),"M1");
 await user.click(screen.getByRole("button",{name:"Find book by name"}));
 await user.type(screen.getByRole("combobox",{name:"Book title"}),"Book");
 await user.click(await screen.findByRole("option"));
 await user.click(await screen.findByRole("button",{name:/ACC-10/}));
 expect(screen.getByRole("textbox",{name:"Book barcode or accession number"})).toHaveValue("BC-10");
 expect(invoke).not.toHaveBeenCalledWith("circulation_issue",expect.anything());
 await user.click(screen.getByRole("button",{name:"Review loan"}));
 await screen.findByRole("dialog",{name:"Confirm issue"});
 expect(invoke).toHaveBeenCalledWith("circulation_prepare_issue",{memberNumber:"M1",barcode:"BC-10"});
});

it.each(["Return", "Renew"] as const)("selects a current loan in %s and still requires confirmation", async (tab) => {
 const loan={loanId:9,title:"Current book",barcode:"COPY-9",accessionNo:"A9",memberName:"Reader",memberNumber:"M9",dueOn:"2026-12-10"};
 vi.mocked(circulationApi.listOpenLoans).mockResolvedValue({rows:[loan],page:1,pageSize:25,total:1});
 vi.mocked(invoke).mockResolvedValue({...loan, previousDueOn:"2026-12-01",estimatedFine:null});
 renderCirculation(); const user=userEvent.setup(); await user.click(screen.getByRole("button",{name:tab}));
 await user.click(await screen.findByRole("button",{name:`Review ${tab.toLowerCase()} for copy COPY-9`}));
 const dialog=within(await screen.findByRole("dialog",{name:`Confirm ${tab.toLowerCase() === "renew" ? "renewal" : "return"}`}));
 expect(dialog.getByText("Current book")).toBeInTheDocument();
 expect(invoke).toHaveBeenCalledExactlyOnceWith(tab === "Renew" ? "circulation_prepare_renewal" : "circulation_prepare_return",{barcode:"COPY-9"});
 await user.click(dialog.getByRole("button",{name:"Cancel"}));
 expect(invoke).toHaveBeenCalledTimes(1);
});

it("chooses an on-loan copy by title in Reservations without placing it automatically", async () => {
 vi.mocked(invoke).mockImplementation(async (command) => {
  if(command === "circulation_list_reservations") return {rows:[],page:1,pageSize:25,total:0};
  if(command === "catalogue_search") return {rows:[{id:1,title:"Reserved book",authors:"Writer",availableCopies:0}],page:1,pageSize:25,total:1};
  if(command === "catalogue_get_record") return {archived:false,copies:[{id:1,accessionNo:"A9",barcode:"B9",status:"on_loan"}]};
 });
 renderCirculation(false,true); const user=userEvent.setup(); await user.click(screen.getByRole("button",{name:"Reservations"}));
 await user.click(await screen.findByRole("button",{name:"Find book by name"}));
 await user.type(screen.getByRole("combobox",{name:"Book title"}),"Reserved"); await user.click(await screen.findByRole("option"));
 await user.click(await screen.findByRole("button",{name:/A9.*B9/}));
 expect(screen.getByRole("textbox",{name:"Book barcode or accession number"})).toHaveValue("B9");
 expect(vi.mocked(invoke).mock.calls.some(([command])=>command==="circulation_place_reservation")).toBe(false);
 await user.type(screen.getByRole("textbox",{name:"Member number"}),"M9"); await user.click(screen.getByRole("button",{name:"Place reservation"}));
 expect(invoke).toHaveBeenCalledWith("circulation_place_reservation",{memberNumber:"M9",barcode:"B9"});
});
