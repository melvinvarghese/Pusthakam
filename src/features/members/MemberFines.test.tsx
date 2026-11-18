import { beforeEach, expect, it, vi } from "vitest";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { Member } from "@/lib/api/types";
const api = vi.hoisted(() => ({ fineAccount: vi.fn(), fineHistory: vi.fn() }));
vi.mock("@/lib/api", async (original) => ({ ...(await original<typeof import("@/lib/api")>()), membersApi: api }));
import { MemberFines } from "./MemberFines";
import { ApiError } from "@/lib/api";
const member: Member = { id: 4, name: "Asha Nair", memberNumber: "MEM-101", categoryId: 1, categoryName: "Student", email: null, phone: null, expiresOn: "2026-01-01" };
const account = {
  rows: [{ id: 9, title: "Book One", memberName: member.name, memberNumber: member.memberNumber, barcode: "000123", assessedAt: "2026-11-18T10:00:00Z", currency: "INR", amountMinor: 250, paidMinor: 100, waivedMinor: 25, outstandingMinor: 125 }],
  balances: [{ currency: "INR", assessedMinor: 250, paidMinor: 100, waivedMinor: 25, outstandingMinor: 125 }, { currency: "USD", assessedMinor: 200, paidMinor: 0, waivedMinor: 0, outstandingMinor: 200 }],
  page: 1, pageSize: 25, total: 26,
};
function show() { render(<QueryClientProvider client={new QueryClient()}><MemberFines member={member} onClose={vi.fn()} /></QueryClientProvider>); }
beforeEach(() => { Object.values(api).forEach((m) => m.mockReset()); api.fineAccount.mockResolvedValue(account); });

it("shows server-provided all-page balances by currency and paginates member fines", async () => {
  show(); const user = userEvent.setup();
  const summary = await screen.findByRole("table", { name: "All assessed fines by currency" });
  expect(within(summary).getByRole("row", { name: /INR/ })).toHaveTextContent("1.25");
  expect(within(summary).getByRole("row", { name: /USD/ })).toHaveTextContent("2.00");
  expect(screen.getByText("000123 · #9")).toBeInTheDocument();
  expect(screen.queryByRole("button", { name: /Record payment|Waive/ })).not.toBeInTheDocument();
  api.fineAccount.mockResolvedValueOnce({ ...account, page: 2 });
  await user.click(screen.getByRole("button", { name: "Next" }));
  expect(api.fineAccount).toHaveBeenLastCalledWith(4, 2);
});

it("opens member-scoped payment/waiver history and returns to the account", async () => {
  api.fineHistory.mockResolvedValue({ page: 1, pageSize: 25, total: 2, rows: [
    { id: 10, kind: "payment", amountMinor: 100, method: "cash", reason: null, recordedAt: "2026-11-18T10:00:00Z", staffName: "Clerk" },
    { id: 11, kind: "waiver", amountMinor: 25, method: null, reason: null, recordedAt: "2026-11-18T10:01:00Z", staffName: "Librarian" },
  ] });
  show(); const user = userEvent.setup();
  await user.click(await screen.findByRole("button", { name: "Settlement history for fine 9" }));
  expect(await screen.findByText(/Payment:/)).toHaveTextContent("1.00");
  expect(screen.getByText(/Waiver:/)).toHaveTextContent("0.25");
  expect(api.fineHistory).toHaveBeenCalledExactlyOnceWith(4, 9, 1);
  expect(screen.getAllByRole("dialog")).toHaveLength(1);
  await user.click(screen.getByRole("button", { name: "Back to fines" }));
  expect(screen.getByRole("table", { name: "All assessed fines by currency" })).toBeInTheDocument();
});

it("shows an empty account and retries a failed account request", async () => {
  api.fineAccount.mockRejectedValueOnce(new ApiError({ code: "PERMISSION_DENIED", message: "Cannot read this account." })).mockResolvedValueOnce({ rows: [], balances: [], page: 1, pageSize: 25, total: 0 });
  show(); const user = userEvent.setup();
  expect(await screen.findByRole("alert")).toHaveTextContent("Cannot read this account.");
  expect(screen.queryByRole("table")).not.toBeInTheDocument();
  await user.click(screen.getByRole("button", { name: "Retry fines" }));
  await screen.findByText("This member has no assessed fines.");
});
