export interface NavItem {
  path: string;
  label: string;
  /** false = module not built yet; page shows an honest "not available" state. */
  available: boolean;
  /** Phase in which the module ships (shown to the user). */
  phase: number;
  /** What the module will do, from the product specification. */
  summary: string;
}

export interface NavGroup {
  label: string;
  items: NavItem[];
}

export const NAV_GROUPS: NavGroup[] = [
  {
    label: "Main",
    items: [
      { path: "/", label: "Dashboard", available: true, phase: 1, summary: "Overview of the library: records, copies, loans, members, overdue items and alerts." },
      { path: "/catalogue", label: "Catalogue", available: true, phase: 3, summary: "Catalogue books with authors, subjects and classification, and search the whole collection." },
      { path: "/circulation", label: "Circulation", available: false, phase: 4, summary: "Issue, return, renew and reserve books, with due dates, overdue tracking and fines." },
      { path: "/members", label: "Members", available: false, phase: 4, summary: "Register members, print membership cards, and see each member's loans and fines." },
    ],
  },
  {
    label: "Collections",
    items: [
      { path: "/accession", label: "Accession Register", available: true, phase: 3, summary: "Track every physical copy by accession number, barcode, location and status." },
      { path: "/inventory", label: "Inventory", available: false, phase: 6, summary: "Verify stock against the shelf and record lost, damaged and withdrawn copies." },
      { path: "/classification", label: "Classification", available: false, phase: 6, summary: "Dewey, UDC, Library of Congress and custom classification schemes, subjects and authorities." },
      { path: "/periodicals", label: "Periodicals", available: false, phase: 6, summary: "Journals, magazines and newspapers: subscriptions, expected and received issues." },
    ],
  },
  {
    label: "Operations",
    items: [
      { path: "/acquisitions", label: "Acquisitions", available: false, phase: 6, summary: "Purchase requests, approvals, vendors, orders, invoices and receiving new stock." },
      { path: "/reports", label: "Reports", available: false, phase: 5, summary: "Circulation, catalogue, accession and fine reports, exportable to PDF, CSV and XLSX." },
      { path: "/printing", label: "Barcode & Printing", available: false, phase: 5, summary: "Barcode labels, spine labels, member cards and receipts." },
    ],
  },
  {
    label: "Administration",
    items: [
      { path: "/staff", label: "Staff & Permissions", available: false, phase: 7, summary: "Staff accounts and role-based permissions." },
      { path: "/settings", label: "Library Settings", available: true, phase: 2, summary: "Library profile, collections, locations, member categories and circulation policies." },
      { path: "/backup", label: "Backup & Restore", available: false, phase: 7, summary: "Scheduled backups, verified restore and data integrity checks." },
      { path: "/audit", label: "Audit Log", available: false, phase: 7, summary: "A permanent record of important actions by staff." },
    ],
  },
];

export const ALL_NAV_ITEMS: NavItem[] = NAV_GROUPS.flatMap((g) => g.items);
