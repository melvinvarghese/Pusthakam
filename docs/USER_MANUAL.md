# Pusthakam — End-User Operations Manual & System Guide

_Version 1.0 · Desktop Library Management System_

---

## Table of Contents

1. [Introduction & System Principles](#1-introduction--system-principles)
2. [Installation & Platform Setup](#2-installation--platform-setup)
   - [Windows (x64)](#windows-x64)
   - [macOS (Apple Silicon & Intel)](#macos-apple-silicon--intel)
   - [Linux (Debian / Ubuntu / Fedora)](#linux-debian--ubuntu--fedora)
3. [Quick-Start Guide (5-Minute Onboarding)](#3-quick-start-guide-5-minute-onboarding)
4. [First-Run Setup Wizard](#4-first-run-setup-wizard)
5. [Catalogue & Accession Register](#5-catalogue--accession-register)
   - [Bibliographic Records](#bibliographic-records)
   - [Physical Copy Accessioning](#physical-copy-accessioning)
   - [Classification & Authority Control](#classification--authority-control)
   - [Bulk Operations & Cover Management](#bulk-operations--cover-management)
6. [Circulation Desk](#6-circulation-desk)
   - [Issuing Items](#issuing-items)
   - [Returning Items & Condition Assessment](#returning-items--condition-assessment)
   - [Renewals & Hold Reservations](#renewals--hold-reservations)
   - [Library Calendar & Closures](#library-calendar--closures)
   - [Fines Ledger & Payment/Waiver Settlement](#fines-ledger--paymentwaiver-settlement)
7. [Patron & Member Management](#7-patron--member-management)
8. [Periodicals & Serials Management](#8-periodicals--serials-management)
9. [Acquisitions & Purchase Orders](#9-acquisitions--purchase-orders)
10. [Reports & Exports](#10-reports--exports)
11. [Data Migration (Koha, MARC, CSV, XLSX)](#11-data-migration-koha-marc-csv-xlsx)
12. [Backup, Restore & Disaster Recovery](#12-backup-restore--disaster-recovery)
13. [Local Network (LAN) Multi-Device Mode](#13-local-network-lan-multi-device-mode)
14. [Code Signing & Application Upgrades](#14-code-signing--application-upgrades)
15. [Troubleshooting & Frequently Asked Questions](#15-troubleshooting--frequently-asked-questions)

---

## 1. Introduction & System Principles

Pusthakam is an offline-first desktop library management system engineered for school, college, community, and public libraries. Built on Tauri 2, Rust, SQLite, and React 19, Pusthakam delivers professional library administration without cloud dependencies, subscriptions, or telemetry.

```
+-----------------------------------------------------------------------+
|                         Pusthakam Desktop App                         |
|                                                                       |
|  +--------------------+                     +----------------------+  |
|  |  React 19 Frontend | <--- IPC Events --  | Tauri 2 Rust Backend |  |
|  |  (Accessible UI,   | --- IPC Commands -> | (Argon2id, Actix LAN,|  |
|  |   Keyboard Nav,    |                     |  PDF/XLSX Exporters, |  |
|  |   Live Announcer)  |                     |  FTS5 Search Engine) |  |
|  +--------------------+                     +----------------------+  |
|                                                         |             |
|                                                Local SQLite (WAL)     |
|                                                +--------------------+ |
|                                                | pusthakam.sqlite3  | |
|                                                | Zero Cloud Telemetry |
|                                                +--------------------+ |
+-----------------------------------------------------------------------+
```

### Core Architecture Highlights

- **Zero Remote Dependencies:** All catalogue searches, circulation rules, PDF/XLSX generation, and barcode rendering execute locally on your computer. An internet disconnection will never halt desk operations.
- **ACID-Compliant Storage:** Library data resides in a local SQLite database utilizing Write-Ahead Logging (WAL) and synchronous file locking for complete crash resilience.
- **Local Network Support (Phase 7b):** One designated desktop acts as the local library server. Staff on other laptops, tablets, or phones on the same Wi-Fi or LAN can access a browser circulation portal (`/staff`) or public OPAC (`/opac`) without touching the internet.
- **Accessible & Screen-Reader Ready:** Built-in ARIA live regions (`LiveAnnouncer`) announce loan status, issue confirmations, and catalogue search results for Windows Narrator and NVDA users.

---

## 2. Installation & Platform Setup

### Windows (x64)
- **Minimum Requirements:** Windows 10 (version 1809 or higher) or Windows 11; 4 GB RAM; 250 MB disk space.
- **Runtime Dependency:** Microsoft Edge WebView2 Evergreen Runtime (pre-installed on Windows 10/11; automatically verified on launch).
- **Installation Procedure:**
  1. Download `Pusthakam_0.1.0_x64-setup.exe` (or MSIX package).
  2. Run the installer. If Windows SmartScreen displays a warning, click **More info** &rarr; **Run anyway**.
  3. The installer creates a desktop shortcut and registers Start Menu entries.
  4. Application data is stored securely in `%APPDATA%\com.pusthakam.desktop\`.

### macOS (Apple Silicon & Intel)
- **Minimum Requirements:** macOS 12 (Monterey) or later.
- **Installation Procedure:**
  1. Download `Pusthakam-Universal.dmg`.
  2. Double-click the DMG and drag the Pusthakam icon into your **Applications** folder.
  3. On initial launch, if prompted by Gatekeeper, right-click (or Control-click) Pusthakam in Applications and select **Open**.
  4. Application data is located in `~/Library/Application Support/com.pusthakam.desktop/`.

### Linux (Debian / Ubuntu / Fedora)
- **System Dependencies:** `libwebkit2gtk-4.1`, `libssl`, `libayatana-appindicator3`.
- **Installation via `.deb`:**
  ```bash
  sudo apt-get update
  sudo apt-get install -y libwebkit2gtk-4.1-0
  sudo dpkg -i pusthakam_amd64.deb
  ```
- **Execution via AppImage:**
  ```bash
  chmod +x Pusthakam-x86_64.AppImage
  ./Pusthakam-x86_64.AppImage
  ```

---

## 3. Quick-Start Guide (5-Minute Onboarding)

Get your library operational in five rapid steps:

```
+-----------------------------------------------------------------------------+
|                               QUICK-START PATH                              |
|                                                                             |
|  [ 1. Install ] -> [ 2. Setup Wizard ] -> [ 3. Add Members ]                |
|         |                  |                       |                        |
|  Select Profile     Choose admin &       Import CSV or add by hand,         |
|  (School/College)   circulation defaults  print patron library cards        |
|                                                    |                        |
|  [ 6. Reports ] <- [ 5. Issue / Return ] <- [ 4. Accession Books ]          |
|         |                  |                       |                        |
|  Export PDF/XLSX,   Optical barcode scan  Batch add copies, assign barcodes |
|  audit ledger       instant loan check    and call numbers                  |
+-----------------------------------------------------------------------------+
```

1. **Launch & Setup:** Follow the 4-step wizard to choose your library profile (**School**, **College**, **Public**, or **Special**), set your currency, and create the administrator passphrase.
2. **Configure Policies:** Review loan rules under **Settings** &rarr; **Circulation Rules** (e.g., 14-day loan, 5 items max). Add weekly closures in **Settings** &rarr; **Calendar**.
3. **Register Members:** Add patrons in **Members** &rarr; **New Member** (or import a patron CSV). Print barcode patron cards directly via **Print Card**.
4. **Accession Books:** Add your catalogue records under **Catalogue** &rarr; **New Record**. Click **Add Copies** to generate sequential accession numbers and print label sheets with Code 128 barcodes.
5. **Begin Circulation:** Go to **Circulation** &rarr; **Issue**. Scan member card, scan book barcode, press Enter. The loan is live!

---

## 4. First-Run Setup Wizard

Upon opening Pusthakam on a newly installed machine, the setup wizard guides the head librarian through four essential steps:

1. **Library Profile Selection:**
   - Choose between **School**, **College / Academic**, **Public**, or **Special / Corporate**.
   - Presets automatically populate standard collection names (e.g., General, Reference, Fiction), item categories (Books, Periodicals, Media), and initial circulation loan rules.
2. **Library Details:**
   - Name of the institution, address, contact email/phone, currency code (`INR`, `USD`, `GBP`, `EUR`, etc.), and default locale.
3. **Head Librarian Account:**
   - Define the administrator username and a secure passphrase (minimum 8 characters).
   - Recovery instructions and administrative credentials are verified and stored using Argon2id password hashing.
4. **Initial Circulation Defaults:**
   - Default loan period (e.g., 14 days), maximum concurrent loans per patron, fine per day, and grace period.

Click **Finish Setup** to initialize the database schema and transition to the library dashboard.

---

## 5. Catalogue & Accession Register

### Bibliographic Records
- **Search & Filter:** Find records instantly by title, author, subject, ISBN, or classification number using full-text search (FTS5).
- **Adding / Editing Records:** Navigate to **Catalogue** &rarr; **New Record**. Enter mandatory title, optional subtitle, edition, publication year, publisher, ISBN-10 or ISBN-13, and Dewey or Colon classification call numbers.
- **Contributors & Subjects:** Tag multiple contributors (Author, Editor, Translator, Illustrator) and controlled subject headings.

### Physical Copy Accessioning
In Pusthakam, one bibliographic record can hold multiple physical copies:
- Click **Add Copies** on any title details view.
- Specify quantity to batch-create copies.
- For each copy, assign an **Accession Number** (sequential inventory number), a **Barcode** (for laser or CCD optical scanners), a **Shelving Location**, and an **Item Category**.
- Track physical item condition: `Available`, `On Loan`, `Reserved`, `Damaged`, `Lost`, or `Withdrawn`.

### Classification & Authority Control
- **Schedules Browser:** Browse hierarchical DDC (Dewey Decimal), UDC, LCC, and Colon Classification schedules under **Classification**.
- **Managed Authorities:** Centralized authority registries for Authors, Publishers, and Subject headings. When an author's name or subject term is updated or merged, all linked bibliographic records and search indexes update synchronously.
- **MARC Authority Mappings:** Standard 100/700 (Authors), 260/264 (Publishers), and 650 (Subjects) tag parsers ensure seamless interoperability.

### Bulk Operations & Cover Management
- **Local Cover Art:** Attach PNG or JPEG cover images (up to 2 MB) securely stored in the application data repository with path traversal protection.
- **Batch Archiving:** Select multiple records in the catalogue table to batch-archive or batch-restore with mandatory confirmation and audit logs.
- **Duplicate Detection:** Automated detection flags duplicate ISBNs, titles, or accession numbers before saving.

---

## 6. Circulation Desk

The circulation desk handles everyday patron interactions with optical barcode scanner integration.

```
+-----------------------------------------------------------------------------+
|                          CIRCULATION DESK WORKFLOW                          |
|                                                                             |
|  +--------------------+      +--------------------+      +----------------+ |
|  | 1. Scan/Enter      | ---> | 2. Eligibility &   | ---> | 3. Scan Copy   | |
|  |    Member Number   |      |    Block Checks    |      |    Barcode     | |
|  +--------------------+      +--------------------+      +----------------+ |
|                                       |                          |          |
|                              Checks: Expiry,             Checks: On shelf,  |
|                              Loan limits, Fines          Hold reservation   |
|                                                                  |          |
|  +--------------------+      +--------------------+      +----------------+ |
|  | 6. Print Due Slip  | <--- | 5. Issue Confirmed | <--- | 4. Due Date    | |
|  |    (Optional)      |      |    (Status: Loan)  |      |    Calculated  | |
|  +--------------------+      +--------------------+      +----------------+ |
|                                                          Considers Weekly   |
|                                                          Closures & Holidays|
+-----------------------------------------------------------------------------+
```

### Issuing Items
1. Navigate to **Circulation** &rarr; **Issue**.
2. Scan or enter the **Member Number** (or type patron name for live auto-complete suggestions).
3. Scan or enter the copy **Barcode** (or Accession Number).
4. Pusthakam verifies member eligibility:
   - Membership expiry date is checked.
   - Active loan count versus category loan limits is verified.
   - Outstanding unpaid fines are evaluated against blocking thresholds.
   - Existing holds/reservations by other patrons are enforced.
5. The system calculates the due date considering library operating days and calendar holidays.
6. Click **Confirm Issue** (or press Enter). The copy status changes to `On Loan`.
7. An accessible ARIA live alert announces: *"Loan issued to [Member Name]. Due date: [Date]."*

### Returning Items & Condition Assessment
1. Navigate to **Circulation** &rarr; **Return**.
2. Scan the copy **Barcode**.
3. Pusthakam displays the active loan, borrower name, and due date.
4. Select the return condition:
   - **Normal / Available:** Returns item to shelf.
   - **Damaged:** Prompts for damage assessment notes and fine levy.
   - **Lost:** Closes loan, updates inventory status to `Lost`, and bills replacement cost.
5. If overdue, the system calculates fines according to the policy in effect when the item was borrowed.
6. An accessible ARIA live alert announces: *"Copy returned. Condition: Normal."*

### Renewals & Hold Reservations
- **Renewals:** Extend loans if renewals remain under the policy limit and no other patron has placed a reservation on that title.
- **Hold Queue (Reservations):** When an item is on loan, other members can place a hold. When the item is returned, Pusthakam alerts staff, marks the reservation `Ready for Pickup`, and reserves the copy exclusively for that member for 7 days.

### Library Calendar & Closures
- Configure regular weekly closures (e.g., Sundays) and public holidays in **Settings** &rarr; **Calendar**.
- If a loan due date lands on a scheduled closure, Pusthakam automatically rolls the due date forward to the next open business day.

### Fines Ledger & Payment/Waiver Settlement
- Accessible from **Circulation** &rarr; **Fines** or the member's profile.
- Displays assessed date, overdue days, initial amount, paid amount, and outstanding balance.
- **Settlement:** Record partial or full payments via cash, card, or UPI.
- **Waivers:** Authorized staff can waive fines with required explanatory notes logged in the audit trail.

---

## 7. Patron & Member Management

- **Directory:** Search members by name, member number, email, phone, or category.
- **Categories:** Configure student, faculty, staff, or general public categories with independent loan limits (e.g., 5 books for students, 15 for faculty) and loan duration rules.
- **Member Cards:** View and print patron library cards featuring library branding, photo, barcode, and expiration date (ISO 7810 format).
- **History:** Complete ledger of active loans, borrowing history, reservations, and fine transactions.

---

## 8. Periodicals & Serials Management

Located under **Periodicals**:
- **Subscriptions:** Track active magazine and journal subscriptions with frequency (`Daily`, `Weekly`, `Monthly`, `Quarterly`, `Annual`), vendor details, and subscription date ranges.
- **Automated Issue Schedules:** Pusthakam automatically calculates expected issue dates across the calendar year.
- **Issue Check-In:** When an issue arrives, one-click check-in assigns an accession number and creates a searchable bibliographic copy.
- **Claims for Missing Issues:** Identify overdue or missing issues and generate vendor claim notices.
- **Expenditure Reconciliation:** Track yearly serials budget allocation against actual subscription and renewal expenditure.

---

## 9. Acquisitions & Purchase Orders

Located under **Acquisitions**:
- **Vendors:** Maintain a directory of book vendors, terms, and contact information.
- **Budgets:** Set annual acquisition budgets by department or subject fund.
- **Purchase Orders:** Create draft purchase orders, add line items with estimated unit prices, and advance through `Draft` &rarr; `Ordered` &rarr; `Received`.
- **Transactional Receiving:** When physical shipments arrive, receive items directly into the catalogue and accession register, allocating barcodes and accession numbers in one atomic operation.

---

## 10. Reports & Exports

Pusthakam includes built-in reporting and vector document generation:
- **Dashboard:** Real-time metrics for total catalogue records, physical copies, active loans, overdue items, and recent circulation trends.
- **Standard Reports:**
  - Overdue loans with patron contact details.
  - Accession register with purchase date, vendor, price, and location.
  - Fine assessment and collection reconciliation.
  - Monthly catalogue growth and acquisition expenditure.
  - Lost and damaged stock audit.
- **Visual Presentation & "Simple Report Design" Mode:**
  - By default, reports display with rich typography, shaded headers, alternating zebra striping, and accented totals.
  - Check the **Use simple report design** box for economical black-and-white printing and minimal styling. Both modes retain identical calculations and data.
- **Export Formats:**
  - **Vector PDF 1.4:** Multi-page formatted tables with page numbering and headers (no third-party PDF tools needed).
  - **OpenXML XLSX:** Clean spreadsheet tables with formatted columns compatible with Microsoft Excel and LibreOffice Calc.
  - **CSV:** Raw comma-separated data for custom scripting.

---

## 11. Data Migration (Koha, MARC, CSV, XLSX)

Pusthakam provides an interactive, transactional migration wizard under **Settings** &rarr; **Import / Export**:

1. **Supported Formats:**
   - **Koha 952 MARC / MARCXML:** Imports bibliographic records and extracts Koha item holdings (952 tags: barcode `$p`, call number `$o`, location `$c`, item type `$y`).
   - **Standard MARC21 / ISO 2709 & MARCXML:** Full bibliographic record import.
   - **CSV & XLSX Workbooks:** Interactive column mapper mapping title, author, ISBN, barcode, and category.
   - **Patron CSV:** Migrates member rosters with existing card numbers.
   - **Circulation History Bundles:** Imports historical closed loans, active checkouts, and fine balances.
2. **Safety Guarantees:**
   - **Preview Phase:** Validates records, detects duplicates, and identifies formatting errors before writing to disk.
   - **Transactional Rollback:** Any database failure during import triggers a complete rollback, leaving existing data untouched.

---

## 12. Backup, Restore & Disaster Recovery

Located under **Settings** &rarr; **Backup & Restore**:

```
+-----------------------------------------------------------------------------+
|                    SAFETY-FIRST RESTORE CYCLE                               |
|                                                                             |
|  [ Select Backup File ] ----------------------------------------->          |
|           |                                                      |          |
|  [ Acquire Exclusive Lock ]                                      |          |
|           |                                                      |          |
|  [ Checkpoint WAL & Terminate Active Tasks ]                     |          |
|           |                                                      |          |
|  [ Create Automatic Pre-Restore Safety Snapshot ]                |          |
|           |                                                      |          |
|  [ Overwrite Database with Backup ]                              |          |
|           |                                                      |          |
|    Verification Check                                            |          |
|    /              \                                              |          |
|  Success        Failure                                          |          |
|    |               |                                             |          |
| [ Reopen DB ]   [ Automatic Rollback to Pre-Restore Snapshot ]   |          |
+-----------------------------------------------------------------------------+
```

- **Automated Backups:** Pusthakam creates scheduled rolling daily snapshots stored in `%APPDATA%\com.pusthakam.desktop\backups\`.
- **Manual Backups:** Click **Create Backup Now** to produce a timestamped SQLite snapshot. Use **Copy to Folder** to save a copy to a USB flash drive or external backup drive.
- **Exclusive Restore:** Restoring a database requires exclusive access. Pusthakam places an exclusive lock, safely terminates WAL checkpoints, creates an automatic pre-restore safety snapshot of the active database, applies the backup, and reloads state. If an issue occurs, the pre-restore snapshot ensures zero data loss.

---

## 13. Local Network (LAN) Multi-Device Mode

When multiple staff members need to work simultaneously from different computers, or when you want to provide a public catalogue for visitors:

```
+-----------------------------------------------------------------------------+
|                      LOCAL NETWORK (LAN) TOPOLOGY                           |
|                                                                             |
|                      +------------------------+                             |
|                      |   Host Library PC      |                             |
|                      |  Pusthakam Desktop     |                             |
|                      |  Embedded Server :4318 |                             |
|                      +------------------------+                             |
|                                  |                                          |
|                       Local Wi-Fi or Ethernet                               |
|                     (No Internet Access Needed)                             |
|                                  |                                          |
|        +-------------------------+-------------------------+                |
|        |                                                   |                |
|  +-------------------+                               +-------------------+  |
|  |  Staff Terminal   |                               |  Public OPAC      |  |
|  |  (Tablet/Laptop)  |                               |  (Kiosk/Phone)    |  |
|  |  http://ip:4318   |                               |  http://ip:4318   |  |
|  |  /staff (Pairing) |                               |  /opac (Read-only)|  |
|  +-------------------+                               +-------------------+  |
+-----------------------------------------------------------------------------+
```

### Setting Up the Host PC
1. Select one computer with a reliable network connection as the **Library Host**.
2. Go to **Settings** &rarr; **Local Network**.
3. Toggle **Enable Local Network Hosting**.
4. Pusthakam binds an embedded HTTP server to your local IP address (e.g., `http://192.168.1.15:4318`).
5. Toggle **Enable Browser Staff Access** to allow staff on phones/tablets to issue and return books.
6. Toggle **Enable Public OPAC** to allow library patrons to search the catalogue.

### Pairing Devices & Staff Login
- To connect a mobile or secondary computer:
  1. On the host desktop, click **Generate Pairing Code** (produces a single-use 6-character code valid for 15 minutes).
  2. On the remote tablet/phone, open a browser and go to `http://<HOST-IP>:4318/staff`.
  3. Enter the pairing code and log in with your staff username and password.
  4. Each staff member receives an independent, encrypted session token.
- **Concurrent Conflict Protection:** If two staff terminals attempt to issue the same physical copy simultaneously, Pusthakam's transactional locking immediately rejects the second request with `409 Conflict: This copy is unavailable for issue`, preventing double lending.
- **Public OPAC:** Visitors visit `http://<HOST-IP>:4318/` or `/opac` to search titles and see real-time availability. All member records, loans, fines, and private notes are strictly isolated.

---

## 14. Code Signing & Application Upgrades

### Code Signing Architecture
To ensure security and eliminate platform warnings:
- **Windows:** Packages are signed using a standard Microsoft Authenticode certificate via `signtool.exe`:
  ```powershell
  signtool sign /fd SHA256 /a /tr http://timestamp.digicert.com /td SHA256 Pusthakam-Setup.exe
  ```
- **macOS:** Signed using Apple Developer ID Application credentials and notarized with Apple Notary Service:
  ```bash
  codesign --deep --force --options runtime --sign "Developer ID Application: Your Name (TEAM_ID)" Pusthakam.app
  xcrun notarytool submit Pusthakam.dmg --keychain-profile "notary-profile" --wait
  ```
- **Linux:** Distribute release tarballs and `.deb` files accompanied by SHA-256 checksums and GPG detached signatures.

### Upgrade Procedure
Upgrading Pusthakam is seamless and non-destructive:
1. Run the installer for the new version.
2. The installer updates application binaries in the installation directory.
3. The database at `%APPDATA%\com.pusthakam.desktop\pusthakam.sqlite3` is preserved.
4. On the first launch of the updated version, Pusthakam's internal migration engine automatically executes any new SQL schema migrations in a single transaction.

---

## 15. Troubleshooting & Frequently Asked Questions

### Q: Does Pusthakam require an internet connection?
**A:** No. Pusthakam is 100% offline-first. All barcode rendering, PDF/XLSX generation, search indexing, and circulation rules run locally.

### Q: Can other computers reach the server in LAN mode?
**A:** Ensure that:
1. Both devices are connected to the same Wi-Fi router or Ethernet switch.
2. Windows Defender Firewall allows incoming connections on port 4318 (see `docs/LAN_OPERATION_RUNBOOK.md`).
3. Your Wi-Fi router does not have "AP Isolation" or "Client Isolation" turned on in its wireless settings.

### Q: How do I move my library data to a new computer?
**A:**
1. On the old PC, open **Settings** &rarr; **Backup & Restore** and click **Create Backup Now**.
2. Click **Copy to Folder** and choose a USB drive.
3. Install Pusthakam on the new PC and complete initial setup.
4. Go to **Settings** &rarr; **Backup & Restore**, select the backup file from your USB drive, and click **Restore**.

### Q: What barcode scanner hardware is supported?
**A:** Any standard USB or Bluetooth barcode scanner operating in "Keyboard Wedge" (HID) mode. When scanned, it inputs the barcode characters followed by an Enter key.

---
_Pusthakam — A better way to manage libraries._
