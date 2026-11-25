import { useState, useEffect } from "react";
import { lanApi, toApiError } from "@/lib/api";
import type { LanConfig } from "@/lib/api/types";
import { Button } from "@/components/ui/Button";
import { SelectField, TextField } from "@/components/ui/Field";
import { BrowserAddresses } from "./BrowserAddresses";
import {
  Wifi,
  RefreshCw,
  Power,
  Globe,
  AlertTriangle,
} from "lucide-react";

export function LanSettingsPanel() {
  const [config, setConfig] = useState<LanConfig | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Form edit state
  const [serverName, setServerName] = useState("");
  const [port, setPort] = useState(4318);
  const [bindAddress, setBindAddress] = useState("0.0.0.0");
  const [opacEnabled, setOpacEnabled] = useState(true);
  const [browserStaffEnabled, setBrowserStaffEnabled] = useState(true);
  const [publicBaseUrl, setPublicBaseUrl] = useState("");

  const loadData = async () => {
    setLoading(true);
    try {
      const cfg = await lanApi.getConfig();
      setConfig(cfg);
      setServerName(cfg.serverName);
      setPort(cfg.port);
      setBindAddress(cfg.bindAddress);
      setOpacEnabled(cfg.opacEnabled);
      setBrowserStaffEnabled(cfg.browserStaffEnabled);
      setPublicBaseUrl(cfg.publicBaseUrl ?? "");

      setError(null);
    } catch (e) {
      setError(toApiError(e).message);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void loadData();
  }, []);

  const handleToggleEnable = async (enabled: boolean) => {
    if (!config) return;
    setSaving(true);
    setMessage(null);
    setError(null);
    try {
      const updated = await lanApi.updateConfig({
        enabled,
        serverName,
        port,
        bindAddress,
        opacEnabled,
        browserStaffEnabled,
        publicBaseUrl,
      });
      setConfig(updated);
      setMessage(enabled ? "LAN hosting enabled." : "LAN hosting disabled.");
    } catch (e) {
      setError(toApiError(e).message);
    } finally {
      setSaving(false);
    }
  };

  const handleSaveConfig = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!config) return;
    setSaving(true);
    setMessage(null);
    setError(null);
    try {
      const updated = await lanApi.updateConfig({
        enabled: config.enabled,
        serverName,
        port,
        bindAddress,
        opacEnabled,
        browserStaffEnabled,
        publicBaseUrl,
      });
      setConfig(updated);
      setMessage("LAN configuration saved.");
    } catch (e) {
      setError(toApiError(e).message);
    } finally {
      setSaving(false);
    }
  };

  const copyToClipboard = async (text: string, label: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setMessage(`Copied ${label} to clipboard.`);
    } catch {
      setError("Could not copy the address. Select the displayed text and copy it manually.");
    }
  };

  if (loading && !config) {
    return (
      <div className="p-8 text-center text-slate-500">
        <RefreshCw className="h-6 w-6 animate-spin mx-auto mb-2" />
        Loading LAN server configuration...
      </div>
    );
  }

  return (
    <div className="space-y-8 max-w-4xl">
      {message && (
        <div className="p-3 bg-emerald-50 dark:bg-emerald-950/40 border border-emerald-200 dark:border-emerald-800 text-emerald-800 dark:text-emerald-200 rounded-md text-sm">
          {message}
        </div>
      )}
      {error && (
        <div className="p-3 bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-800 text-rose-800 dark:text-rose-200 rounded-md text-sm">
          {error}
        </div>
      )}

      {config?.listenerError && <p role="alert" className="text-danger">{config.listenerError} Check the bind address and whether another application uses this port.</p>}
      {/* 1. Header & Server Status Card */}
      <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg p-6 shadow-sm">
        <div className="staff-hosting-status flex items-start justify-between">
          <div className="flex items-center gap-3">
            <div className={`p-3 rounded-lg ${config?.enabled ? "bg-emerald-100 text-emerald-700 dark:bg-emerald-950 dark:text-emerald-400" : "bg-slate-100 text-slate-600 dark:bg-slate-800 dark:text-slate-400"}`}>
              <Wifi className="h-6 w-6" />
            </div>
            <div>
              <h2 className="text-lg font-semibold text-slate-900 dark:text-slate-100">
                Browser access
              </h2>
              <p className="text-sm text-slate-500 dark:text-slate-400">
                Share OPAC and staff access on the library network, with optional public web hosting.
              </p>
            </div>
          </div>
          <div>
            <Button
              variant={config?.enabled ? "secondary" : "primary"}
              onClick={() => handleToggleEnable(!config?.enabled)}
              disabled={saving}
              className={`gap-2 ${config?.enabled ? "text-rose-600 dark:text-rose-400 border-rose-300 dark:border-rose-800 hover:bg-rose-50 dark:hover:bg-rose-950" : ""}`}
            >
              <Power className="h-4 w-4" />
              {config?.enabled ? "Disable Wi-Fi / LAN Hosting" : "Enable Wi-Fi / LAN Hosting"}
            </Button>
          </div>
        </div>

        {/* Server Status Banner */}
        <div className="mt-6 pt-6 border-t border-slate-200 dark:border-slate-800 grid grid-cols-1 md:grid-cols-2 gap-4">
          <div className="p-4 bg-slate-50 dark:bg-slate-800/50 rounded-lg border border-slate-200 dark:border-slate-700/60">
            <div className="text-xs font-semibold text-slate-500 dark:text-slate-400 uppercase tracking-wider mb-1">
              Server Status
            </div>
            <div className="flex items-center gap-2">
              <span className={`h-2.5 w-2.5 rounded-full ${config?.listenerRunning ? "bg-emerald-500 animate-pulse" : "bg-slate-400"}`} />
              <span className="font-semibold text-slate-900 dark:text-slate-100">
                {config?.listenerRunning ? `Active on port ${config.port}` : config?.enabled ? "Listener unavailable" : "Stopped"}
              </span>
            </div>
          </div>

          <div className="p-4 bg-slate-50 dark:bg-slate-800/50 rounded-lg border border-slate-200 dark:border-slate-700/60">
            <div className="text-xs font-semibold text-slate-500 dark:text-slate-400 uppercase tracking-wider mb-1">
              Browser Addresses
            </div>
            <BrowserAddresses config={config} onCopy={copyToClipboard} onRefresh={() => void loadData()} />
          </div>
        </div>
      </div>

      {config?.listenerRunning && <p className="text-sm text-muted">Open /staff for staff sign-in or /opac for public catalogue search on the addresses above. Use the network address on other devices; 127.0.0.1 works only on this computer. Devices must share the same network. If this computer works but another device cannot connect, check the Windows firewall and network isolation.</p>}
      <p className="text-sm text-muted">Visitors open the OPAC address without signing in. Staff open the staff address and sign in with their account. Device pairing is not required.</p>

      {/* Server configuration */}
      <form onSubmit={handleSaveConfig} className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg p-6 shadow-sm space-y-4">
        <h3 className="text-base font-semibold text-slate-900 dark:text-slate-100 flex items-center gap-2">
          <Globe className="h-5 w-5 text-sky-600 dark:text-sky-400" />
          Server Settings
        </h3>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <TextField
            label="Library Server Name"
            value={serverName}
            onChange={(e) => setServerName(e.target.value)}
            required
          />
          <TextField
            label="Port"
            type="number"
            value={port.toString()}
            onChange={(e) => setPort(parseInt(e.target.value, 10) || 4318)}
            required
          />
        </div>

        <TextField label="Public HTTPS address (optional)" type="url" value={publicBaseUrl} onChange={e=>setPublicBaseUrl(e.target.value)} placeholder="https://library.example.org" hint="Each library can use its own website address. Leave this empty for local access. Set it after HTTPS hosting is configured; entering an address alone does not publish the library." />
        <p className="text-sm text-muted">The library computer is the default host. A library can also use an always-on server for its website. The chosen host must stay running for visitors to connect.</p>

        <SelectField label="Network access" value={bindAddress} onChange={e => setBindAddress(e.target.value)} hint="Choose the same Wi-Fi or wired network to let phones and other computers open OPAC.">
          <option value="0.0.0.0">Same Wi-Fi or wired network</option>
          <option value="127.0.0.1">This computer only</option>
          {!["0.0.0.0", "127.0.0.1"].includes(bindAddress) && <option value={bindAddress}>Current interface: {bindAddress}</option>}
        </SelectField>

        <div className="space-y-3 pt-2">
          <label className="flex items-center gap-3 cursor-pointer">
            <input
              type="checkbox"
              checked={opacEnabled}
              onChange={(e) => setOpacEnabled(e.target.checked)}
              className="h-4 w-4 rounded border-slate-300 text-sky-600 focus:ring-sky-500"
            />
            <span className="text-sm font-medium text-slate-800 dark:text-slate-200">
              Enable Public Anonymous OPAC (visitors can search catalogue without logging in)
            </span>
          </label>

          <label className="flex items-center gap-3 cursor-pointer">
            <input
              type="checkbox"
              checked={browserStaffEnabled}
              onChange={(e) => setBrowserStaffEnabled(e.target.checked)}
              className="h-4 w-4 rounded border-slate-300 text-sky-600 focus:ring-sky-500"
            />
            <span className="text-sm font-medium text-slate-800 dark:text-slate-200">
              Enable Browser Staff Access (staff can sign in from phone or tablet browsers)
            </span>
          </label>
        </div>

        <div className="pt-3 flex justify-end">
          <Button type="submit" variant="primary" disabled={saving}>
            Save Server Settings
          </Button>
        </div>
      </form>

      {/* 4. LAN Operating Guidance & Firewall Tips (LAN-OPS) */}
      <div className="bg-amber-50 dark:bg-amber-950/30 border border-amber-200 dark:border-amber-800 rounded-lg p-5">
        <div className="flex gap-3">
          <AlertTriangle className="h-5 w-5 text-amber-600 dark:text-amber-400 shrink-0 mt-0.5" />
          <div className="text-xs text-amber-900 dark:text-amber-200 space-y-2">
            <div className="font-semibold text-sm">Operating Guidance for Local Network Mode</div>
            <p>
              • <strong>Host Running Requirement:</strong> This computer hosts the SQLite library database. Pusthakam must remain running and the computer must not enter sleep mode while other computers or tablets are using the system.
            </p>
            <p>
              • <strong>Windows Defender Firewall:</strong> If connected clients cannot load the server page, ensure Windows Defender Firewall allows incoming connections on port <strong>{port}</strong> for Pusthakam.
            </p>
            <p>
              • <strong>Wi-Fi Client Isolation:</strong> Ensure all computers and phones are connected to the same Wi-Fi network or switch, and that the router's "AP Client Isolation" or "Guest Isolation" is turned off so devices can see each other.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
