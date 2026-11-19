import { beforeEach, expect, it, vi } from "vitest";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
const api=vi.hoisted(()=>({previewHistory:vi.fn(),commitHistory:vi.fn(),historyImports:vi.fn()}));
vi.mock("@/lib/api",async(original)=>({...await original<typeof import("@/lib/api")>(),circulationApi:api}));
import HistoryImport from "./HistoryImport";
import { ApiError } from "@/lib/api";
const summary={counts:{reservations:3,fines:2,payments:2,waivers:1},balances:[
  {currency:"INR",assessedMinor:1000,paidMinor:300,waivedMinor:200,outstandingMinor:500},
  {currency:"USD",assessedMinor:500,paidMinor:100,waivedMinor:0,outstandingMinor:400},
]};
const preview={sourceSystem:"Legacy",summary,expected:summary,problems:[],canCommit:true,fingerprint:"exact-review"};
function show(){render(<QueryClientProvider client={new QueryClient()}><HistoryImport onClose={vi.fn()}/></QueryClientProvider>);}
async function load(){const user=userEvent.setup();await user.upload(screen.getByLabelText("History bundle JSON"),new File(["bundle"],"history.json",{type:"application/json"}));await user.click(screen.getByRole("button",{name:"Preview history"}));return user;}
beforeEach(()=>{Object.values(api).forEach((m)=>m.mockReset());api.previewHistory.mockResolvedValue(preview);api.historyImports.mockResolvedValue([]);});

it("compares source/destination counts and currencies, confirms the exact review and displays saved reconciliation",async()=>{
  api.commitHistory.mockResolvedValue(summary);show();const user=await load();
  const table=await screen.findByRole("table",{name:"Calculated destination balances"});
  expect(within(table).getByRole("row",{name:/INR/})).toHaveTextContent("5.00");
  expect(within(table).getByRole("row",{name:/USD/})).toHaveTextContent("4.00");
  expect(screen.getByRole("table",{name:"Expected source balances"})).toBeInTheDocument();
  await user.click(screen.getByRole("button",{name:"Confirm import history"}));
  expect(await screen.findByRole("status")).toHaveTextContent("History imported and reconciled");
  expect(api.commitHistory).toHaveBeenCalledWith(btoa("bundle"),"exact-review");
  expect(screen.getByRole("table",{name:"Saved reconciliation balances"})).toBeInTheDocument();
});
it("blocks mismatches and clears the old review after file changes",async()=>{
  api.previewHistory.mockResolvedValue({...preview,canCommit:false,problems:["Source currency totals do not match."]});
  show();const user=await load();
  expect(await screen.findByRole("alert")).toHaveTextContent("Source currency totals do not match.");
  expect(screen.getByRole("button",{name:"Confirm import history"})).toBeDisabled();
  await user.upload(screen.getByLabelText("History bundle JSON"),new File(["new"],"other.json",{type:"application/json"}));
  expect(screen.queryByRole("button",{name:"Confirm import history"})).not.toBeInTheDocument();expect(api.commitHistory).not.toHaveBeenCalled();
});
it("retains an atomic-commit error and permits a fresh preview",async()=>{
  api.commitHistory.mockRejectedValue(new ApiError({code:"VALIDATION",message:"Library data changed. Preview again."}));
  show();const user=await load();await user.click(await screen.findByRole("button",{name:"Confirm import history"}));
  expect(await screen.findByRole("alert")).toHaveTextContent("Nothing was imported.");
  await user.click(screen.getByRole("button",{name:"Preview history"}));await screen.findByRole("status");expect(screen.queryByRole("alert")).not.toBeInTheDocument();
});
it("shows malformed-file errors without exposing confirmation",async()=>{
  api.previewHistory.mockRejectedValue(new ApiError({code:"VALIDATION",message:"Invalid history JSON."}));
  show();await load();expect(await screen.findByRole("alert")).toHaveTextContent("Invalid history JSON.");expect(screen.queryByRole("button",{name:"Confirm import history"})).not.toBeInTheDocument();
});

it("reopens previously saved reconciliation summaries",async()=>{
  api.historyImports.mockResolvedValue([{id:7,sourceSystem:"Legacy",importedAt:"2026-11-19T00:00:00Z",summary}]);
  show();await screen.findByText("Recent saved reconciliations (latest 20)");
  await userEvent.setup().click(screen.getByText("Recent saved reconciliations (latest 20)"));
  expect(screen.getByRole("table",{name:"Saved batch #7 balances"})).toBeInTheDocument();
});
