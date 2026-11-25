import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { LanSettingsPanel } from "./LanSettingsPanel";
import { lanApi } from "@/lib/api";
import type { LanConfig } from "@/lib/api/types";

vi.mock("@/lib/api", () => ({
  lanApi: {
    getConfig: vi.fn(),
    updateConfig: vi.fn(),
    generatePairingCode: vi.fn(),
    listPairingCodes: vi.fn(),
    listPairedClients: vi.fn(),
    revokeClient: vi.fn(),
  },
  toApiError: (e: unknown) => ({ code: "ERROR", message: String(e) }),
}));

describe("LanSettingsPanel", () => {
  const mockConfig: LanConfig = {
    enabled: true,
    listenerRunning: true,
    listenerError: null,
    bindAddress: "0.0.0.0",
    port: 4318,
    serverName: "Central Library Server",
    serverFingerprint: "A1B2C3D4E5F678901234567890ABCDEF",
    opacEnabled: true,
    browserStaffEnabled: true,
    activeInterfaces: ["192.168.1.100"],
  };

  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(lanApi.getConfig).mockResolvedValue(mockConfig);
  });

  it("renders browser addresses without device pairing or client-list requests", async () => {
    render(<LanSettingsPanel />);

    await waitFor(() => {
      expect(screen.getByText("Browser access")).toBeInTheDocument();
    });

    expect(screen.getByText("Active on port 4318")).toBeInTheDocument();
    expect(screen.queryByText(/Fingerprint:/)).not.toBeInTheDocument();
    expect(screen.getByText("http://192.168.1.100:4318/opac")).toBeInTheDocument();
    expect(screen.queryByText("Device Pairing")).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Generate Pairing Code" })).not.toBeInTheDocument();
    expect(lanApi.listPairedClients).not.toHaveBeenCalled();
  });

  it("shows the host OPAC address immediately after enabling Wi-Fi hosting", async () => {
    const user = userEvent.setup();
    vi.mocked(lanApi.getConfig).mockResolvedValue({ ...mockConfig, enabled: false, listenerRunning: false });
    vi.mocked(lanApi.updateConfig).mockResolvedValue(mockConfig);
    render(<LanSettingsPanel />);
    await user.click(await screen.findByRole("button", { name: "Enable Wi-Fi / LAN Hosting" }));
    expect(await screen.findByText("http://192.168.1.100:4318/opac")).toBeInTheDocument();
    expect(screen.getByText("Library computer IP:")).toBeInTheDocument();
  });

  it("can change a local-only listener to the shared Wi-Fi network", async () => {
    const user = userEvent.setup();
    vi.mocked(lanApi.getConfig).mockResolvedValue({ ...mockConfig, bindAddress: "127.0.0.1" });
    vi.mocked(lanApi.updateConfig).mockResolvedValue(mockConfig);
    render(<LanSettingsPanel />);
    await user.selectOptions(await screen.findByLabelText("Network access"), "0.0.0.0");
    await user.click(screen.getByRole("button", { name: "Save Server Settings" }));
    expect(lanApi.updateConfig).toHaveBeenCalledWith(expect.objectContaining({ bindAddress: "0.0.0.0", enabled: true }));
    expect(await screen.findByText("http://192.168.1.100:4318/opac")).toBeInTheDocument();
  });

  it("saves updated server configuration", async () => {
    const user = userEvent.setup();
    vi.mocked(lanApi.updateConfig).mockResolvedValue({
      ...mockConfig,
      serverName: "New Server Name",
      port: 4320,
    });

    render(<LanSettingsPanel />);

    await waitFor(() => {
      expect(screen.getByDisplayValue("Central Library Server")).toBeInTheDocument();
    });

    const nameInput = screen.getByDisplayValue("Central Library Server");
    await user.clear(nameInput);
    await user.type(nameInput, "New Server Name");

    const saveButton = screen.getByRole("button", { name: "Save Server Settings" });
    await user.click(saveButton);

    expect(lanApi.updateConfig).toHaveBeenCalledWith(
      expect.objectContaining({
        serverName: "New Server Name",
      })
    );
  });
});

it("does not claim active hosting when the saved setting has no running listener", async () => {
 vi.mocked(lanApi.getConfig).mockResolvedValue({enabled:true,listenerRunning:false,listenerError:"Port already in use",bindAddress:"0.0.0.0",port:4318,serverName:"Library",serverFingerprint:"test",opacEnabled:true,browserStaffEnabled:true,activeInterfaces:["192.0.2.1"]});
 vi.mocked(lanApi.listPairedClients).mockResolvedValue([]);
 render(<LanSettingsPanel />);
 expect(await screen.findByText("Listener unavailable")).toBeInTheDocument();
 expect(screen.getByRole("alert")).toHaveTextContent("Port already in use");
 expect(screen.queryByText(/Active on port/)).not.toBeInTheDocument();
 expect(screen.queryByText("http://192.0.2.1:4318/opac")).not.toBeInTheDocument();
});
