import { useEffect, useRef, useState, type FormEvent } from "react";
import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { PageHeader } from "@/components/PageHeader";
import { Button } from "@/components/ui/Button";
import { SelectField, TextField } from "@/components/ui/Field";
import { Panel } from "@/components/ui/Panel";
import { reportsApi, toApiError } from "@/lib/api";
import type { CirculationDay, PopularTitle, StockRow } from "@/lib/api/types";
import { localDate } from "@/lib/dates";
import { can, useSession } from "@/lib/session";

const COLUMNS = [
  ["issued", "Issued"], ["returned", "Returned"], ["lost", "Closed as lost"],
  ["renewed", "Renewed"], ["reservations", "Reservations placed"],
] as const;
const STOCK_COLUMNS = [
  ["available", "Available"], ["onLoan", "On loan"], ["held", "Held"],
  ["lost", "Lost"], ["damaged", "Damaged"], ["other", "In repair or transit"], ["total", "Total"],
] as const;
type Report =
  | { kind: "activity"; rows: CirculationDay[] }
  | { kind: "popular"; rows: PopularTitle[] }
  | { kind: "stock"; rows: StockRow[] };

function recentRange() {
  const today = new Date();
  const start = new Date(today);
  start.setDate(start.getDate() - 29);
  return { from: localDate(start), to: localDate(today) };
}

function textCell(value: string) {
  const text = /^[\t\r\n]|^\s*[=+@-]/.test(value) ? `'${value}` : value;
  return `"${text.replaceAll('"', '""')}"`;
}

function downloadCsv(report: Report, from: string, to: string) {
  let header: string;
  let lines: string[];
  let filename: string;
  switch (report.kind) {
    case "activity":
      header = ["Date", ...COLUMNS.map(([, label]) => label)].join(",");
      lines = report.rows.map((row) => [row.date, ...COLUMNS.map(([key]) => row[key])].join(","));
      filename = `circulation-${from}-to-${to}.csv`;
      break;
    case "popular":
      header = "Title,Issues,Catalogue status";
      lines = report.rows.map((row) => [textCell(row.title), row.issues, row.archived ? "Archived" : "Active"].join(","));
      filename = `most-borrowed-${from}-to-${to}.csv`;
      break;
    case "stock":
      header = ["Location", "Location status", ...STOCK_COLUMNS.map(([, label]) => label)].join(",");
      lines = report.rows.map((row) => [
        textCell(row.location ?? "Unassigned"), row.archivedLocation ? "Archived" : row.location ? "Active" : "",
        ...STOCK_COLUMNS.map(([key]) => row[key]),
      ].join(","));
      filename = `stock-by-location-${localDate()}.csv`;
      break;
  }
  const urls = URL;
  const url = urls.createObjectURL(new Blob([[header, ...lines].join("\r\n") + "\r\n"], { type: "text/csv;charset=utf-8" }));
  const link = document.createElement("a");
  link.href = url;
  link.download = filename;
  document.body.append(link);
  link.click();
  link.remove();
  window.setTimeout(() => urls.revokeObjectURL(url), 1000);
}

export default function ReportsPage() {
  const { session } = useSession();
  const allowed = can(session, "reports.read");
  const [range, setRange] = useState(recentRange);
  const [kind, setKind] = useState<Report["kind"]>("activity");
  const reportTables = useRef<HTMLDivElement>(null);
  const printCleanup = useRef<(() => void) | null>(null);
  useEffect(() => () => printCleanup.current?.(), []);
  const result = useQuery({
    queryKey: ["reports", kind, kind === "stock" ? null : range],
    queryFn: async (): Promise<Report> => {
      switch (kind) {
        case "activity": return { kind, rows: await reportsApi.circulationActivity(range.from, range.to) };
        case "popular": return { kind, rows: await reportsApi.popularTitles(range.from, range.to) };
        case "stock": return { kind, rows: await reportsApi.stockByLocation() };
      }
    },
    enabled: allowed,
    retry: false,
  });
  const activity = result.data?.kind === "activity" ? result.data.rows : null;
  const stock = result.data?.kind === "stock" ? result.data.rows : null;

  if (!allowed) return <p role="alert" className="p-6">You do not have permission to view reports.</p>;
  function printReport() {
    if (!reportTables.current) return;
    printCleanup.current?.();
    const copy = reportTables.current.cloneNode(true) as HTMLDivElement;
    copy.className = "report-print";
    const cleanup = () => {
      copy.remove();
      window.removeEventListener("afterprint", cleanup);
      printCleanup.current = null;
    };
    printCleanup.current = cleanup;
    document.body.append(copy);
    window.addEventListener("afterprint", cleanup, { once: true });
    window.print();
  }

  function applyRange(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (kind === "stock") {
      void result.refetch();
      return;
    }
    const fields = new FormData(event.currentTarget);
    const from = String(fields.get("from") ?? "");
    const to = String(fields.get("to") ?? "");
    if (from === range.from && to === range.to) void result.refetch();
    else setRange({ from, to });
  }

  return (
    <div className="mx-auto max-w-[1100px] space-y-4 p-6">
      <PageHeader title="Reports" />
      <form className="flex flex-wrap items-end gap-3" onSubmit={applyRange}>
        <SelectField label="Report" value={kind} onChange={(event) => setKind(event.target.value as Report["kind"])}>
          <option value="activity">Circulation activity</option>
          <option value="popular">Most borrowed titles</option>
          <option value="stock">Stock by location</option>
        </SelectField>
        {kind !== "stock" && <>
          <TextField label="Start date" name="from" type="date" defaultValue={range.from} required />
          <TextField label="End date" name="to" type="date" defaultValue={range.to} required />
        </>}
        <Button type="submit" disabled={result.isFetching}>Run report</Button>
        <Button className="ml-auto" disabled={!result.data?.rows.length || result.isFetching}
          onClick={() => { if (result.data) downloadCsv(result.data, range.from, range.to); }}>Export CSV</Button>
        <Button disabled={!result.data?.rows.length || result.isFetching} onClick={printReport}>Print report</Button>
      </form>
      {result.isPending && <p role="status" className="text-muted">Loading report.</p>}
      {result.isError && <p role="alert" className="text-danger">{toApiError(result.error).message}</p>}
      <div ref={reportTables}>
      {activity && (
        <Panel>
          <div className="overflow-x-auto">
            <table className="w-full border-collapse text-left text-sm">
              <caption className="border-b border-line px-4 py-3 text-left text-sm font-medium">
                Circulation activity from {range.from} to {range.to}
              </caption>
              <thead className="border-b border-line text-xs text-muted">
                <tr>
                  <th scope="col" className="px-4 py-2">Date</th>
                  {COLUMNS.map(([key, label]) => <th key={key} scope="col" className="px-4 py-2 text-right">{label}</th>)}
                </tr>
              </thead>
              <tbody>
                {activity.map((row) => (
                  <tr key={row.date} className="border-b border-line last:border-b-0">
                    <th scope="row" className="whitespace-nowrap px-4 py-2 font-normal">{row.date}</th>
                    {COLUMNS.map(([key]) => <td key={key} className="px-4 py-2 text-right font-mono">{row[key]}</td>)}
                  </tr>
                ))}
              </tbody>
              <tfoot className="border-t border-line bg-band font-semibold">
                <tr>
                  <th scope="row" className="px-4 py-2">Total</th>
                  {COLUMNS.map(([key]) => (
                    <td key={key} className="px-4 py-2 text-right font-mono">
                      {activity.reduce((sum, row) => sum + row[key], 0)}
                    </td>
                  ))}
                </tr>
              </tfoot>
            </table>
          </div>
        </Panel>
      )}
      {result.data?.kind === "popular" && (
        result.data.rows.length === 0 ? <Panel className="p-6 text-muted">No titles were issued in this date range.</Panel> :
        <Panel>
          <table className="w-full border-collapse text-left text-sm">
            <caption className="border-b border-line px-4 py-3 text-left text-sm font-medium">
              Top 25 borrowed titles from {range.from} to {range.to}
            </caption>
            <thead className="border-b border-line text-xs text-muted">
              <tr>
                <th scope="col" className="px-4 py-2">Title</th>
                <th scope="col" className="px-4 py-2 text-right">Issues</th>
              </tr>
            </thead>
            <tbody>
              {result.data.rows.map((row) => (
                <tr key={row.recordId} className="border-b border-line last:border-b-0">
                  <th scope="row" className="px-4 py-2 font-normal">
                    {can(session, "catalogue.read")
                      ? <Link to={`/catalogue/${row.recordId}`} className="underline">{row.title}</Link>
                      : row.title}
                    {row.archived && <span className="ml-2 text-xs text-muted">Archived</span>}
                  </th>
                  <td className="px-4 py-2 text-right font-mono">{row.issues}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </Panel>
      )}
      {stock && (stock.length === 0 ? <Panel className="p-6 text-muted">No stock to report.</Panel> : (
        <Panel>
          <div className="overflow-x-auto">
            <table className="w-full border-collapse text-left text-sm">
              <caption className="border-b border-line px-4 py-3 text-left text-sm font-medium">
                Stock by location · active catalogue, excluding withdrawn copies
              </caption>
              <thead className="border-b border-line text-xs text-muted">
                <tr>
                  <th scope="col" className="px-3 py-2">Location</th>
                  {STOCK_COLUMNS.map(([key, label]) => <th key={key} scope="col" className="px-3 py-2 text-right">{label}</th>)}
                </tr>
              </thead>
              <tbody>
                {stock.map((row) => (
                  <tr key={row.locationId ?? "unassigned"} className="border-b border-line last:border-b-0">
                    <th scope="row" className="px-3 py-2 font-normal">
                      {row.location ?? "Unassigned"}
                      {row.archivedLocation && <span className="ml-2 text-xs text-muted">Archived</span>}
                    </th>
                    {STOCK_COLUMNS.map(([key]) => <td key={key} className="px-3 py-2 text-right font-mono">{row[key]}</td>)}
                  </tr>
                ))}
              </tbody>
              <tfoot className="border-t border-line bg-band font-semibold">
                <tr>
                  <th scope="row" className="px-3 py-2">All locations</th>
                  {STOCK_COLUMNS.map(([key]) => (
                    <td key={key} className="px-3 py-2 text-right font-mono">{stock.reduce((sum, row) => sum + row[key], 0)}</td>
                  ))}
                </tr>
              </tfoot>
            </table>
          </div>
        </Panel>
      ))}
      </div>
    </div>
  );
}
