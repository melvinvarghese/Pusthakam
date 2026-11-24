import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it, vi } from "vitest";
import { circulationApi } from "@/lib/api";
import { OpenLoans } from "./OpenLoans";

it("searches and paginates open loans, selects the exact copy and resets search to page one", async () => {
 const list=vi.spyOn(circulationApi,"listOpenLoans").mockImplementation(async (search,page) => ({rows:[{loanId:page,title:search || "Book",barcode:`B${page}`,accessionNo:`A${page}`,memberName:"Reader",memberNumber:"M1",dueOn:"2026-01-01"}],page,pageSize:25,total:26}));
 const select=vi.fn(); render(<QueryClientProvider client={new QueryClient()}><OpenLoans action="return" busy={false} onSelect={select}/></QueryClientProvider>);
 const user=userEvent.setup(); await screen.findByText("Book"); await user.click(screen.getByRole("button",{name:"Next"}));
 await user.click(await screen.findByRole("button",{name:"Review return for copy B2"})); expect(select).toHaveBeenCalledExactlyOnceWith("B2");
 await user.type(screen.getByRole("textbox",{name:"Search current loans"}),"Reader");
 await waitFor(()=>expect(list).toHaveBeenLastCalledWith("Reader",1));
 list.mockRestore();
});
