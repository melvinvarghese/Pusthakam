import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { useVisibleNavGroups } from "@/app/nav";
import { PageHeader } from "@/components/PageHeader";
import { Panel, PanelHeader } from "@/components/ui/Panel";
import { appApi, ApiError, isDesktop, reportsApi, toApiError } from "@/lib/api";
import { localDate } from "@/lib/dates";
import { previewMode } from "@/lib/preview";
import { can, useSession } from "@/lib/session";

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

function presetRange(days: number) {
  const today = new Date();
  const start = new Date(today);
  start.setDate(start.getDate() - (days - 1));
  return { from: localDate(start), to: localDate(today) };
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

  const hasReports = can(session, "reports.read");
  const [preset, setPreset] = useState<"7" | "30" | "90" | "custom">("30");
  const [range, setRange] = useState(() => presetRange(30));

  function handlePresetChange(opt: "7" | "30" | "90" | "custom") {
    setPreset(opt);
    if (opt !== "custom") {
      setRange(presetRange(Number(opt)));
    }
  }

  const activityQuery = useQuery({
    queryKey: ["dashboard-activity", range.from, range.to],
    queryFn: () => reportsApi.circulationActivity(range.from, range.to),
    enabled: hasReports && (isDesktop() || !!previewMode()),
  });

  const popularQuery = useQuery({
    queryKey: ["dashboard-popular", range.from, range.to],
    queryFn: () => reportsApi.popularTitles(range.from, range.to),
    enabled: hasReports && (isDesktop() || !!previewMode()),
  });

  const growthQuery = useQuery({
    queryKey: ["dashboard-growth", range.from, range.to],
    queryFn: () => reportsApi.catalogueGrowth(range.from, range.to),
    enabled: hasReports && (isDesktop() || !!previewMode()),
  });

  const issuesInPeriod = activityQuery.data?.reduce((sum, d) => sum + d.issued, 0) ?? 0;
  const returnsInPeriod = activityQuery.data?.reduce((sum, d) => sum + d.returned, 0) ?? 0;
  const renewalsInPeriod = activityQuery.data?.reduce((sum, d) => sum + d.renewed, 0) ?? 0;
  const titlesAdded = growthQuery.data?.reduce((sum, d) => sum + d.titlesAdded, 0) ?? 0;
  const copiesAdded = growthQuery.data?.reduce((sum, d) => sum + d.copiesAdded, 0) ?? 0;
  const topTitles = (popularQuery.data ?? []).slice(0, 5);

  return (
    <div className="mx-auto max-w-[1100px] space-y-5 p-6">
      <PageHeader title="Dashboard" description="A Better Way to Manage Libraries." />

      {overview.isFetching && !overview.data && <p role="status" className="text-muted">Loading library overview.</p>}
      {overview.isError && <p role="alert" className="text-danger">{toApiError(overview.error).message}</p>}

      {/* Actionable circulation alerts */}
      {circulation && (circulation.overdue > 0 || circulation.readyHolds > 0 || circulation.dueToday > 0) && (
        <section aria-label="Actionable alerts" className="space-y-3">
          {circulation.overdue > 0 && (
            <div
              role="alert"
              className="flex flex-wrap items-center justify-between gap-3 rounded-lg border border-danger/40 bg-danger/10 px-4 py-3 text-sm"
            >
              <div className="flex items-center gap-2">
                <span className="font-semibold text-danger">Action required:</span>
                <span>
                  {circulation.overdue.toLocaleString()} {circulation.overdue === 1 ? "loan is" : "loans are"} overdue.
                </span>
              </div>
              <Link
                to="/circulation"
                className="font-medium text-danger underline hover:opacity-80"
              >
                Review overdue loans →
              </Link>
            </div>
          )}

          {circulation.readyHolds > 0 && (
            <div
              role="status"
              className="flex flex-wrap items-center justify-between gap-3 rounded-lg border border-accent/40 bg-accent/10 px-4 py-3 text-sm"
            >
              <div className="flex items-center gap-2">
                <span className="font-semibold text-accent">Pickup waiting:</span>
                <span>
                  {circulation.readyHolds.toLocaleString()} {circulation.readyHolds === 1 ? "hold is" : "holds are"} ready for pickup.
                </span>
              </div>
              <Link
                to="/circulation"
                className="font-medium text-accent underline hover:opacity-80"
              >
                View reservations queue →
              </Link>
            </div>
          )}

          {circulation.dueToday > 0 && (
            <div
              role="status"
              className="flex flex-wrap items-center justify-between gap-3 rounded-lg border border-line bg-card/60 px-4 py-3 text-sm"
            >
              <div className="flex items-center gap-2">
                <span className="font-semibold text-muted">Due today:</span>
                <span>
                  {circulation.dueToday.toLocaleString()} {circulation.dueToday === 1 ? "loan is" : "loans are"} due today.
                </span>
              </div>
              <Link
                to="/circulation"
                className="font-medium underline hover:opacity-80"
              >
                Open circulation desk →
              </Link>
            </div>
          )}
        </section>
      )}

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

      {/* Activity & growth trends with date filter */}
      {hasReports && (
        <Panel aria-labelledby="trends-h">
          <PanelHeader
            id="trends-h"
            title="Activity & growth trends"
            aside={
              <div className="flex flex-wrap items-center gap-2">
                <span className="text-xs text-muted">Range:</span>
                <div className="inline-flex rounded-md border border-line p-0.5 text-xs">
                  {(["7", "30", "90", "custom"] as const).map((opt) => (
                    <button
                      key={opt}
                      type="button"
                      onClick={() => handlePresetChange(opt)}
                      className={`rounded px-2.5 py-1 font-medium transition-colors ${
                        preset === opt ? "bg-accent text-accent-contrast" : "hover:bg-hover"
                      }`}
                    >
                      {opt === "custom" ? "Custom" : `${opt}d`}
                    </button>
                  ))}
                </div>
                {preset === "custom" && (
                  <div className="flex items-center gap-2 text-xs">
                    <input
                      type="date"
                      value={range.from}
                      onChange={(e) => setRange((r) => ({ ...r, from: e.target.value }))}
                      className="rounded border border-line bg-background px-2 py-1"
                      aria-label="Start date"
                    />
                    <span>to</span>
                    <input
                      type="date"
                      value={range.to}
                      onChange={(e) => setRange((r) => ({ ...r, to: e.target.value }))}
                      className="rounded border border-line bg-background px-2 py-1"
                      aria-label="End date"
                    />
                  </div>
                )}
              </div>
            }
          />

          <div className="p-4 space-y-6">
            <div>
              <h3 className="mb-2 text-xs font-semibold text-muted">
                Summary for {range.from} to {range.to}
              </h3>
              <dl className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-3">
                <div className="rounded border border-line bg-card/40 p-3">
                  <dt className="text-xs text-muted">Issues</dt>
                  <dd className="mt-1 font-mono text-xl font-semibold tabular-nums">{issuesInPeriod.toLocaleString()}</dd>
                </div>
                <div className="rounded border border-line bg-card/40 p-3">
                  <dt className="text-xs text-muted">Returns</dt>
                  <dd className="mt-1 font-mono text-xl font-semibold tabular-nums">{returnsInPeriod.toLocaleString()}</dd>
                </div>
                <div className="rounded border border-line bg-card/40 p-3">
                  <dt className="text-xs text-muted">Renewals</dt>
                  <dd className="mt-1 font-mono text-xl font-semibold tabular-nums">{renewalsInPeriod.toLocaleString()}</dd>
                </div>
                <div className="rounded border border-line bg-card/40 p-3">
                  <dt className="text-xs text-muted">New titles</dt>
                  <dd className="mt-1 font-mono text-xl font-semibold tabular-nums">{titlesAdded.toLocaleString()}</dd>
                </div>
                <div className="rounded border border-line bg-card/40 p-3">
                  <dt className="text-xs text-muted">New copies</dt>
                  <dd className="mt-1 font-mono text-xl font-semibold tabular-nums">{copiesAdded.toLocaleString()}</dd>
                </div>
              </dl>
            </div>

            <div>
              <div className="flex items-center justify-between mb-2">
                <h3 className="text-xs font-semibold text-muted">Most borrowed titles in this period</h3>
                <Link to="/reports" className="text-xs text-accent hover:underline">
                  View full reports →
                </Link>
              </div>
              {topTitles.length > 0 ? (
                <div className="divide-y divide-line rounded border border-line">
                  {topTitles.map((t, idx) => (
                    <div key={idx} className="flex items-center justify-between p-3 text-sm">
                      <div className="min-w-0 pr-4">
                        <div className="font-medium truncate">{t.title}</div>
                      </div>
                      <div className="shrink-0 font-mono text-sm font-semibold text-muted">
                        {t.issues} {t.issues === 1 ? "borrow" : "borrows"}
                      </div>
                    </div>
                  ))}
                </div>
              ) : (
                <p className="text-xs text-muted py-2">No circulation recorded for this period.</p>
              )}
            </div>
          </div>
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
