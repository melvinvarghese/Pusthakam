import { lazy, Suspense } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { HashRouter, Route, Routes } from "react-router-dom";
import { AppShell } from "@/app/AppShell";
import { ALL_NAV_ITEMS } from "@/app/nav";
import { Gate } from "@/features/boot/Gate";
import { AccessionPage } from "@/features/catalogue/AccessionPage";
import { CataloguePage } from "@/features/catalogue/CataloguePage";
import { RecordPage } from "@/features/catalogue/RecordPage";
import { Dashboard } from "@/features/dashboard/Dashboard";
import { MembersPage } from "@/features/members/MembersPage";
import { NotAvailable } from "@/features/NotAvailable";
import { SettingsPage } from "@/features/settings/SettingsPage";
import { ThemeProvider } from "@/lib/theme";

const queryClient = new QueryClient();

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
        <Route path="/catalogue" element={<CataloguePage />} />
        <Route path="/catalogue/:id" element={<RecordPage />} />
        <Route path="/accession" element={<AccessionPage />} />
        <Route path="/members" element={<MembersPage />} />
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
