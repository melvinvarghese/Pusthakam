# OPAC and staff hosting

The library PC remains the default host. Settings → Wi-Fi / LAN controls its listener and shows the addresses to share with computers and phones on the same network. Wi-Fi and wired clients use the same service. Keep the PC and Pusthakam running. `127.0.0.1` always refers to the device opening the address; share the displayed host IP instead.

Hosting serves the shared staff application when browser staff access is enabled; web.full_staff_enabled=false is an operator rollback to the legacy portal. Use the current staff-web installer; earlier cover-only installers do not contain this work. Do not expose the legacy staff login API to the internet.

## Always-on host

Build the bundled staff assets before compiling the server:

```powershell
npm.cmd run build
cargo build --manifest-path src-tauri/Cargo.toml --release --bin pusthakam-server
```

Run the console host with a persistent directory outside the repository, temporary folders and cloud-synced folders:

```powershell
.\pusthakam-server.exe --data-dir "D:\LibraryData" --bind 127.0.0.1 --port 4318
```

First setup must use `http://127.0.0.1:4318/staff` on the server itself. Existing libraries retain their users, roles and passwords. This host uses the same SQLite migrations, pre-upgrade snapshots, pending-restore process, backup scheduler and command services as the desktop application. Type `quit` in its console for a shutdown that stops accepting requests and waits for current requests to finish. A service manager may supervise the console process; service installation and target-platform packaging remain deployment work.

For same-network access, choose `--bind 0.0.0.0` and use the server's actual private IP. Permit the selected application/port on the host's private-network firewall when required. A network with client isolation can prevent devices from reaching each other even on the same Wi-Fi.

## HTTPS address from anywhere

Both the library PC and an always-on host can sit behind an operator-managed HTTPS reverse proxy. Configure the domain and certificate on that proxy. Configure the same HTTPS origin in Pusthakam's Public HTTPS address setting, or pass `--public-url https://catalogue.example.org` to the console host. Entering this setting does not create a domain, certificate, tunnel or internet connection.

The supported proxy boundary is on the same machine, connecting through loopback. The staff gateway accepts the public host only with `X-Forwarded-Proto: https` from loopback. Public cookies are Secure, HttpOnly, SameSite=Strict and scoped to the host. The proxy must replace forwarding headers with values it establishes itself. Keep the backend port private.

Example Caddy configuration (replace the example domain with your domain):

```caddyfile
catalogue.example.org {
    @app path / /opac /staff /staff/ /assets/* /api/opac/* /api/staff/commands /api/staff/backups/*
    handle @app {
        reverse_proxy 127.0.0.1:4318 {
            request_buffers 140MiB
            header_up Host {hostport}
            header_up X-Forwarded-Proto https
            header_up X-Forwarded-For {remote_host}
        }
    }
    handle {
        respond "Not found" 404
    }
}
```

The proxy must buffer uploads and forward a valid Content-Length; the backend deliberately rejects chunked bodies, duplicate headers and oversized requests. Anonymous commands are limited to 16 KiB; authenticated uploads to 140 MiB, with two large uploads at a time. Import file limits may be lower. Do not forward `/api/lan/*` or the legacy staff login/issue/return routes publicly. A publicly reachable PC needs an appropriate ingress connection; a private IP alone cannot provide that.

The buffering/header options were checked against the [official Caddy reverse proxy reference](https://caddyserver.com/docs/caddyfile/directives/reverse_proxy). This template still requires validation on the deployed proxy/version, including upload limits and host resource capacity; it has not been deployed against a public domain here.

## Moving the authoritative host

1. Stop new circulation work and create, download and verify a backup on the current host. Keep an additional independent copy.
2. Copy the verified snapshot into the destination host backup folder using local operator access, then queue its restore through Pusthakam's backup workflow. Restart, verify the restored library, users, holdings and current loans, and retain the pre-restore snapshot.
3. Test OPAC and staff access, role restrictions, printing/downloads and a new backup before distributing the destination URL.
4. Stop writes on the old host. To roll back, stop destination writes first and restore a verified up-to-date backup on the chosen authoritative host.

There is one writable library database. This does not synchronize a library-PC database with a second hosted database. Never open a live SQLite database over a shared network drive. Remote-device tests, actual HTTPS ingress, printing hardware and restore acceptance are required before production rollout.

## Windows host: verified local firewall diagnosis (25 November 2026)

The inspected running host listens on all IPv4 interfaces at port 4318, and both portals respond using its Wi-Fi address. Windows currently classifies that Wi-Fi as Public and has an enabled Public TCP Block rule for that exact executable, with its matching Allow rule limited to Private networks. A same-host request cannot prove remote reachability.

For a trusted home/library network only, tools/Enable-PusthakamPrivateNetwork.ps1 is a reviewable administrator repair: it marks the selected interface Private and creates a Pusthakam-program/TCP-port rule scoped to LocalSubnet and Private. It does not disable the firewall, delete existing blocks or publish the server on the internet. Network profile changes affect other applications, so confirm network trust first. The current agent process is not an administrator; the script has not been applied. Use -WhatIf to preview from an administrator PowerShell.

Browser access needs no pairing code. Staff use their own accounts; OPAC visitors sign in only if a future patron feature explicitly requires it. Different-network access still requires an actual HTTPS ingress/domain or hosted server. No public address is currently configured on the inspected host.

Trusted-network confirmation received. Administrator repair launch failed with Windows0xc0000142; it was not applied. User should set the connected trusted Wi-Fi profile to Private using Windows Settings, then test the displayed address on the actual second device. New pairing-free installer delivery is pending the separate telemetry approval/build blocker; existing installer still contains the old settings card.
