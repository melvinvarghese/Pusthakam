import {
  LayoutDashboard, BookOpen, ArrowLeftRight, Users, ClipboardList, Boxes, Network, Newspaper,
  ShoppingCart, BarChart3, Printer, ShieldCheck, Settings, DatabaseBackup, ScrollText,
  type LucideIcon,
} from "lucide-react";

export interface NavItem {
  path: string;
  label: string;
  icon: LucideIcon;
  /** false = module not built yet; page shows an honest "not available" state. */
  available: boolean;
  /** Phase in which the module ships (shown to the user). */
  phase: number;
}

export interface NavGroup {
  label: string;
  items: NavItem[];
}

export const NAV_GROUPS: NavGroup[] = [
  {
    label: "Main",
    items: [
      { path: "/", label: "Dashboard", icon: LayoutDashboard, available: true, phase: 1 },
      { path: "/catalogue", label: "Catalogue", icon: BookOpen, available: false, phase: 3 },
      { path: "/circulation", label: "Circulation", icon: ArrowLeftRight, available: false, phase: 4 },
      { path: "/members", label: "Members", icon: Users, available: false, phase: 4 },
    ],
  },
  {
    label: "Collections",
    items: [
      { path: "/accession", label: "Accession Register", icon: ClipboardList, available: false, phase: 3 },
      { path: "/inventory", label: "Inventory", icon: Boxes, available: false, phase: 6 },
      { path: "/classification", label: "Classification", icon: Network, available: false, phase: 3 },
      { path: "/periodicals", label: "Periodicals", icon: Newspaper, available: false, phase: 6 },
    ],
  },
  {
    label: "Operations",
    items: [
      { path: "/acquisitions", label: "Acquisitions", icon: ShoppingCart, available: false, phase: 6 },
      { path: "/reports", label: "Reports", icon: BarChart3, available: false, phase: 5 },
      { path: "/printing", label: "Barcode & Printing", icon: Printer, available: false, phase: 5 },
    ],
  },
  {
    label: "Administration",
    items: [
      { path: "/staff", label: "Staff & Permissions", icon: ShieldCheck, available: false, phase: 7 },
      { path: "/settings", label: "Library Settings", icon: Settings, available: false, phase: 2 },
      { path: "/backup", label: "Backup & Restore", icon: DatabaseBackup, available: false, phase: 7 },
      { path: "/audit", label: "Audit Log", icon: ScrollText, available: false, phase: 7 },
    ],
  },
];

export const ALL_NAV_ITEMS: NavItem[] = NAV_GROUPS.flatMap((g) => g.items);
