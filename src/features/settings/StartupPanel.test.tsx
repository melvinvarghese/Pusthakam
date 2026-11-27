import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import { afterEach, expect, it, vi } from "vitest";
import { StartupPanel } from "./StartupPanel";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
afterEach(() => { vi.unstubAllGlobals(); vi.mocked(invoke).mockReset(); });

function renderPanel(canEdit: boolean) {
  vi.stubGlobal("__TAURI_INTERNALS__", {});
  return render(<QueryClientProvider client={new QueryClient()}><StartupPanel canEdit={canEdit} /></QueryClientProvider>);
}

it("turns starting with Windows off and back on", async () => {
  let enabled = true;
  vi.mocked(invoke).mockImplementation(async (command, args) => {
    if (command === "launch_at_startup_get") return { supported: true, enabled };
    if (command === "launch_at_startup_set") { enabled = (args as { enabled: boolean }).enabled; return { supported: true, enabled }; }
    throw new Error(`Unexpected command: ${command}`);
  });
  renderPanel(true);
  const box = await screen.findByRole("checkbox", { name: /system tray when this computer starts/ });
  expect(box).toBeChecked();
  const user = userEvent.setup();
  await user.click(box);
  expect(invoke).toHaveBeenCalledWith("launch_at_startup_set", { enabled: false });
  expect(await screen.findByRole("checkbox", { name: /system tray/ })).not.toBeChecked();
});

it("is hidden where starting with the computer is not supported", async () => {
  vi.mocked(invoke).mockResolvedValue({ supported: false, enabled: false });
  renderPanel(true);
  await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("launch_at_startup_get", undefined));
  expect(screen.queryByRole("checkbox")).not.toBeInTheDocument();
});

it("is read-only without settings permission", async () => {
  vi.mocked(invoke).mockResolvedValue({ supported: true, enabled: true });
  renderPanel(false);
  expect(await screen.findByRole("checkbox", { name: /system tray/ })).toBeDisabled();
});
