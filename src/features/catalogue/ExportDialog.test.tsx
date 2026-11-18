import { beforeEach, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

const mocks = vi.hoisted(() => ({ exportRecords: vi.fn(), save: vi.fn(), download: vi.fn() }));
vi.mock("@/lib/api", async (original) => ({ ...(await original<typeof import("@/lib/api")>()), isDesktop: () => true, catalogueApi: { exportRecords: mocks.exportRecords, saveExport: mocks.save } }));
vi.mock("@/lib/files", () => ({ downloadBase64File: mocks.download }));
import { ExportDialog } from "./ExportDialog";
import { ApiError } from "@/lib/api";

function show() {
  render(<QueryClientProvider client={new QueryClient()}><ExportDialog ids={[7, 12]} onClose={vi.fn()} /></QueryClientProvider>);
}
beforeEach(() => { mocks.exportRecords.mockReset(); mocks.download.mockReset(); mocks.save.mockReset(); mocks.save.mockResolvedValue({ native: true, path: "D:\\Exports\\catalogue.xlsx" }); });

it("prepares selected IDs, shows warnings, then saves the returned bytes only on request", async () => {
  mocks.exportRecords.mockResolvedValue({ fileName: "catalogue.xlsx", mimeType: "application/test", dataBase64: "AAEC", records: 2, copies: 3, warnings: ["Test exchange limitation."] });
  show();
  await userEvent.selectOptions(screen.getByLabelText("File format"), "xlsx");
  await userEvent.click(screen.getByRole("button", { name: "Prepare export" }));
  expect(mocks.exportRecords).toHaveBeenCalledExactlyOnceWith([7, 12], "xlsx");
  await screen.findByText("Ready: 2 titles and 3 copies.");
  expect(screen.getByText("Test exchange limitation.")).toBeInTheDocument();
  expect(mocks.download).not.toHaveBeenCalled();
  await userEvent.click(screen.getByRole("button", { name: "Save file" }));
  await waitFor(() => expect(mocks.save).toHaveBeenCalledExactlyOnceWith("catalogue.xlsx", "AAEC"));
  expect(await screen.findByText(/Saved to/)).toHaveTextContent("catalogue.xlsx");
  expect(mocks.download).not.toHaveBeenCalled();
  await userEvent.selectOptions(screen.getByLabelText("File format"), "json");
  expect(screen.queryByRole("button", { name: "Save file" })).not.toBeInTheDocument();
});

it("shows export errors and permits retry", async () => {
  mocks.exportRecords.mockRejectedValueOnce(new ApiError({ code: "VALIDATION", message: "Select fewer titles." })).mockResolvedValueOnce({ fileName: "catalogue.json", mimeType: "application/json", dataBase64: "W10=", records: 2, copies: 0, warnings: [] });
  show();
  await userEvent.click(screen.getByRole("button", { name: "Prepare export" }));
  expect(await screen.findByRole("alert")).toHaveTextContent("Select fewer titles.");
  await userEvent.click(screen.getByRole("button", { name: "Prepare export" }));
  await waitFor(() => expect(screen.getByRole("button", { name: "Save file" })).toBeEnabled());
});

it("keeps the prepared export after cancellation or a save error, and allows retry", async () => {
  mocks.exportRecords.mockResolvedValue({ fileName: "pusthakam-catalogue.json", mimeType: "application/json", dataBase64: "W10=", records: 2, copies: 0, warnings: [] });
  mocks.save.mockResolvedValueOnce({ native: true, path: null }).mockRejectedValueOnce(new ApiError({ code: "IO", message: "Choose another folder." })).mockResolvedValueOnce({ native: true, path: "D:/Exports/renamed.json" });
  show();
  await userEvent.click(screen.getByRole("button", { name: "Prepare export" }));
  await userEvent.click(await screen.findByRole("button", { name: "Save file" }));
  await screen.findByText(/Save cancelled/);
  expect(mocks.download).not.toHaveBeenCalled();
  await userEvent.click(screen.getByRole("button", { name: "Save file" }));
  expect(await screen.findByRole("alert")).toHaveTextContent("Choose another folder.");
  await userEvent.click(screen.getByRole("button", { name: "Save file" }));
  await screen.findByText("Saved to D:/Exports/renamed.json");
  expect(mocks.exportRecords).toHaveBeenCalledTimes(1);
});
