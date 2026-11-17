import { useEffect, useRef, useState, type FormEvent } from "react";
import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { PageHeader } from "@/components/PageHeader";
import { Button } from "@/components/ui/Button";
import { SelectField, TextField } from "@/components/ui/Field";
import { Panel } from "@/components/ui/Panel";
import { reportsApi, toApiError } from "@/lib/api";
import type { CatalogueGrowthDay, CirculationDay, FineSummary, MemberSummary, PopularTitle, StockRow } from "@/lib/api/types";
import { localDate } from "@/lib/dates";
import { downloadCsvFile } from "@/lib/files";
import { can, useSession } from "@/lib/session";

const COLUMNS = [
  ["issued", "Issued"], ["returned", "Returned"], ["lost", "Closed as lost"],
  ["renewed", "Renewed"], ["reservations", "Reservations placed"],
] as const;
const STOCK_COLUMNS = [
  ["available", "Available"], ["onLoan", "On loan"], ["held", "Held"],
  ["lost", "Lost"], ["damaged", "Damaged"], ["other", "In repair or transit"], ["total", "Total"],
] as const;
const FINE_COLUMNS = [["assessed", "Assessed"], ["paid", "Paid"], ["waived", "Waived"], ["outstanding", "Outstanding"]] as const;
const MEMBER_COLUMNS = [["current", "Current"], ["expired", "Expired"], ["archived", "Archived"], ["total", "Total"]] as const;

function decimalAmount(minor: string) {
  const digits = minor.padStart(3, "0");
  return `${digits.slice(0, -2)}.${digits.slice(-2)}`;
}

type Report =
  | { kind: "activity"; rows: CirculationDay[] }
  | { kind: "popular"; rows: PopularTitle[] }
  | { kind: "stock"; rows: StockRow[] }
  | { kind: "fines"; rows: FineSummary[] }
  | { kind: "members"; rows: MemberSummary[] }
  | { kind: "growth"; rows: CatalogueGrowthDay[] };

function recentRange() {
  const today = new Date();
  const start = new Date(today);
  start.setDate(start.getDate() - 29);
  return { from: localDate(start), to: localDate(today) };
}

function textCell(value: string) {
  const text = /^[\t\r\n]|^\s*[=+@＝＋－＠-]/.test(value) ? `'${value}` : value;
  return `"${text.replaceAll('"', '""')}"`;
}

function countTotal<Key extends string>(rows: ReadonlyArray<Record<Key, number>>, key: Key) {
  return rows.reduce((sum, row) => sum + row[key], 0);
}

function downloadCsv(report: Report, from: string, to: string) {
  let header: string;
  let lines: string[];
  let filename: string;
  switch (report.kind) {
    case "activity":
      header = ["Date", ...COLUMNS.map(([, label]) => label)].join(",");
      lines = report.rows.map((row) => [row.date, ...COLUMNS.map(([key]) => row[key])].join(","));
      lines.push(["Total", ...COLUMNS.map(([key]) => countTotal(report.rows, key))].join(","));
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
      lines.push(["All locations", "", ...STOCK_COLUMNS.map(([key]) => countTotal(report.rows, key))].join(","));
      filename = `stock-by-location-${localDate()}.csv`;
      break;
    case "fines":
      header = ["Currency", ...FINE_COLUMNS.map(([, label]) => label)].join(",");
      lines = report.rows.map((row) => [textCell(row.currency), ...FINE_COLUMNS.map(([key]) => decimalAmount(row[key]))].join(","));
      filename = `fine-summary-${localDate()}.csv`;
      break;
    case "growth":
      header = "Date,Titles added,Copies added";
      lines = report.rows.map((row) => [row.date, row.titlesAdded, row.copiesAdded].join(","));
      lines.push(["Total", countTotal(report.rows, "titlesAdded"), countTotal(report.rows, "copiesAdded")].join(","));
      filename = `catalogue-growth-${from}-to-${to}.csv`;
      break;
    case "members":
      header = ["Category", "Category status", ...MEMBER_COLUMNS.map(([, label]) => label)].join(",");
      lines = report.rows.map((row) => [
        textCell(row.category), row.archivedCategory ? "Archived" : "Active",
        ...MEMBER_COLUMNS.map(([key]) => row[key]),
      ].join(","));
      lines.push(["All categories", "", ...MEMBER_COLUMNS.map(([key]) => countTotal(report.rows, key))].join(","));
      filename = `members-by-category-${localDate()}.csv`;
      break;
  }
  downloadCsvFile([header, ...lines].join("\r\n") + "\r\n", filename);
}

export default function ReportsPage() {
  const { session } = useSession();
  const allowed = can(session, "reports.read");
  const [range, setRange] = useState(recentRange);
  const [kind, setKind] = useState<Report["kind"]>("activity");
  const usesDateRange = kind === "activity" || kind === "popular" || kind === "growth";
  const reportTables = useRef<HTMLDivElement>(null);
  const printCleanup = useRef<(() => void) | null>(null);
  useEffect(() => () => printCleanup.current?.(), []);
  const result = useQuery({
    queryKey: ["reports", kind, usesDateRange ? range : null],
    queryFn: async (): Promise<Report> => {
      switch (kind) {
        case "activity": return { kind, rows: await reportsApi.circulationActivity(range.from, range.to) };
        case "popular": return { kind, rows: await reportsApi.popularTitles(range.from, range.to) };
        case "stock": return { kind, rows: await reportsApi.stockByLocation() };
        case "fines": return { kind, rows: await reportsApi.fineSummary() };
        case "growth": return { kind, rows: await reportsApi.catalogueGrowth(range.from, range.to) };
        case "members": return { kind, rows: await reportsApi.memberSummary() };
      }
    },
    enabled: allowed,
    retry: false,
  });
  const activity = result.data?.kind === "activity" ? result.data.rows : null;
  const stock = result.data?.kind === "stock" ? result.data.rows : null;
  const members = result.data?.kind === "members" ? result.data.rows : null;
  const canOutput = result.isSuccess && !result.isFetching && result.data.rows.length > 0;

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
    if (!usesDateRange) {
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
          <option value="fines">Fine summary</option>
          <option value="growth">Catalogue growth</option>
          <option value="members">Members by category</option>
        </SelectField>
        {usesDateRange && <>
          <TextField label="Start date" name="from" type="date" defaultValue={range.from} required />
          <TextField label="End date" name="to" type="date" defaultValue={range.to} required />
        </>}
        <Button type="submit" disabled={result.isFetching}>Run report</Button>
        <Button className="ml-auto" disabled={!canOutput}
          onClick={() => { if (result.data) downloadCsv(result.data, range.from, range.to); }}>Export CSV</Button>
        <Button disabled={!canOutput} onClick={printReport}>Print report</Button>
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
                      {countTotal(activity, key)}
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
      {members && (members.length === 0 ? <Panel className="p-6 text-muted">No members recorded.</Panel> : (
        <Panel>
          <table className="w-full border-collapse text-left text-sm">
            <caption className="border-b border-line px-4 py-3 text-left font-medium">Members by category</caption>
            <thead className="border-b border-line text-xs text-muted"><tr>
              <th scope="col" className="px-4 py-2">Category</th>
              {MEMBER_COLUMNS.map(([key, label]) => <th key={key} scope="col" className="px-4 py-2 text-right">{label}</th>)}
            </tr></thead>
            <tbody>{members.map((row) => <tr key={row.categoryId} className="border-b border-line">
              <th scope="row" className="px-4 py-2 font-normal">
                {row.category}
                {row.archivedCategory && <span className="ml-2 text-xs text-muted">Archived category</span>}
              </th>
              {MEMBER_COLUMNS.map(([key]) => <td key={key} className="px-4 py-2 text-right font-mono">{row[key]}</td>)}
            </tr>)}</tbody>
            <tfoot className="bg-band font-semibold"><tr>
              <th scope="row" className="px-4 py-2">All categories</th>
              {MEMBER_COLUMNS.map(([key]) => (
                <td key={key} className="px-4 py-2 text-right font-mono">{countTotal(members, key)}</td>
              ))}
            </tr></tfoot>
          </table>
        </Panel>
      ))}
      {result.data?.kind === "growth" && (
        <Panel>
          <table className="w-full border-collapse text-left text-sm">
            <caption className="border-b border-line px-4 py-3 text-left font-medium">
              Titles and copies added from {range.from} to {range.to}
            </caption>
            <thead className="border-b border-line text-xs text-muted"><tr>
              <th scope="col" className="px-4 py-2">Date</th>
              <th scope="col" className="px-4 py-2 text-right">Titles added</th>
              <th scope="col" className="px-4 py-2 text-right">Copies added</th>
            </tr></thead>
            <tbody>{result.data.rows.map((row) => <tr key={row.date} className="border-b border-line">
              <th scope="row" className="px-4 py-2 font-normal">{row.date}</th>
              <td className="px-4 py-2 text-right font-mono">{row.titlesAdded}</td>
              <td className="px-4 py-2 text-right font-mono">{row.copiesAdded}</td>
            </tr>)}</tbody>
            <tfoot className="bg-band font-semibold"><tr>
              <th scope="row" className="px-4 py-2">Total</th>
              <td className="px-4 py-2 text-right font-mono">{countTotal(result.data.rows, "titlesAdded")}</td>
              <td className="px-4 py-2 text-right font-mono">{countTotal(result.data.rows, "copiesAdded")}</td>
            </tr></tfoot>
          </table>
        </Panel>
      )}
      {result.data?.kind === "fines" && (result.data.rows.length === 0 ? <Panel className="p-6 text-muted">No fines recorded.</Panel> : (
        <Panel>
          <table className="w-full border-collapse text-left text-sm">
            <caption className="border-b border-line px-4 py-3 text-left font-medium">All-time fine summary by currency</caption>
            <thead className="border-b border-line text-xs text-muted"><tr>
              <th scope="col" className="px-4 py-2">Currency</th>
              {FINE_COLUMNS.map(([key, label]) => <th key={key} scope="col" className="px-4 py-2 text-right">{label}</th>)}
            </tr></thead>
            <tbody>{result.data.rows.map((row) => <tr key={row.currency} className="border-b border-line last:border-b-0">
              <th scope="row" className="px-4 py-2 font-normal">{row.currency}</th>
              {FINE_COLUMNS.map(([key]) => <td key={key} className="px-4 py-2 text-right font-mono">{decimalAmount(row[key])}</td>)}
            </tr>)}</tbody>
          </table>
        </Panel>
      ))}
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
                    <td key={key} className="px-3 py-2 text-right font-mono">{countTotal(stock, key)}</td>
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
