import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { NAV_GROUPS } from "@/app/nav";
import { PageHeader } from "@/components/PageHeader";
import { Panel, PanelHeader } from "@/components/ui/Panel";
import { appApi, ApiError } from "@/lib/api";

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div className="grid grid-cols-[9rem_1fr] gap-4 border-b border-line py-2 last:border-0">
      <dt className="text-muted">{label}</dt>
      <dd className="min-w-0 break-all text-right font-medium">{value}</dd>
    </div>
  );
}

/**
 * Phase 1 dashboard: real system status and the real module list only.
 * Library figures (records, copies, loans, members) appear with their data modules; no placeholder numbers.
 */
export function Dashboard() {
  const info = useQuery({ queryKey: ["app-info"], queryFn: appApi.info, retry: false });

  return (
    <div className="mx-auto max-w-[1100px] space-y-5 p-6">
      <PageHeader title="Dashboard" description="A Better Way to Manage Libraries." />

      <div className="grid gap-5 lg:grid-cols-[minmax(0,1fr)_320px]">
        <Panel aria-labelledby="modules-h">
          <PanelHeader id="modules-h" title="Coming to your library" />
          <p className="px-4 pt-3 text-muted">
            Pusthakam is being built in stages. Library setup comes first, then the catalogue and circulation.
            Modules below open a page that explains what they will do.
          </p>
          <div className="grid gap-x-8 gap-y-5 p-4 sm:grid-cols-2">
            {NAV_GROUPS.map((group) => (
              <div key={group.label}>
                <h3 className="mb-1 text-xs font-semibold text-muted">{group.label}</h3>
                <ul>
                  {group.items
                    .filter((i) => i.path !== "/")
                    .map((item) => (
                      <li key={item.path} className="border-t border-line py-1.5">
                        <Link to={item.path} className="hover:underline">{item.label}</Link>
                      </li>
                    ))}
                </ul>
              </div>
            ))}
          </div>
        </Panel>

        <div className="space-y-5">
          <Panel aria-labelledby="status-h">
            <PanelHeader id="status-h" title="System status" />
            <div className="px-4 py-2">
              {info.isPending && <p className="py-2 text-muted" role="status">Checking local database…</p>}
              {info.isError && (
                <p role="alert" className="py-2 text-danger">
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
            </div>
          </Panel>
        </div>
      </div>
    </div>
  );
}
