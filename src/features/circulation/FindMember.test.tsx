import { beforeEach, expect, it, vi } from "vitest";
import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { MemberPage } from "@/lib/api/types";
const list = vi.hoisted(() => vi.fn());
vi.mock("@/lib/api", async (original) => ({ ...(await original<typeof import("@/lib/api")>()), membersApi: { list } }));
import { FindMember } from "./FindMember";
import { ApiError } from "@/lib/api";

const page = (name = "Asha Nair", number = "MEM-001") => ({ rows: [{ id: 1, name, memberNumber: number }], total: 1, page: 1, pageSize: 25 });
function show() {
  const selected = vi.fn();
  render(<QueryClientProvider client={new QueryClient()}><FindMember onClose={vi.fn()} onSelect={selected} /></QueryClientProvider>);
  return selected;
}
beforeEach(() => { list.mockReset(); });

it("searches automatically after typing, ignores empty text, and selects a duplicate name by number", async () => {
  list.mockResolvedValue({ ...page(), rows: [{ id: 1, name: "Asha Nair", memberNumber: "MEM-001" }, { id: 2, name: "Asha Nair", memberNumber: "MEM-002" }], total: 2 });
  const selected = show();
  const user = userEvent.setup();
  expect(list).not.toHaveBeenCalled();
  await user.type(screen.getByRole("combobox"), "  Asha");
  await screen.findByRole("option", { name: "Asha Nair (MEM-002)" });
  expect(list).toHaveBeenCalledExactlyOnceWith("Asha", 1, false);
  await user.keyboard("{ArrowDown}{ArrowDown}{Enter}");
  expect(selected).toHaveBeenCalledExactlyOnceWith("MEM-002");
  await user.clear(screen.getByRole("combobox"));
  expect(screen.queryByRole("listbox")).not.toBeInTheDocument();
  expect(screen.getByRole("combobox")).toHaveAttribute("aria-expanded", "false");
});

it("does not display a stale response after the typed name changes", async () => {
  let resolveOld!: (value: ReturnType<typeof page>) => void;
  let resolveNew!: (value: ReturnType<typeof page>) => void;
  list.mockImplementation((term: string) => new Promise((resolve) => { if (term === "Asha") resolveOld = resolve; else resolveNew = resolve; }));
  const selected = show(); const user = userEvent.setup();
  await user.type(screen.getByRole("combobox"), "Asha");
  await waitFor(() => expect(list).toHaveBeenCalledWith("Asha", 1, false));
  await user.clear(screen.getByRole("combobox")); await user.type(screen.getByRole("combobox"), "Binu");
  await act(async () => resolveOld(page()));
  expect(screen.queryByRole("option", { name: /Asha/ })).not.toBeInTheDocument();
  await waitFor(() => expect(list).toHaveBeenCalledWith("Binu", 1, false));
  await act(async () => resolveNew(page("Binu", "MEM-003")));
  await user.click(await screen.findByRole("option", { name: "Binu (MEM-003)" }));
  expect(selected).toHaveBeenCalledExactlyOnceWith("MEM-003");
});

it("shows no matches and allows retry after a search error", async () => {
  list.mockRejectedValueOnce(new ApiError({ code: "PERMISSION_DENIED", message: "Cannot search members." }))
    .mockResolvedValueOnce({ rows: [], total: 0, page: 1, pageSize: 25 });
  show(); const user = userEvent.setup();
  await user.type(screen.getByRole("combobox"), "Unknown");
  expect(await screen.findByRole("alert")).toHaveTextContent("Cannot search members.");
  await user.click(screen.getByRole("button", { name: "Retry search" }));
  await screen.findByText("No matching members.");
  expect(screen.queryByRole("option")).not.toBeInTheDocument();
});

it("loads another bounded page and resets to page one for a new term", async () => {
  list.mockImplementation(async (_term: string, current: number) => ({ ...page("Member " + current, "MEM-" + current), total: 26, page: current } as MemberPage));
  show(); const user = userEvent.setup();
  await user.type(screen.getByRole("combobox"), "Member");
  await user.click(await screen.findByRole("button", { name: "Next matches" }));
  await screen.findByRole("option", { name: "Member 2 (MEM-2)" });
  expect(list).toHaveBeenCalledWith("Member", 2, false);
  await user.clear(screen.getByRole("combobox")); await user.type(screen.getByRole("combobox"), "New");
  await waitFor(() => expect(list).toHaveBeenLastCalledWith("New", 1, false));
});
