import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { LanSettingsPanel } from "./LanSettingsPanel";
import { lanApi } from "@/lib/api";
import type { LanConfig, PairedClient, LanPairingCode } from "@/lib/api/types";

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
    bindAddress: "0.0.0.0",
    port: 4318,
    serverName: "Central Library Server",
    serverFingerprint: "A1B2C3D4E5F678901234567890ABCDEF",
    opacEnabled: true,
    browserStaffEnabled: true,
    activeInterfaces: ["192.168.1.100"],
  };

  const mockClients: PairedClient[] = [
    {
      clientId: "client-1",
      clientName: "Front Desk PC",
      deviceKind: "desktop",
      ipAddress: "192.168.1.55",
      pairedAt: "2026-11-20T08:00:00Z",
      lastSeenAt: "2026-11-20T09:30:00Z",
    },
    {
      clientId: "client-2",
      clientName: "Catalog Tablet",
      deviceKind: "tablet",
      ipAddress: "192.168.1.60",
      pairedAt: "2026-11-19T10:00:00Z",
      lastSeenAt: "2026-11-19T11:00:00Z",
      revokedAt: "2026-11-19T12:00:00Z",
    },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(lanApi.getConfig).mockResolvedValue(mockConfig);
    vi.mocked(lanApi.listPairedClients).mockResolvedValue(mockClients);
  });

  it("renders server status, fingerprint, OPAC addresses, and connected devices", async () => {
    render(<LanSettingsPanel />);

    await waitFor(() => {
      expect(screen.getByText("LAN Multi-Computer & Browser Access")).toBeInTheDocument();
    });

    expect(screen.getByText("Active on port 4318")).toBeInTheDocument();
    expect(screen.getByText(/Fingerprint: A1B2C3D4/)).toBeInTheDocument();
    expect(screen.getByText("http://192.168.1.100:4318/opac")).toBeInTheDocument();
    expect(screen.getByText("Front Desk PC")).toBeInTheDocument();
    expect(screen.getByText("Catalog Tablet")).toBeInTheDocument();
    expect(screen.getByText("Authorized")).toBeInTheDocument();
    expect(screen.getByText("Revoked")).toBeInTheDocument();
  });

  it("generates and displays a pairing code", async () => {
    const user = userEvent.setup();
    const mockCode: LanPairingCode = {
      code: "7K9M2P",
      expiresAt: "2026-11-20T10:30:00Z",
      createdAt: "2026-11-20T10:15:00Z",
      isValid: true,
    };
    vi.mocked(lanApi.generatePairingCode).mockResolvedValue(mockCode);

    render(<LanSettingsPanel />);

    await waitFor(() => {
      expect(screen.getByText("Generate Pairing Code")).toBeInTheDocument();
    });

    await user.click(screen.getByText("Generate Pairing Code"));

    await waitFor(() => {
      expect(screen.getByText("7K9M2P")).toBeInTheDocument();
    });
    expect(screen.getByText("Active Pairing Code")).toBeInTheDocument();
  });

  it("allows revoking an authorized device", async () => {
    const user = userEvent.setup();
    vi.spyOn(window, "confirm").mockReturnValue(true);
    vi.mocked(lanApi.revokeClient).mockResolvedValue(undefined);

    render(<LanSettingsPanel />);

    await waitFor(() => {
      expect(screen.getByText("Front Desk PC")).toBeInTheDocument();
    });

    const revokeButtons = screen.getAllByRole("button", { name: "Revoke" });
    expect(revokeButtons.length).toBe(1); // Only for the un-revoked device

    const targetBtn = revokeButtons[0];
    expect(targetBtn).toBeDefined();
    if (targetBtn) {
      await user.click(targetBtn);
    }

    expect(window.confirm).toHaveBeenCalled();
    expect(lanApi.revokeClient).toHaveBeenCalledWith("client-1");
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
