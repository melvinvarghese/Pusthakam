import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import { afterEach, expect, it, vi } from "vitest";
import { SessionContext } from "@/lib/session";
import PrintingPage from "./PrintingPage";
vi.mock("@tauri-apps/api/core",()=>({invoke:vi.fn()}));
vi.mock("@/features/catalogue/AccessionPage",()=>({AccessionPage:()=> <p>Copy selection tools</p>}));
afterEach(()=>{vi.unstubAllGlobals();vi.restoreAllMocks();vi.mocked(invoke).mockReset();});
function show(permissions: string[]){
 vi.stubGlobal("__TAURI_INTERNALS__",{});
 render(<QueryClientProvider client={new QueryClient()}><SessionContext.Provider value={{session:{userId:1,username:"clerk",displayName:"Clerk",role:"circulation",permissions},signOut:async()=>{}}}><PrintingPage /></SessionContext.Provider></QueryClientProvider>);
}
it("guards print tools individually without reading unauthorized member data",async()=>{
 show(["catalogue.read"]); const user=userEvent.setup(); expect(screen.getByText("Copy selection tools")).toBeInTheDocument();
 await user.click(screen.getByRole("button",{name:"Member cards"})); expect(screen.getByRole("alert")).toHaveTextContent("permission");expect(invoke).not.toHaveBeenCalled();
});
it("previews member cards and reprints open-loan slips without modifying circulation",async()=>{
 vi.mocked(invoke).mockImplementation(async(command)=>{
  if(command==="config_overview") return {library:{name:"Library"}};
  if(command==="members_list") return {rows:[{id:1,name:"Reader",memberNumber:"M-1"}],page:1,pageSize:25,total:1};
  if(command==="members_get_photo") return null;
  if(command==="circulation_prepare_return") return {loanId:1,memberName:"Reader",title:"Book",dueOn:"2026-12-01"};
 });
 show(["members.read","circulation.issue"]);const user=userEvent.setup();
 await user.click(screen.getByRole("button",{name:"Member cards"}));
 await user.click(await screen.findByRole("button",{name:"Preview card for M-1"}));
 await screen.findByRole("button",{name:"Print card"});
 await user.keyboard("{Escape}");
 await user.click(screen.getByRole("button",{name:"Loan due slips"}));
 await user.type(screen.getByRole("textbox",{name:"Book barcode or accession number"}),"BC1");
 await user.click(screen.getByRole("button",{name:"Preview due slip"}));
 await screen.findByRole("dialog",{name:"Loan due slip"});
 expect(screen.getByText("Current loan")).toBeInTheDocument();
 expect(invoke).toHaveBeenCalledWith("circulation_prepare_return",{barcode:"BC1"});
 expect(vi.mocked(invoke).mock.calls.some(([cmd])=>["circulation_return","circulation_issue","circulation_renew"].includes(cmd))).toBe(false);
});
