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
      folder: "test backups", files: [{ name, sizeBytes: 100 }], dailyEnabled: false, pendingRestore, lastRestore: null,
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
