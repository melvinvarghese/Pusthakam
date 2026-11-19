import { lazy, Suspense } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { HashRouter, Route, Routes } from "react-router-dom";
import { AppShell } from "@/app/AppShell";
import { ALL_NAV_ITEMS } from "@/app/nav";
import { Gate } from "@/features/boot/Gate";
import { BackupPage } from "@/features/backup/BackupPage";
import { AuditPage } from "@/features/audit/AuditPage";
import { StaffPage } from "@/features/staff/StaffPage";
import { InventoryPage } from "@/features/inventory/InventoryPage";
import { AccessionPage } from "@/features/catalogue/AccessionPage";
import { CataloguePage } from "@/features/catalogue/CataloguePage";
import { RecordPage } from "@/features/catalogue/RecordPage";
import { Dashboard } from "@/features/dashboard/Dashboard";
import { CirculationPage } from "@/features/circulation/CirculationPage";
import { MembersPage } from "@/features/members/MembersPage";
import { NotAvailable } from "@/features/NotAvailable";
import { SettingsPage } from "@/features/settings/SettingsPage";
import { ThemeProvider } from "@/lib/theme";

const queryClient = new QueryClient();
const ReportsPage = lazy(() => import("@/features/reports/ReportsPage"));
const AcquisitionsPage = lazy(() => import("@/features/acquisitions/AcquisitionsPage"));

// Development-only visual reference; `import.meta.env.DEV` is false in production builds, so it is never bundled.
const DesignGallery = import.meta.env.DEV
  ? lazy(() => import("@/features/dev/DesignGallery").then((m) => ({ default: m.DesignGallery })))
  : null;

export function AppRoutes() {
  return (
    <Routes>
      <Route element={<AppShell />}>
        <Route index element={<Dashboard />} />
        <Route path="/settings" element={<SettingsPage />} />
        <Route path="/backup" element={<BackupPage />} />
        <Route path="/audit" element={<AuditPage />} />
        <Route path="/staff" element={<StaffPage />} />
        <Route path="/catalogue" element={<CataloguePage />} />
        <Route path="/catalogue/:id" element={<RecordPage />} />
        <Route path="/accession" element={<AccessionPage />} />
        <Route path="/inventory" element={<InventoryPage />} />
        <Route path="/members" element={<MembersPage />} />
        <Route path="/circulation" element={<CirculationPage />} />
        <Route path="/reports" element={<Suspense fallback={<p role="status" className="p-6">Loading reports.</p>}><ReportsPage /></Suspense>} />
        <Route path="/acquisitions" element={<Suspense fallback={<p role="status" className="p-6">Loading acquisitions.</p>}><AcquisitionsPage /></Suspense>} />
        {ALL_NAV_ITEMS.filter((i) => !i.available).map((i) => (
          <Route key={i.path} path={i.path} element={<NotAvailable />} />
        ))}
        {DesignGallery && (
          <Route path="/design" element={<Suspense fallback={null}><DesignGallery /></Suspense>} />
        )}
        <Route path="*" element={<NotAvailable />} />
      </Route>
    </Routes>
  );
}

export default function App() {
  return (
    <ThemeProvider>
      <QueryClientProvider client={queryClient}>
        <Gate>
          <HashRouter>
            <AppRoutes />
          </HashRouter>
        </Gate>
      </QueryClientProvider>
    </ThemeProvider>
  );
}
