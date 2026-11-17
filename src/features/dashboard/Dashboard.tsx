import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { useVisibleNavGroups } from "@/app/nav";
import { PageHeader } from "@/components/PageHeader";
import { Panel, PanelHeader } from "@/components/ui/Panel";
import { appApi, ApiError, isDesktop, toApiError } from "@/lib/api";
import { previewMode } from "@/lib/preview";
import { useSession } from "@/lib/session";

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div className="grid grid-cols-[9rem_1fr] gap-4 border-b border-line py-2 last:border-0">
      <dt className="text-muted">{label}</dt>
      <dd className="min-w-0 break-all text-right font-medium">{value}</dd>
    </div>
  );
}

function Count({ label, value, attention = false }: { label: string; value: number; attention?: boolean }) {
  return (
    <div className="min-w-0 border-b border-line p-4">
      <dt className="text-xs text-muted">{label}</dt>
      <dd className={`mt-1 font-mono text-2xl font-semibold tabular-nums ${attention && value > 0 ? "text-danger" : ""}`}>
        {value.toLocaleString()}
      </dd>
    </div>
  );
}

export function Dashboard() {
  const groups = useVisibleNavGroups();
  const { session } = useSession();
  const info = useQuery({ queryKey: ["app-info"], queryFn: appApi.info, retry: false });
  const overview = useQuery({
    queryKey: ["dashboard", session.userId],
    queryFn: appApi.overview,
    enabled: isDesktop() || !!previewMode(),
    retry: false,
    refetchInterval: 60_000,
  });
  const { catalogue, activeMembers, circulation } = overview.data ?? {};

  return (
    <div className="mx-auto max-w-[1100px] space-y-5 p-6">
      <PageHeader title="Dashboard" description="A Better Way to Manage Libraries." />

      {overview.isFetching && !overview.data && <p role="status" className="text-muted">Loading library overview.</p>}
      {overview.isError && <p role="alert" className="text-danger">{toApiError(overview.error).message}</p>}
      {overview.data && (catalogue || activeMembers != null || circulation) && (
        <Panel aria-labelledby="overview-h">
          <PanelHeader id="overview-h" title="Library at a glance" />
          <dl className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5">
            {catalogue && <>
              <Count label="Catalogue titles" value={catalogue.titles} />
              <Count label="Copies" value={catalogue.copies} />
              <Count label="Available copies" value={catalogue.available} />
            </>}
            {activeMembers != null && <Count label="Active members" value={activeMembers} />}
            {circulation && <>
              <Count label="On loan" value={circulation.onLoan} />
              <Count label="Due today" value={circulation.dueToday} />
              <Count label="Overdue" value={circulation.overdue} attention />
              <Count label="Holds ready" value={circulation.readyHolds} />
              <Count label="Outstanding fines" value={circulation.outstandingFines} attention />
            </>}
          </dl>
        </Panel>
      )}

      <div className="grid gap-5 lg:grid-cols-[minmax(0,1fr)_320px]">
        <Panel aria-labelledby="modules-h">
          <PanelHeader id="modules-h" title="Library modules" />
          <div className="grid gap-x-8 gap-y-5 p-4 sm:grid-cols-2">
            {groups.map((group) => (
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
