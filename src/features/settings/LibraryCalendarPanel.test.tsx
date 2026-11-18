import { beforeEach, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
const api = vi.hoisted(() => ({ calendar: vi.fn(), saveClosedWeekdays: vi.fn(), saveHoliday: vi.fn(), removeHoliday: vi.fn() }));
vi.mock("@/lib/api", async (original) => ({ ...(await original<typeof import("@/lib/api")>()), configApi: api }));
import { LibraryCalendarPanel } from "./LibraryCalendarPanel";
import { ApiError } from "@/lib/api";
function show(canEdit = true) { render(<QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}><LibraryCalendarPanel canEdit={canEdit} /></QueryClientProvider>); }
beforeEach(() => { Object.values(api).forEach((m) => m.mockReset()); api.calendar.mockResolvedValue({ closedWeekdays: [0], holidays: [{ id: 8, date: "2027-01-01", name: "New year" }] }); });

it("saves weekly closures and prevents closing all seven days", async () => {
  api.saveClosedWeekdays.mockResolvedValue(undefined);
  show();
  await screen.findByRole("checkbox", { name: "Saturday" });
  await userEvent.click(screen.getByRole("checkbox", { name: "Saturday" }));
  await userEvent.click(screen.getByRole("button", { name: "Save closed days" }));
  expect(api.saveClosedWeekdays).toHaveBeenCalledWith([0, 6], expect.anything());
  await waitFor(() => expect(screen.getByRole("button", { name: "Save closed days" })).toBeDisabled());
  for (const name of ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"]) await userEvent.click(screen.getByRole("checkbox", { name }));
  expect(screen.getByRole("button", { name: "Save closed days" })).toBeDisabled();
  expect(screen.getByRole("alert")).toHaveTextContent("Leave at least one weekday open.");
});

it("adds, edits and confirms removal of holiday dates", async () => {
  api.saveHoliday.mockResolvedValue(8); api.removeHoliday.mockResolvedValue(undefined);
  show();
  await userEvent.click(await screen.findByRole("button", { name: "Add holiday" }));
  fireEvent.change(screen.getByLabelText("Holiday date"), { target: { value: "2027-02-01" } });
  await userEvent.type(screen.getByLabelText("Holiday name"), "Library event");
  await userEvent.click(screen.getByRole("button", { name: "Save holiday" }));
  expect(api.saveHoliday).toHaveBeenCalledWith({ id: null, date: "2027-02-01", name: "Library event" }, expect.anything());
  await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
  await userEvent.click(screen.getByRole("button", { name: "Edit holiday New year" }));
  await userEvent.clear(screen.getByLabelText("Holiday name"));
  await userEvent.type(screen.getByLabelText("Holiday name"), "New year closure");
  await userEvent.click(screen.getByRole("button", { name: "Save holiday" }));
  expect(api.saveHoliday).toHaveBeenLastCalledWith({ id: 8, date: "2027-01-01", name: "New year closure" }, expect.anything());
  await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
  await userEvent.click(screen.getByRole("button", { name: "Remove holiday New year" }));
  expect(api.removeHoliday).not.toHaveBeenCalled();
  await userEvent.click(within(screen.getByRole("dialog")).getByRole("button", { name: "Remove holiday" }));
  expect(api.removeHoliday).toHaveBeenCalledWith(8, expect.anything());
});

it("keeps read-only calendar controls disabled and hides editing actions", async () => {
  show(false); await screen.findByText("New year");
  expect(screen.getByRole("checkbox", { name: "Sunday" })).toBeDisabled();
  expect(screen.queryByRole("button", { name: /Add holiday|Edit holiday|Remove holiday|Save closed days/ })).not.toBeInTheDocument();
});

it("shows a save failure without discarding the holiday draft", async () => {
  api.saveHoliday.mockRejectedValue(new ApiError({ code: "CONFLICT", message: "That date is already in the library calendar." }));
  show(); await userEvent.click(await screen.findByRole("button", { name: "Add holiday" }));
  fireEvent.change(screen.getByLabelText("Holiday date"), { target: { value: "2027-01-01" } });
  await userEvent.type(screen.getByLabelText("Holiday name"), "Duplicate");
  await userEvent.click(screen.getByRole("button", { name: "Save holiday" }));
  expect(await screen.findByRole("alert")).toHaveTextContent("already in the library calendar");
  expect(screen.getByLabelText("Holiday name")).toHaveValue("Duplicate");
});
