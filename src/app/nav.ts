import { useQuery } from "@tanstack/react-query";
import { appApi } from "@/lib/api";

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
      { path: "/circulation", label: "Circulation", available: true, phase: 4, summary: "Issue, return and renew loans by barcode, with due dates based on library loan rules." },
      { path: "/members", label: "Members", available: true, phase: 4, summary: "Register members, find them by name or member number, and manage their membership." },
    ],
  },
  {
    label: "Collections",
    items: [
      { path: "/accession", label: "Accession Register", available: true, phase: 3, summary: "Track every physical copy by accession number, barcode, location and status." },
      { path: "/inventory", label: "Inventory", available: true, phase: 6, summary: "Scan copies and review matched, missing and unexpected stock." },
      { path: "/classification", label: "Classification", available: false, phase: 6, summary: "Dewey, UDC, Library of Congress and custom classification schemes, subjects and authorities." },
      { path: "/periodicals", label: "Periodicals", available: false, phase: 6, summary: "Journals, magazines and newspapers: subscriptions, expected and received issues." },
    ],
  },
  {
    label: "Operations",
    items: [
      { path: "/acquisitions", label: "Acquisitions", available: true, phase: 6, summary: "Purchase requests, approvals, vendors, orders, invoices and receiving new stock." },
      { path: "/reports", label: "Reports", available: true, phase: 5, summary: "Review circulation activity and most borrowed titles by date, and export to CSV." },
      { path: "/printing", label: "Barcode & Printing", available: false, phase: 5, summary: "Barcode labels, spine labels, member cards and receipts." },
    ],
  },
  {
    label: "Administration",
    items: [
      { path: "/staff", label: "Staff & Permissions", available: true, phase: 7, summary: "Manage staff accounts and roles, or change your password." },
      { path: "/settings", label: "Library Settings", available: true, phase: 2, summary: "Library profile, collections, locations, member categories and circulation policies." },
      { path: "/backup", label: "Database Backups", available: true, phase: 7, summary: "Create checked database snapshots and review saved backups." },
      { path: "/audit", label: "Audit Log", available: true, phase: 7, summary: "Search permanent staff and system events by date, staff name and action." },
    ],
  },
];

export const ALL_NAV_ITEMS: NavItem[] = NAV_GROUPS.flatMap((g) => g.items);

export const OPTIONAL_MODULES = [
  { path: "/acquisitions", label: "Acquisitions" },
  { path: "/periodicals", label: "Periodicals" },
] as const;

export function moduleVisibilityOptions(path: string) {
  return {
    queryKey: ["module-visibility", path],
    queryFn: async () => (await appApi.setting(`modules.${path.slice(1)}`)) !== "false",
    retry: false,
  };
}

export function useVisibleNavGroups() {
  const acquisitions = useQuery(moduleVisibilityOptions("/acquisitions"));
  const periodicals = useQuery(moduleVisibilityOptions("/periodicals"));
  return NAV_GROUPS.map((group) => ({
    ...group,
    items: group.items.filter(({ path }) =>
      !(path === "/acquisitions" && acquisitions.data === false)
      && !(path === "/periodicals" && periodicals.data === false)),
  }));
}
