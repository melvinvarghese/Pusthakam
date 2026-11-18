import { beforeEach, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
const api = vi.hoisted(() => ({ previewImport: vi.fn(), commitImport: vi.fn() }));
vi.mock("@/lib/api", async (original) => ({ ...(await original<typeof import("@/lib/api")>()), circulationApi: api }));
import LoanImport from "./LoanImport";
import { ApiError } from "@/lib/api";
function show() { render(<QueryClientProvider client={new QueryClient()}><LoanImport onClose={vi.fn()} /></QueryClientProvider>); }
beforeEach(() => { api.previewImport.mockReset(); api.commitImport.mockReset(); });

it("blocks invalid imports and clears stale results on file change", async () => {
  api.previewImport.mockResolvedValue({ total: 3, open: 1, closed: 1, invalid: 1, problems: ["Record 3: Member number is not imported."] });
  show(); const user = userEvent.setup();
  expect(screen.getByRole("button", { name: "Preview CSV" })).toBeDisabled();
  await user.upload(screen.getByLabelText("Loan history CSV"), new File(["example"], "loans.csv", { type: "text/csv" }));
  await user.click(screen.getByRole("button", { name: "Preview CSV" }));
  expect(await screen.findByRole("status")).toHaveTextContent("3 records: 1 valid open loans, 1 valid closed loans, 1 invalid.");
  expect(api.previewImport).toHaveBeenCalledWith(btoa("example"));
  expect(screen.getByText(/Record 3:/)).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Confirm import loans" })).toBeDisabled();
  await user.upload(screen.getByLabelText("Loan history CSV"), new File(["other"], "other.csv", { type: "text/csv" }));
  expect(screen.queryByRole("status")).not.toBeInTheDocument();
});

it("shows backend errors and permits retry with empty-file feedback", async () => {
  api.previewImport.mockRejectedValueOnce(new ApiError({ code: "VALIDATION", message: "Invalid CSV header." }))
    .mockResolvedValueOnce({ total: 0, open: 0, closed: 0, invalid: 0, problems: [] });
  show(); const user=userEvent.setup();
  await user.upload(screen.getByLabelText("Loan history CSV"), new File(["header"], "loans.csv", { type: "text/csv" }));
  await user.click(screen.getByRole("button", { name: "Preview CSV" }));
  expect(await screen.findByRole("alert")).toHaveTextContent("Invalid CSV header.");
  await user.click(screen.getByRole("button", { name: "Preview CSV" }));
  expect(await screen.findByText("The file contains no loan records.")).toBeInTheDocument();
  expect(screen.queryByRole("alert")).not.toBeInTheDocument();
});

it("requires source and fine-policy confirmation, then submits the exact reviewed file and fingerprint", async () => {
  api.previewImport.mockResolvedValue({ fingerprint: "review-token", total: 1, open: 1, closed: 0, invalid: 0, problems: [] });
  api.commitImport.mockResolvedValue(1);
  show(); const user=userEvent.setup();
  await user.upload(screen.getByLabelText("Loan history CSV"), new File(["reviewed"], "loans.csv", { type: "text/csv" }));
  await user.click(screen.getByRole("button", { name: "Preview CSV" }));
  const confirm=await screen.findByRole("button", { name: "Confirm import loans" });
  expect(confirm).toBeDisabled();
  await user.type(screen.getByLabelText("Source system or library"), "Old library");
  expect(confirm).toBeDisabled();
  await user.click(screen.getByRole("checkbox"));
  await user.click(confirm);
  expect(await screen.findByRole("status")).toHaveTextContent("Imported 1 loans");
  expect(api.commitImport).toHaveBeenCalledWith(btoa("reviewed"), "Old library", "review-token", true);
  expect(screen.queryByRole("button", { name: "Confirm import loans" })).not.toBeInTheDocument();
});

it("shows a commit conflict and requires a new preview when data changed", async () => {
  api.previewImport.mockResolvedValue({ fingerprint: "old", total: 1, open: 0, closed: 1, invalid: 0, problems: [] });
  api.commitImport.mockRejectedValue(new ApiError({ code: "VALIDATION", message: "Library data changed." }));
  show(); const user=userEvent.setup();
  await user.upload(screen.getByLabelText("Loan history CSV"), new File(["history"], "loans.csv", { type: "text/csv" }));
  await user.click(screen.getByRole("button", { name: "Preview CSV" }));
  await user.type(await screen.findByLabelText("Source system or library"), "Old library");
  expect(screen.queryByRole("checkbox")).not.toBeInTheDocument();
  await user.click(screen.getByRole("button", { name: "Confirm import loans" }));
  expect(await screen.findByRole("alert")).toHaveTextContent("Library data changed.");
  await user.click(screen.getByRole("button", { name: "Preview CSV" }));
  await screen.findByLabelText("Source system or library");
  expect(screen.queryByRole("alert")).not.toBeInTheDocument();
});
