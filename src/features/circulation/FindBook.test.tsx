import { beforeEach, expect, it, vi } from "vitest";
import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
const api = vi.hoisted(() => ({ search: vi.fn(), getRecord: vi.fn() }));
vi.mock("@/lib/api", async (original) => ({ ...(await original<typeof import("@/lib/api")>()), catalogueApi: api }));
import { FindBook } from "./FindBook";
const title = (id = 1, name = "Book", authors = "Writer") => ({ id, title: name, authors, availableCopies: 1 });
const page = (rows = [title()], current = 1, total = rows.length) => ({ rows, page: current, pageSize: 25, total });
function show() { const selected = vi.fn(); render(<QueryClientProvider client={new QueryClient()}><FindBook onClose={vi.fn()} onSelect={selected} /></QueryClientProvider>); return selected; }
beforeEach(() => { api.search.mockReset(); api.getRecord.mockReset(); });
it("chooses a duplicate title and the exact eligible copy without issuing a loan", async () => {
 api.search.mockResolvedValue(page([title(), title(2, "Book", "Other writer")]));
 api.getRecord.mockResolvedValue({ archived: false, copies: [{ id: 1, accessionNo: "A1", barcode: "B1", status: "on_loan" }, { id: 2, accessionNo: "A2", barcode: "B2", status: "available" }, { id: 3, accessionNo: "A3", barcode: "B3", status: "reserved_hold" }] });
 const selected=show(); const user=userEvent.setup();
 expect(api.search).not.toHaveBeenCalled();
 await user.type(screen.getByRole("combobox"),"Book");
 await screen.findByRole("option",{name:/Other writer/});
 await user.keyboard("{ArrowDown}{ArrowDown}{Enter}");
 expect(api.getRecord).toHaveBeenCalledWith(2);
 expect(await screen.findByRole("button",{name:/A1 · B1 · On loan/})).toBeDisabled();
 await user.click(screen.getByRole("button",{name:/A2 · B2 · Available/}));
 expect(selected).toHaveBeenCalledExactlyOnceWith("B2");
 expect(screen.getByRole("button",{name:/A3 · B3 · Reserved/})).toBeEnabled();
});
it("hides late results after a new search term", async () => {
 let old!: (value: ReturnType<typeof page>)=>void;
 api.search.mockImplementation(({text}) => text === "Old" ? new Promise(resolve=>{old=resolve;}) : Promise.resolve(page([title(2,"New book")])));
 show(); const user=userEvent.setup(); await user.type(screen.getByRole("combobox"),"Old");
 await waitFor(()=>expect(api.search).toHaveBeenCalled());
 await user.clear(screen.getByRole("combobox")); await user.type(screen.getByRole("combobox"),"New");
 await act(async()=>old(page([title(1,"Old book")])));
 expect(screen.queryByRole("option",{name:/Old book/})).not.toBeInTheDocument();
 await screen.findByRole("option",{name:/New book/});
});
it("retries errors and resets pagination for a new title", async () => {
 api.search.mockRejectedValueOnce(new Error("Search failed")).mockImplementation(async ({page: current}) => page([title(current,`Book ${current}`)],current,26));
 show(); const user=userEvent.setup(); await user.type(screen.getByRole("combobox"),"Book");
 await screen.findByRole("alert"); await user.click(screen.getByRole("button",{name:"Retry search"}));
 await user.click(await screen.findByRole("button",{name:"Next matches"}));
 await screen.findByRole("option",{name:/Book 2/});
 await user.clear(screen.getByRole("combobox")); await user.type(screen.getByRole("combobox"),"Next");
 await waitFor(()=>expect(api.search).toHaveBeenLastCalledWith(expect.objectContaining({text:"Next",page:1,includeArchived:false})));
});
it("prevents selecting copies of a title archived after search", async () => {
 api.search.mockResolvedValue(page()); api.getRecord.mockResolvedValue({archived:true,copies:[{id:1,status:"available"}]});
 const selected=show(); const user=userEvent.setup(); await user.type(screen.getByRole("combobox"),"Book");
 await user.click(await screen.findByRole("option"));
 expect(await screen.findByRole("alert")).toHaveTextContent("archived"); expect(selected).not.toHaveBeenCalled();
});

it("allows an on-loan copy for reservation but keeps damaged copies disabled", async () => {
 api.search.mockResolvedValue(page()); api.getRecord.mockResolvedValue({archived:false,copies:[{id:1,accessionNo:"A1",barcode:"B1",status:"on_loan"},{id:2,accessionNo:"A2",barcode:"B2",status:"damaged"}]});
 const selected=vi.fn(); render(<QueryClientProvider client={new QueryClient()}><FindBook purpose="reservation" onClose={vi.fn()} onSelect={selected} /></QueryClientProvider>);
 const user=userEvent.setup(); await user.type(screen.getByRole("combobox"),"Book"); await user.click(await screen.findByRole("option"));
 await user.click(await screen.findByRole("button",{name:/A1.*B1/})); expect(selected).toHaveBeenCalledWith("B1");
 expect(screen.getByRole("button",{name:/A2.*B2/})).toBeDisabled();
});
