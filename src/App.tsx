import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { HashRouter, Route, Routes } from "react-router-dom";
import { AppShell } from "@/app/AppShell";
import { ALL_NAV_ITEMS } from "@/app/nav";
import { Dashboard } from "@/features/dashboard/Dashboard";
import { NotAvailable } from "@/features/NotAvailable";
import { ThemeProvider } from "@/lib/theme";

const queryClient = new QueryClient();

export function AppRoutes() {
  return (
    <Routes>
      <Route element={<AppShell />}>
        <Route index element={<Dashboard />} />
        {ALL_NAV_ITEMS.filter((i) => !i.available).map((i) => (
          <Route key={i.path} path={i.path} element={<NotAvailable />} />
        ))}
        <Route path="*" element={<NotAvailable />} />
      </Route>
    </Routes>
  );
}

export default function App() {
  return (
    <ThemeProvider>
      <QueryClientProvider client={queryClient}>
        <HashRouter>
          <AppRoutes />
        </HashRouter>
      </QueryClientProvider>
    </ThemeProvider>
  );
}
