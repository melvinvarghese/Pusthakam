<p align="center">
  <img src="branding/pusthakam-logo.png" alt="Pusthakam logo" width="96" />
</p>

<h1 align="center">Pusthakam</h1>

<p align="center">
  <strong>A Better Way to Manage Libraries.</strong><br />
  Offline-first. One-time purchase. Built for the people who run libraries.
</p>

<p align="center">
  <em>Windows · macOS · Linux</em>
</p>

---

**Pusthakam** is a fast, offline-first desktop library management system for schools, colleges, and community libraries. Everything runs on your library's own computer—no monthly subscriptions, no mandatory cloud accounts, and no internet connection required for daily library work.

> **Your library. Your data. No cloud. No monthly subscription.**

---

## Why Libraries Choose Pusthakam

- **Works completely offline** — Issue, return, search, and manage your library even when the internet is down.
- **Your data stays with you** — All books, members, and loan records are stored safely on your own computer, with automatic daily backups.
- **Saves typing** — Scan a barcode or type an ISBN to auto-fill book details (when online), or import your existing catalogue and patrons from **Koha**, Excel, or CSV.
- **Built for the library desk** — Works with standard USB barcode scanners, keyboard shortcuts, and ordinary office/label printers.
- **Multi-device access on your Wi-Fi** — Let students and visitors search your **Public Catalogue (OPAC)** from their phones, and let staff work from tablets or other computers on the same network.

---

## What Pusthakam Does

| Area | How it helps your library |
|---|---|
| **Catalogue & ISBN Auto-Fill** | Add books quickly with **Fill from ISBN**, search by title, author, subject, or call number, and save common searches. |
| **Accession & Barcodes** | Track every individual copy with its accession number, barcode, shelf location, and condition history. |
| **Circulation & Holds** | Issue, return, and renew books by barcode scanner or book title; manage hold queues, library holidays, and overdue fines. |
| **Overdue Reminders** | Send optional **Email** or **WhatsApp** overdue notices (or copy a ready message) directly from the Overdue list. |
| **Members & ID Cards** | Register members, track membership expiry and fine balances, upload photos, and print library member cards. |
| **Acquisitions & Periodicals** | Manage book purchase requests, vendors, and orders, plus magazine, journal, and newspaper issues. |
| **Inventory (Stock Check)** | Scan books on the shelf to quickly spot missing or misplaced copies. |
| **Reports & Printing** | Export circulation, popular titles, and stock reports to **PDF, Excel (XLSX), or CSV**, and print barcode/spine labels and due slips. |
| **Public Catalogue (OPAC)** | Visitors can search books, check shelf locations and availability, and sign in with their member number + phone/email to view loans, fines, and request holds. |
| **Backups & Staff Roles** | Create and restore verified backups, assign staff roles (Administrator, Librarian, Circulation, Read-only), and review the audit log. |

---

## Privacy & Offline Operation

Pusthakam is built around local ownership of your library's records:

- **No cloud database** — Library records stay on your computer.
- **No telemetry or tracking** — Pusthakam does not collect usage analytics.
- **Optional online helpers only** — Features like **Fill from ISBN**, online book covers, and **Email / WhatsApp** reminders only contact external services when you explicitly use them, and never send your library's database.

---

## Support & Creator

Pusthakam is created and maintained by **[Melvin Varghese](https://melvinvarghese.in/support/?utm_source=pusthakam&utm_medium=readme&utm_campaign=github_readme)** and named after the Malayalam word for *book*.

For support, licensing inquiries, or documentation, visit **[melvinvarghese.in/support](https://melvinvarghese.in/support/?utm_source=pusthakam&utm_medium=readme&utm_campaign=github_readme)**.

---

<details>
<summary><strong>For Developers: Project Architecture, Builds & Progress Monitor</strong></summary>

### Technology Stack
- **Desktop shell:** Tauri 2 (Rust backend + embedded webview)
- **Frontend:** React, TypeScript, Vite, Tailwind CSS
- **Database:** Local SQLite with FTS5 full-text search
- **Brand assets:** Approved raster master in [branding/pusthakam-logo.png](branding/pusthakam-logo.png) (see [branding/README.md](branding/README.md))

### Standalone Windows Builds
- Build standalone app & NSIS installer: `npm.cmd run build:windows`
- Build standalone debug executable: `npm.cmd run build:standalone-debug`
- *(A plain `cargo build` produces a dev binary expecting Vite on `127.0.0.1:1420`; use Tauri CLI commands above for standalone testing.)*

### Browser Access & Hosting
- The embedded HTTP server serves the public catalogue at `/opac` (plus patron endpoints under `/api/opac/patron/*`) and the staff portal at `/staff`.
- Enable hosting in **Settings → Wi-Fi / LAN**, or run the standalone `pusthakam-server` binary for an always-on host. See [docs/WEB_HOSTING.md](docs/WEB_HOSTING.md) and [docs/STAFF_BROWSER_PARITY.md](docs/STAFF_BROWSER_PARITY.md).

### Local Progress Monitor
- Start the local progress dashboard: `npm run progress:monitor` (opens at `http://127.0.0.1:4317/`)
- Validate progress data: `npm run progress:validate` (use `npm.cmd` in Windows PowerShell if scripts are restricted)

### Key Documentation
- [Development progress](PROGRESS.md)
- [Pending work checklist](docs/PENDING_WORK.md)
- [Product specification](docs/PRODUCT_SPECIFICATION.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Known issues](docs/KNOWN_ISSUES.md)

</details>
