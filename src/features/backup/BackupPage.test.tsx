import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import { afterEach, expect, it, vi } from "vitest";
import { SessionContext } from "@/lib/session";
import { BackupPage } from "./BackupPage";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
afterEach(() => { vi.unstubAllGlobals(); vi.mocked(invoke).mockReset(); });

it("queues a saved backup only after explicit confirmation", async () => {
  vi.stubGlobal("__TAURI_INTERNALS__", {});
  const name = "pusthakam-test.sqlite3";
  let pendingRestore: string | null = null;
  vi.mocked(invoke).mockImplementation(async (command) => {
    if (command === "backup_list") return {
      folder: "test backups", files: [{ name, sizeBytes: 100, folder: "test backups" }], dailyEnabled: false, pendingRestore, lastRestore: null,
      dailyFolder: null, dailyFolderAvailable: true, dailyKeep: 7, lastDaily: null,
    };
    if (command === "backup_request_restore") { pendingRestore = name; return undefined; }
    throw new Error(`Unexpected command: ${command}`);
  });
  render(<QueryClientProvider client={new QueryClient()}>
    <SessionContext.Provider value={{
      session: { userId: 1, username: "admin", displayName: "Administrator", role: "administrator", permissions: ["admin.backup"] },
      signOut: async () => {},
    }}><BackupPage /></SessionContext.Provider>
  </QueryClientProvider>);
  const user = userEvent.setup();
  await user.click(await screen.findByRole("button", { name: `Restore ${name}` }));
  expect(invoke).not.toHaveBeenCalledWith("backup_request_restore", { name });
  const dialog = within(screen.getByRole("dialog", { name: "Restore database on next launch" }));
  await user.click(dialog.getByRole("button", { name: "Queue restore" }));
  expect(await screen.findByText(/Restore queued for/)).toBeInTheDocument();
  expect(invoke).toHaveBeenCalledWith("backup_request_restore", { name });
});

function renderBackups() {
  render(<QueryClientProvider client={new QueryClient()}>
    <SessionContext.Provider value={{
      session: { userId: 1, username: "admin", displayName: "Administrator", role: "administrator", permissions: ["admin.backup"] },
      signOut: async () => {},
    }}><BackupPage /></SessionContext.Provider>
  </QueryClientProvider>);
}

it("chooses a daily backup folder and warns when it cannot be reached", async () => {
  vi.stubGlobal("__TAURI_INTERNALS__", {});
  let dailyFolder: string | null = null;
  vi.mocked(invoke).mockImplementation(async (command, args) => {
    if (command === "backup_list") return {
      folder: "C:\\data\\backups", files: [{ name: "pusthakam-20261127-daily.sqlite3", sizeBytes: 100, folder: "E:\\Backups" }],
      dailyEnabled: true, pendingRestore: null, lastRestore: null,
      dailyFolder, dailyFolderAvailable: dailyFolder === null, dailyKeep: 7,
      lastDaily: dailyFolder ? "E:\\Backups could not be reached, so the daily backup was saved in the Pusthakam data folder." : null,
    };
    if (command === "backup_set_daily_folder") { dailyFolder = (args as { folder: string }).folder; return dailyFolder; }
    throw new Error(`Unexpected command: ${command}`);
  });
  renderBackups();
  expect(await screen.findByText(/The last 7 are kept/)).toBeInTheDocument();
  expect(screen.getByText(/In the daily backup folder/)).toBeInTheDocument();
  const user = userEvent.setup();
  await user.click(screen.getByRole("button", { name: "Change folder" }));
  await user.type(screen.getByLabelText("Folder for daily backups"), "E:\\Backups");
  await user.click(screen.getByRole("button", { name: "Save folder" }));
  expect(invoke).toHaveBeenCalledWith("backup_set_daily_folder", { folder: "E:\\Backups" });
  expect(await screen.findByText(/cannot be reached right now/)).toBeInTheDocument();
  expect(screen.getByText(/could not be reached, so the daily backup/)).toBeInTheDocument();
});
