import { useState, useEffect } from "react";
import { lanApi, toApiError } from "@/lib/api";
import type { LanConfig, PairedClient, LanPairingCode } from "@/lib/api/types";
import { Button } from "@/components/ui/Button";
import { TextField } from "@/components/ui/Field";
import {
  Wifi,
  ShieldCheck,
  Smartphone,
  Laptop,
  Copy,
  RefreshCw,
  Power,
  Globe,
  AlertTriangle,
  Users,
} from "lucide-react";

export function LanSettingsPanel() {
  const [config, setConfig] = useState<LanConfig | null>(null);
  const [clients, setClients] = useState<PairedClient[]>([]);
  const [pairingCode, setPairingCode] = useState<LanPairingCode | null>(null);
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

      const clientList = await lanApi.listPairedClients();
      setClients(clientList);
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
      });
      setConfig(updated);
      setMessage("LAN configuration saved.");
    } catch (e) {
      setError(toApiError(e).message);
    } finally {
      setSaving(false);
    }
  };

  const handleGenerateCode = async () => {
    setMessage(null);
    setError(null);
    try {
      const code = await lanApi.generatePairingCode();
      setPairingCode(code);
      setMessage("New pairing code generated (valid for 15 minutes).");
    } catch (e) {
      setError(toApiError(e).message);
    }
  };

  const handleRevokeClient = async (clientId: string, clientName: string) => {
    if (!confirm(`Are you sure you want to revoke access for "${clientName}"? Any active sessions will be terminated immediately.`)) {
      return;
    }
    setMessage(null);
    setError(null);
    try {
      await lanApi.revokeClient(clientId);
      setMessage(`Revoked access for ${clientName}.`);
      void loadData();
    } catch (e) {
      setError(toApiError(e).message);
    }
  };

  const copyToClipboard = (text: string, label: string) => {
    void navigator.clipboard.writeText(text);
    setMessage(`Copied ${label} to clipboard.`);
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

      {/* 1. Header & Server Status Card */}
      <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg p-6 shadow-sm">
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-3">
            <div className={`p-3 rounded-lg ${config?.enabled ? "bg-emerald-100 text-emerald-700 dark:bg-emerald-950 dark:text-emerald-400" : "bg-slate-100 text-slate-600 dark:bg-slate-800 dark:text-slate-400"}`}>
              <Wifi className="h-6 w-6" />
            </div>
            <div>
              <h2 className="text-lg font-semibold text-slate-900 dark:text-slate-100">
                LAN Multi-Computer & Browser Access
              </h2>
              <p className="text-sm text-slate-500 dark:text-slate-400">
                Allow library staff and patrons on the local network (Wi-Fi or Ethernet) to connect without internet.
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
              {config?.enabled ? "Disable LAN Mode" : "Enable LAN Mode"}
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
              <span className={`h-2.5 w-2.5 rounded-full ${config?.enabled ? "bg-emerald-500 animate-pulse" : "bg-slate-400"}`} />
              <span className="font-semibold text-slate-900 dark:text-slate-100">
                {config?.enabled ? `Active on port ${config.port}` : "Stopped (Offline)"}
              </span>
            </div>
            <div className="mt-2 text-xs font-mono text-slate-500 break-all">
              Fingerprint: {config?.serverFingerprint}
            </div>
          </div>

          <div className="p-4 bg-slate-50 dark:bg-slate-800/50 rounded-lg border border-slate-200 dark:border-slate-700/60">
            <div className="text-xs font-semibold text-slate-500 dark:text-slate-400 uppercase tracking-wider mb-1">
              Public OPAC Address
            </div>
            {config?.enabled && config?.activeInterfaces.length ? (
              <div className="space-y-1">
                {config.activeInterfaces.map((ip) => {
                  const url = `http://${ip}:${config.port}/opac`;
                  return (
                    <div key={ip} className="flex items-center justify-between font-mono text-xs bg-white dark:bg-slate-900 p-1.5 rounded border border-slate-200 dark:border-slate-700">
                      <span className="text-emerald-700 dark:text-emerald-400 select-all">{url}</span>
                      <button
                        type="button"
                        onClick={() => copyToClipboard(url, "OPAC URL")}
                        className="text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 p-1"
                        title="Copy URL"
                      >
                        <Copy className="h-3.5 w-3.5" />
                      </button>
                    </div>
                  );
                })}
              </div>
            ) : (
              <span className="text-xs text-slate-500">Enable LAN mode to generate local addresses.</span>
            )}
          </div>
        </div>
      </div>

      {/* 2. Device Pairing Section */}
      <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg p-6 shadow-sm">
        <div className="flex items-center justify-between mb-4">
          <div>
            <h3 className="text-base font-semibold text-slate-900 dark:text-slate-100 flex items-center gap-2">
              <ShieldCheck className="h-5 w-5 text-sky-600 dark:text-sky-400" />
              Device Pairing
            </h3>
            <p className="text-xs text-slate-500 dark:text-slate-400">
              Generate a single-use pairing code to authorize new computers or mobile devices.
            </p>
          </div>
          <Button
            type="button"
            variant="secondary"
            onClick={handleGenerateCode}
            disabled={!config?.enabled}
          >
            Generate Pairing Code
          </Button>
        </div>

        {pairingCode && (
          <div className="p-4 bg-sky-50 dark:bg-sky-950/40 border border-sky-200 dark:border-sky-800 rounded-lg mb-6 flex items-center justify-between">
            <div>
              <div className="text-xs font-semibold text-sky-700 dark:text-sky-300 uppercase tracking-wider mb-1">
                Active Pairing Code
              </div>
              <div className="text-3xl font-mono font-bold tracking-widest text-sky-900 dark:text-sky-100 select-all">
                {pairingCode.code}
              </div>
              <div className="text-xs text-sky-600 dark:text-sky-400 mt-1">
                Enter this code on the connecting device within 15 minutes.
              </div>
            </div>
            <Button
              type="button"
              variant="secondary"
              size="sm"
              onClick={() => copyToClipboard(pairingCode.code, "pairing code")}
              className="gap-1.5"
            >
              <Copy className="h-4 w-4" />
              Copy Code
            </Button>
          </div>
        )}

        {/* Paired Clients List */}
        <div>
          <h4 className="text-xs font-semibold text-slate-500 dark:text-slate-400 uppercase tracking-wider mb-3 flex items-center gap-2">
            <Users className="h-4 w-4" />
            Connected Devices ({clients.length})
          </h4>
          {clients.length === 0 ? (
            <div className="p-6 text-center text-sm text-slate-500 border border-dashed border-slate-200 dark:border-slate-800 rounded-lg">
              No devices paired yet. Click "Generate Pairing Code" above to pair a client terminal or tablet.
            </div>
          ) : (
            <div className="border border-slate-200 dark:border-slate-800 rounded-lg overflow-hidden">
              <table className="w-full text-left text-sm">
                <thead className="bg-slate-50 dark:bg-slate-800/60 text-xs font-medium text-slate-500 dark:text-slate-400 border-b border-slate-200 dark:border-slate-800">
                  <tr>
                    <th className="px-4 py-2.5">Device</th>
                    <th className="px-4 py-2.5">Type</th>
                    <th className="px-4 py-2.5">IP Address</th>
                    <th className="px-4 py-2.5">Paired Date</th>
                    <th className="px-4 py-2.5">Status</th>
                    <th className="px-4 py-2.5 text-right">Actions</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-100 dark:divide-slate-800">
                  {clients.map((c) => {
                    const isRevoked = Boolean(c.revokedAt);
                    return (
                      <tr key={c.clientId} className="hover:bg-slate-50/50 dark:hover:bg-slate-800/30">
                        <td className="px-4 py-3 font-medium text-slate-900 dark:text-slate-100">
                          {c.clientName}
                        </td>
                        <td className="px-4 py-3 text-slate-600 dark:text-slate-300">
                          <span className="flex items-center gap-1.5 capitalize text-xs">
                            {c.deviceKind === "tablet" || c.deviceKind === "phone" ? (
                              <Smartphone className="h-3.5 w-3.5" />
                            ) : (
                              <Laptop className="h-3.5 w-3.5" />
                            )}
                            {c.deviceKind}
                          </span>
                        </td>
                        <td className="px-4 py-3 font-mono text-xs text-slate-600 dark:text-slate-400">
                          {c.ipAddress || "—"}
                        </td>
                        <td className="px-4 py-3 text-xs text-slate-500">
                          {c.pairedAt.slice(0, 10)}
                        </td>
                        <td className="px-4 py-3">
                          <span className={`inline-flex px-2 py-0.5 text-xs font-semibold rounded-full ${isRevoked ? "bg-rose-100 text-rose-700 dark:bg-rose-950 dark:text-rose-400" : "bg-emerald-100 text-emerald-700 dark:bg-emerald-950 dark:text-emerald-400"}`}>
                            {isRevoked ? "Revoked" : "Authorized"}
                          </span>
                        </td>
                        <td className="px-4 py-3 text-right">
                          {!isRevoked && (
                            <Button
                              type="button"
                              variant="secondary"
                              size="sm"
                              onClick={() => handleRevokeClient(c.clientId, c.clientName)}
                              className="text-rose-600 dark:text-rose-400 border-rose-300 dark:border-rose-800 hover:bg-rose-50 dark:hover:bg-rose-950"
                            >
                              Revoke
                            </Button>
                          )}
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          )}
        </div>
      </div>

      {/* 3. Server Configuration Form */}
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
