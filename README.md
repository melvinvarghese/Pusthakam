# Pusthakam

<p align="center">
  <img src="brand-assets/pusthakam-horizontal.svg" alt="Pusthakam — A Better Way to Manage Libraries" width="620" />
</p>

<p align="center">
  <strong>A Better Way to Manage Libraries.</strong><br />
  Free. Offline-first. Built for the people who run libraries.
</p>

<p align="center">
  <em>Windows · macOS · Linux</em>
</p>

---

**Pusthakam is a free, offline-first library management system for Windows, macOS and Linux.** It is being built to help librarians manage their collections and everyday work without depending on cloud services, recurring subscriptions or an internet connection.

Catalogue books, manage accession records, organise copies and locations, and build a dependable foundation for circulation, membership, reports and barcode workflows — all from a desktop application with locally stored data.

> **Your library. Your data. No cloud. No subscription.**

## Why Pusthakam?

- **Free to use** — no subscription fees or paid tiers are part of the product plan.
- **Works offline** — core library work is designed to run locally, without a cloud account or internet connection.
- **Your data stays local** — the application uses a local SQLite database; it does not require a hosted database.
- **Made for different libraries** — designed around one configurable core for school, college, university, public, special, research and private libraries.
- **Built for the library desk** — search-first workflows, keyboard support and USB barcode scanners.
- **Cross-platform** — Windows, macOS and Linux are the target desktop platforms.

## What it is being built to do

| Area | Planned capabilities | Current state |
|---|---|---|
| Catalogue | Bibliographic records, contributors, subjects, classification and search | Implemented; further work pending |
| Accession & inventory | Individual copies, accession numbers, barcodes, locations, shelves and copy history | Core workflows implemented |
| Import | CSV and MARC/ISO 2709 import, including a Koha-oriented profile | Implemented; real-world Koha export verification pending |
| Members | Member records, categories and cards | Planned |
| Circulation | Issue, return, renewals, reservations and loan rules | Planned |
| Reports | Library statistics and operational reports | Planned |
| Barcode & printing | Barcode labels and other library print workflows | Partially implemented |
| Acquisitions & serials | Vendors, orders, subscriptions and periodicals | Planned |
| Backup & restore | Local backup, validation and safe restoration | Planned |

Pusthakam is under active development. The catalogue and accession work has passed automated tests, but the complete application is not yet feature-complete or ready for production use. See [PROGRESS.md](PROGRESS.md) for verified status and outstanding work.

## Privacy and offline operation

Pusthakam is designed around local ownership of library data:

- No cloud database or mandatory online account.
- No subscription or recurring payment.
- No required network connection for core workflows.
- No telemetry or analytics collection.
- Library records are stored locally on the computer.

Optional external links, where present, are separate from core library operations and do not send library records.

## Technology

- **Desktop framework:** Tauri 2
- **Frontend:** React, TypeScript and Vite
- **Backend:** Rust
- **Database:** SQLite with full-text search
- **Interface:** Tailwind CSS, accessible keyboard-first components and bundled assets

## Supported platforms

The project targets:

- Windows
- macOS
- Linux

Cross-platform support is a development target; installers and end-to-end operation still need to be verified on each operating system before a stable release.

## Project status

Pusthakam is currently in development. The catalogue, accession register, copy management, search and initial import workflows are implemented. Member management and circulation are the next major development phase.

- [Development progress](PROGRESS.md)
- [Product specification](docs/PRODUCT_SPECIFICATION.md)
- [Development roadmap](docs/DEVELOPMENT_ROADMAP.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Known issues](docs/KNOWN_ISSUES.md)

## Getting started

There is no stable public release yet. Development setup instructions and required system dependencies are being maintained in [docs/KNOWN_ISSUES.md](docs/KNOWN_ISSUES.md) and [CLAUDE.md](CLAUDE.md). Please do not use an unreleased development build as the sole system of record for a live library.

## Contributing

The project is being developed in phases, with data integrity, reliable workflows, accessibility and honest feature status taking priority over adding features quickly. Bug reports, thoughtful suggestions and testing feedback are welcome.

## Brand

The name **Pusthakam** comes from the Malayalam word for book. The brand assets are in [brand-assets/](brand-assets/).

---

**Pusthakam — A Better Way to Manage Libraries.**  
Free to use. Offline-first. No cloud. No subscription.
