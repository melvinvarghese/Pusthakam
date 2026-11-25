import { describe, expect, it, vi } from "vitest";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { LanConfig } from "@/lib/api/types";
import { BrowserAddresses } from "./BrowserAddresses";

const config: LanConfig = { enabled: true, listenerRunning: true, bindAddress: "0.0.0.0", port: 4318, serverName: "Library", serverFingerprint: "test", opacEnabled: true, browserStaffEnabled: true, activeInterfaces: ["192.0.2.10", "127.0.0.1"] };
const renderAddresses = (overrides: Partial<LanConfig> = {}) => {
  const onCopy = vi.fn();
  const onRefresh = vi.fn();
  render(<BrowserAddresses config={{ ...config, ...overrides }} onCopy={onCopy} onRefresh={onRefresh} />);
  return { onCopy, onRefresh };
};

describe("BrowserAddresses", () => {
  it("separates the shared host IP from loopback and copies the complete OPAC URL", async () => {
    const user = userEvent.setup();
    const { onCopy, onRefresh } = renderAddresses();
    const shared = screen.getByRole("region", { name: "Open on other devices" });
    expect(within(shared).getByText("192.0.2.10")).toBeInTheDocument();
    expect(within(shared).getByText("http://192.0.2.10:4318/opac")).toBeInTheDocument();
    expect(within(shared).queryByText(/127\.0\.0\.1/)).not.toBeInTheDocument();
    expect(within(screen.getByRole("region", { name: "On this computer" })).getByText("http://127.0.0.1:4318/opac")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Copy OPAC address 192.0.2.10" }));
    expect(onCopy).toHaveBeenCalledWith("http://192.0.2.10:4318/opac", "OPAC address");
    await user.click(screen.getByRole("button", { name: "Refresh addresses" }));
    expect(onRefresh).toHaveBeenCalled();
  });

  it("does not advertise network addresses when bound only to loopback", () => {
    renderAddresses({ bindAddress: "127.0.0.1" });
    expect(screen.getByRole("alert")).toHaveTextContent("Hosting is limited to this computer");
    expect(screen.queryByText("http://192.0.2.10:4318/opac")).not.toBeInTheDocument();
  });

  it("shows only the bound interface and enabled portals", () => {
    renderAddresses({ bindAddress: "192.0.2.10", activeInterfaces: ["192.0.2.10", "192.0.2.11"], browserStaffEnabled: false });
    expect(screen.queryByText(/192\.0\.2\.11/)).not.toBeInTheDocument();
    expect(screen.queryByText(/\/staff/)).not.toBeInTheDocument();
  });

  it("explains missing Wi-Fi address without substituting loopback", () => {
    renderAddresses({ activeInterfaces: ["127.0.0.1"] });
    const shared = screen.getByRole("region", { name: "Open on other devices" });
    expect(within(shared).getByText(/No shared network address detected/)).toBeInTheDocument();
    expect(within(shared).queryByText(/http:/)).not.toBeInTheDocument();
  });

  it("hides addresses while disabled even if runtime data is stale", () => {
    renderAddresses({ enabled: false });
    expect(screen.queryByText(/http:/)).not.toBeInTheDocument();
  });

  it("uses bracketed IPv6 URLs", () => {
    renderAddresses({ bindAddress: "::", activeInterfaces: ["2001:db8::10", "::1"] });
    expect(screen.getByText("http://[2001:db8::10]:4318/opac")).toBeInTheDocument();
    expect(screen.getByText("http://[::1]:4318/opac")).toBeInTheDocument();
  });
  it("clearly distinguishes local hosting from unconfigured internet access", () => {
    renderAddresses();
    expect(screen.getByRole("region", { name: "Internet access" })).toHaveTextContent("Access from other networks: not set up");
  });
  it("labels public addresses as configured without claiming connectivity", () => {
    renderAddresses({ publicBaseUrl: "https://catalogue.example.org" });
    const internet = screen.getByRole("region", { name: "Configured internet addresses" });
    expect(within(internet).getByText("https://catalogue.example.org/staff")).toBeInTheDocument();
    expect(internet).toHaveTextContent("does not confirm internet connectivity");
    expect(screen.queryByRole("region", { name: "Internet access" })).not.toBeInTheDocument();
  });
});
