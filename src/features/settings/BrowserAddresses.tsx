import type { LanConfig } from "@/lib/api/types";
import { Button } from "@/components/ui/Button";

interface Props {
  config: LanConfig | null;
  onCopy: (text: string, label: string) => void;
  onRefresh: () => void;
}

export function BrowserAddresses({ config, onCopy, onRefresh }: Props) {
  if (!config?.enabled || !config.listenerRunning) {
    return <p className="text-sm text-muted">Enable Wi-Fi / LAN hosting and wait for the server to start to see the OPAC address.</p>;
  }
  const bind = config.bindAddress;
  const loopback = (ip: string) => ip.startsWith("127.") || ip === "::1" || ip === "localhost";
  const addresses = [...new Set(config.activeInterfaces)].filter(ip =>
    !loopback(ip) && ip !== "0.0.0.0" && ip !== "::" &&
    (bind === "0.0.0.0" ? !ip.includes(":") : bind === "::" ? ip.includes(":") : ip === bind),
  );
  const local = bind === "0.0.0.0" ? "127.0.0.1" : bind === "::" ? "::1" : bind;
  const portals = [config.opacEnabled ? "opac" : null, config.browserStaffEnabled ? "staff" : null].filter((value): value is string => value !== null);
  const url = (ip: string, portal: string) => `http://${ip.includes(":") ? `[${ip}]` : ip}:${config.port}/${portal}`;
  const links = (ip: string) => portals.map(portal => {
    const address = url(ip, portal);
    const name = portal === "opac" ? "OPAC" : "Staff";
    return <div key={address} className="space-y-1">
      <span className="text-sm font-medium">{name}</span>
      <div className="flex flex-wrap items-center gap-2">
        <span className="min-w-0 break-all font-mono text-sm select-all">{address}</span>
        <Button type="button" size="sm" variant="secondary" onClick={() => onCopy(address, `${name} address`)} aria-label={`Copy ${name} address ${ip}`}>Copy address</Button>
      </div>
    </div>;
  });
  let publicOrigin: string | null = null;
  try { if (config.publicBaseUrl) { const parsed = new URL(config.publicBaseUrl); if (parsed.protocol === "https:" && !parsed.username && !parsed.password) publicOrigin = parsed.origin; } } catch { /* Invalid stored addresses are not advertised. */ }
  return <div className="space-y-4">
    {publicOrigin && <section aria-label="Configured internet addresses" className="space-y-2">
      <h4 className="font-semibold">Internet addresses (configured)</h4>
      {portals.map(portal => {
        const address = `${publicOrigin}/${portal}`;
        return <div key={portal} className="flex flex-wrap items-center gap-2"><span className="min-w-0 break-all font-mono text-sm select-all">{address}</span><Button type="button" size="sm" variant="secondary" onClick={()=>onCopy(address,`${portal} internet address`)}>Copy internet {portal} address</Button></div>;
      })}
      <p className="text-xs text-muted">Use these after your HTTPS host is configured and reachable. This display does not confirm internet connectivity.</p>
    </section>}
    {!publicOrigin && <section aria-label="Internet access" className="space-y-2">
      <h4 className="font-semibold">Access from other networks: not set up</h4>
      <p className="text-sm text-muted">The local IP address works only on the library network. Access from mobile data or another Wi-Fi needs a public HTTPS address connected to this host or an always-on server.</p>
    </section>}
    <section aria-label="Open on other devices" className="space-y-3">
      <h4 className="font-semibold">Open on other devices on the same Wi-Fi</h4>
      {loopback(bind) ? <p role="alert" className="text-sm text-danger">Hosting is limited to this computer. Select “Same Wi-Fi or wired network” under Network access and save to allow other devices.</p> : addresses.length ? addresses.map(ip =>
        <div key={ip} className="space-y-2">
          <p className="text-sm">Library computer IP: <strong className="font-mono">{ip}</strong></p>
          {links(ip)}
        </div>,
      ) : <p className="text-sm text-muted">No shared network address detected. Connect this computer to the library Wi-Fi or wired network, then refresh addresses.</p>}
      {addresses.length > 0 && <p className="text-sm text-muted">Copy the complete OPAC address into the other device’s browser. Both devices must use the same Wi-Fi or wired network. Keep Pusthakam running on this computer.</p>}
    </section>
    <section aria-label="On this computer" className="space-y-2 border-t border-line pt-3">
      <h4 className="font-semibold">On this computer</h4>
      {links(local)}
      {loopback(local) && <p className="text-xs text-muted">This address works only on this computer. Do not use it on phones or other computers.</p>}
    </section>
    {!portals.length && <p className="text-sm text-muted">Enable Public OPAC to show its address.</p>}
    <Button type="button" size="sm" variant="secondary" onClick={onRefresh}>Refresh addresses</Button>
  </div>;
}
