# Product

<!-- impeccable:product-schema 1 -->

*Source: the owner's master brief and session answers on 2026-11-15; no separate product interview was held. Facts below are brief-derived, not separately confirmed.*

## Platform

web

(Tauri 2 desktop app: the UI is React rendered in an OS webview; Windows, macOS, Linux. Design language is the app's own, not a native-OS skin. Desktop only; minimum window 1024x640.)

## Users

Librarians and library staff of every kind of library (school, college, university, public, special, research, institutional, private collections). They work at a circulation desk or cataloguing bench for hours a day: issuing and returning books with a USB barcode scanner (keyboard input), looking members up, cataloguing new stock, running reports, printing labels and receipts. Many are not technical; some run a library alone. Speed, accuracy and calm under queue pressure matter more than novelty. The owner is a non-programmer and judges the product visually.

## Product Purpose

Pusthakam (Malayalam: book) is a fully offline library management system: catalogue, accession, circulation, members, acquisitions, serials, reports, printing, backup. Success: a librarian installs it, configures their library, and runs ordinary daily operations confidently, with data that is never lost.

## Positioning

Completely offline (local SQLite, no cloud, no subscription, no telemetry) and built for every library type from one configurable core, with an interface that feels like premium professional software rather than a dated library system or a generic admin dashboard.

## Operating Context

Desk use with a barcode scanner acting as a keyboard; keyboard-first, search-first workflows; dense tables of records; long sessions; printed output (labels, member cards, receipts, registers). Libraries often have modest hardware and old monitors. Data is local; trust and recoverability (backup/restore, audit log) are core rituals.

## Capabilities and Constraints

Modules: Dashboard, Catalogue, Circulation, Members, Accession Register, Inventory, Classification, Periodicals, Acquisitions, Reports, Barcode & Printing, Staff & Permissions, Library Settings, Backup & Restore, Audit Log. Only the Dashboard shell exists today; others are built phase by phase and show an honest "not available yet" state until built. Light and dark themes required. No remote fonts, CDNs or network assets. Malayalam word may appear as an optional brand element but nothing may depend on Malayalam fonts. Bundled fonts allowed via npm. Technical constraints: Tailwind CSS v4 tokens in `src/styles/index.css`, Lucide icons, accessible keyboard navigation.

## Brand Commitments

Name "Pusthakam" (working name, never changed or abbreviated), Malayalam form പുസ്തകം (owner asked on 2026-11-15 that it not appear in the app interface), tagline "A Better Way to Manage Libraries." The first built UI (ivory/sage Inter sidebar shell) was explicitly rejected by the owner as generic "AI slop"; treat it as an anti-reference. The owner wants it to read as made by the world's best designers, programmers and librarians. Brief palette ideas (charcoal, ivory, sand, sage, grey) were suggestions, not binding.

## Evidence on Hand

No real library data, logo, customers or screenshots exist. Any sample content shown in design previews must be clearly synthetic and never ship in production paths. No testimonials, figures or claims to fabricate.

## Product Principles

1. Data integrity and trust over decoration: the interface must make safety (backups, history, confirmations) visible and calm.
2. The desk comes first: scanner-driven, keyboard-first, scannable at a glance, fast under a queue.
3. One configurable core for all libraries; never a per-type fork.
4. Honest software: no fake data, no dead buttons, no unfinished module dressed as finished.
5. Offline is a feature to be felt, not apologised for.

## Accessibility & Inclusion

WCAG 2.2 AA contrast and full keyboard operability; visible focus; users may be non-technical and of varied ages and screen quality; Malayalam and English content may coexist in records.
