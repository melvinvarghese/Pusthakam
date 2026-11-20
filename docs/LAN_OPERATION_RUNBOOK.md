# Pusthakam Local Network (LAN) Operation & Deployment Runbook

This guide covers deployment, daily operations, troubleshooting, and network resilience for Pusthakam in multi-computer library environments (Phase 7b).

---

## 1. Architecture Overview

Pusthakam operates on a lightweight, serverless, offline-first multi-computer architecture:

- **Host Workstation (Primary Server):**
  - Runs the Pusthakam desktop application.
  - Maintains the canonical SQLite database file with WAL mode enabled.
  - Hosts the embedded LAN HTTP listener on a configurable port (default `4318`).
  - Generates secure SHA-256 server identity fingerprints and single-use pairing codes.
  - Exposes public OPAC (`/` and `/opac`) and mobile staff circulation (`/staff`).
- **Paired Desktop Clients:**
  - Additional desktop computers running Pusthakam on the same local subnet.
  - Paired using a single-use 6-character code with 15-minute expiry.
  - Cryptographically verify the host's SHA-256 fingerprint before transmitting credentials.
- **Mobile / Browser Staff Clients:**
  - Tablets or smartphones used by roving library staff for shelf checking or desk circulation.
  - Access the web interface at `http://<host-ip>:4318/staff`.
  - Authenticate using individual staff credentials with Argon2id password hashing.
- **Public OPAC Terminals:**
  - Dedicated visitor kiosks, tablets, or library patrons' personal phones connected to library Wi-Fi.
  - Access `http://<host-ip>:4318/opac` anonymously without logging in.
  - Calculates real-time physical copy availability while strictly isolating private patron and financial data.

---

## 2. Host Computer Requirements & Sleep Management

> [!IMPORTANT]
> The host computer runs the database engine. If the host goes to sleep, shuts down, or disconnects from the router, all connected terminals will lose access immediately.

### Preventing Sleep on Windows:
1. Open **Windows Settings &rarr; System &rarr; Power & battery**.
2. Under **Screen and sleep**:
   - Set *When plugged in, put my device to sleep after* to **Never**.
3. In Pusthakam:
   - Ensure the app remains running in the background during library operating hours.

---

## 3. Windows Defender Firewall Setup

To allow other computers and mobile devices on the local network to connect to the Pusthakam host, an inbound firewall rule must be configured.

### Quick Setup via PowerShell (Administrator):
```powershell
New-NetFirewallRule -DisplayName "Pusthakam LAN Server" -Direction Inbound -LocalPort 4318 -Protocol TCP -Action Allow -Profile Private
```

### Manual Setup via Windows GUI:
1. Open **Control Panel &rarr; System and Security &rarr; Windows Defender Firewall**.
2. Click **Advanced settings** in the left sidebar.
3. Select **Inbound Rules &rarr; New Rule...**.
4. Choose **Port &rarr; Next**.
5. Select **TCP** and enter Specific local ports: `4318`.
6. Select **Allow the connection &rarr; Next**.
7. Check **Private** (and Domain if in an institution), uncheck **Public** for security &rarr; Next.
8. Name the rule: `Pusthakam LAN Server` and click **Finish**.

---

## 4. Wi-Fi Router Configuration & Client Isolation

In some Wi-Fi routers (especially guest networks or enterprise APs), a security feature called **"AP Isolation"**, **"Client Isolation"**, or **"Station Isolation"** prevents connected Wi-Fi devices from communicating with one another.

### How to Fix:
1. Log into your router's administrative portal (e.g. `192.168.1.1`).
2. Navigate to **Wireless Settings &rarr; Advanced**.
3. Verify that **AP Isolation / Client Isolation / Guest Isolation** is **Disabled** on the library's primary network.
4. Ensure all library computers and tablets are connected to the same Wi-Fi SSID or wired Ethernet switch.

---

## 5. Pairing Client Devices

1. On the host computer, navigate to **Settings &rarr; Local Network**.
2. Under **Pair a New Computer or Device**, click **Generate Pairing Code**.
3. A large 6-character code (e.g. `7K9X2A`) will appear with a 15-minute countdown.
4. On the client device:
   - If using a desktop client: open Pusthakam, choose **Connect to Existing Library**, enter the host IP and the pairing code.
   - If using a browser/tablet: navigate to `http://<host-ip>:4318/staff`, enter the code and assign a friendly device name (e.g., "Circulation iPad 1").
5. The device is now registered in the host's **Connected Computers & Devices** inventory.
6. The pairing code is single-use and expires immediately upon use.

---

## 6. Network Loss & Reconnection Resilience

### Handling Unexpected Disconnections:
- **Server Discovery / Reconnection:**
  - If the router restarts or network drops momentarily, the Pusthakam client will display a warning banner: *"Host connection lost. Attempting to reconnect..."*.
  - Clients retry every 5 seconds without discarding in-progress form entries.
  - Active sessions remain valid for 24 hours unless explicitly revoked.
- **Concurrency & Stale Edits:**
  - If two librarians attempt to issue the same book simultaneously on different terminals, SQLite's transactional isolation guarantees that only one transaction succeeds.
  - The second terminal receives an immediate `409 Conflict: This copy is unavailable for issue` error message, preventing double-lending.
- **Revoking Lost or Stolen Devices:**
  - If a tablet is misplaced or taken off library premises, the administrator can navigate to **Settings &rarr; Local Network** on the host and click **Revoke** next to that device.
  - All active sessions on that client are immediately invalidated, and subsequent requests are rejected with `401 Unauthorized`.

---

## 7. Daily Library Operating Checklist

| Phase | Action |
|---|---|
| **Opening** | 1. Power on Host computer and log into Windows.<br/>2. Open Pusthakam.<br/>3. Verify **Settings &rarr; Local Network** shows green "Hosting Active" badge.<br/>4. Open secondary desk terminals and staff tablets. |
| **Midday Operations** | 1. Keep Host computer plugged into AC power.<br/>2. Issue and return books normally across all terminals.<br/>3. Real-time availability updates instantly across all screens. |
| **Closing** | 1. Log out staff on roving tablets.<br/>2. On the Host computer, perform daily backup (**Settings &rarr; Backups &rarr; Create Backup**).<br/>3. Safely exit Pusthakam. |
