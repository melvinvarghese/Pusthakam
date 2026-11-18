import { beforeEach, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
const api=vi.hoisted(()=>({previewRenewalImport:vi.fn(),commitRenewalImport:vi.fn()}));
vi.mock("@/lib/api",async(original)=>({...await original<typeof import("@/lib/api")>(),circulationApi:api}));
import RenewalImport from "./RenewalImport";
import { ApiError } from "@/lib/api";
function show(){render(<QueryClientProvider client={new QueryClient()}><RenewalImport onClose={vi.fn()}/></QueryClientProvider>);}
beforeEach(()=>{Object.values(api).forEach((mock)=>mock.mockReset());});

it("requires source and file, confirms the exact review, and shows success",async()=>{
  api.previewRenewalImport.mockResolvedValue({total:2,loans:1,invalid:0,problems:[],fingerprint:"review"});
  api.commitRenewalImport.mockResolvedValue(2);
  show();const user=userEvent.setup();
  expect(screen.getByRole("button",{name:"Preview renewals"})).toBeDisabled();
  await user.type(screen.getByLabelText("Source system or library"),"Windows test");
  await user.upload(screen.getByLabelText("Renewal history CSV"),new File(["renewals"],"renewals.csv",{type:"text/csv"}));
  await user.click(screen.getByRole("button",{name:"Preview renewals"}));
  expect(await screen.findByRole("status")).toHaveTextContent("2 renewal events for 1 loans; 0 invalid.");
  await user.click(screen.getByRole("button",{name:"Confirm import renewals"}));
  expect(await screen.findByRole("status")).toHaveTextContent("Imported 2 renewal events");
  expect(api.previewRenewalImport).toHaveBeenCalledWith(btoa("renewals"),"Windows test");
  expect(api.commitRenewalImport).toHaveBeenCalledWith(btoa("renewals"),"Windows test","review");
});

it("blocks invalid chains and clears the review when source changes",async()=>{
  api.previewRenewalImport.mockResolvedValue({total:1,loans:1,invalid:1,problems:["Record 1: Incomplete chain."],fingerprint:"review"});
  show();const user=userEvent.setup();
  await user.type(screen.getByLabelText("Source system or library"),"Windows test");
  await user.upload(screen.getByLabelText("Renewal history CSV"),new File(["bad"],"renewals.csv",{type:"text/csv"}));
  await user.click(screen.getByRole("button",{name:"Preview renewals"}));
  expect(await screen.findByRole("button",{name:"Confirm import renewals"})).toBeDisabled();
  expect(screen.getByText("Record 1: Incomplete chain.")).toBeInTheDocument();
  await user.type(screen.getByLabelText("Source system or library")," changed");
  expect(screen.queryByRole("status")).not.toBeInTheDocument();
  expect(screen.queryByRole("button",{name:"Confirm import renewals"})).not.toBeInTheDocument();
});

it("shows commit conflict and allows a fresh preview",async()=>{
  api.previewRenewalImport.mockResolvedValue({total:1,loans:1,invalid:0,problems:[],fingerprint:"review"});
  api.commitRenewalImport.mockRejectedValue(new ApiError({code:"VALIDATION",message:"Library data changed. Preview renewals again."}));
  show();const user=userEvent.setup();
  await user.type(screen.getByLabelText("Source system or library"),"Windows test");
  await user.upload(screen.getByLabelText("Renewal history CSV"),new File(["data"],"renewals.csv",{type:"text/csv"}));
  await user.click(screen.getByRole("button",{name:"Preview renewals"}));
  await user.click(await screen.findByRole("button",{name:"Confirm import renewals"}));
  expect(await screen.findByRole("alert")).toHaveTextContent("Library data changed.");
  await user.click(screen.getByRole("button",{name:"Preview renewals"}));
  await screen.findByRole("status");
  expect(screen.queryByRole("alert")).not.toBeInTheDocument();
});
