import { useQuery } from "@tanstack/react-query";
import { appApi, ApiError } from "@/lib/api";

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex gap-4 border-b border-border py-2 last:border-0">
      <dt className="w-40 shrink-0 text-muted">{label}</dt>
      <dd className="min-w-0 break-all font-mono text-[13px]">{value}</dd>
    </div>
  );
}

/**
 * Phase 1 dashboard: shows real system status only. Library KPIs (records, copies, loans, etc.)
 * arrive with their data modules; no placeholder numbers are displayed.
 */
export function Dashboard() {
  const info = useQuery({ queryKey: ["app-info"], queryFn: appApi.info, retry: false });

  return (
    <div className="mx-auto max-w-3xl space-y-6 p-6">
      <header>
        <h1 className="text-xl font-semibold">Dashboard</h1>
        <p className="text-muted">Library statistics will appear here as catalogue and circulation modules are added.</p>
      </header>

      <section aria-labelledby="status-h" className="rounded-md border border-border bg-surface p-4">
        <h2 id="status-h" className="mb-2 text-sm font-semibold">System status</h2>
        {info.isPending && <p className="text-muted" role="status">Checking local database…</p>}
        {info.isError && (
          <p role="alert" className="text-danger">
            {info.error instanceof ApiError ? info.error.message : "Could not read system status."}
          </p>
        )}
        {info.data && (
          <dl>
            <Row label="Version" value={info.data.version} />
            <Row label="Database schema" value={`v${info.data.schemaVersion}`} />
            <Row label="Data folder" value={info.data.dataDir} />
            <Row label="Database file" value={info.data.databasePath} />
          </dl>
        )}
      </section>
    </div>
  );
}
