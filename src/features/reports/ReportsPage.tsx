import { useEffect, useRef, useState, type FormEvent } from "react";
import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { PageHeader } from "@/components/PageHeader";
import { Button } from "@/components/ui/Button";
import { SelectField, TextField } from "@/components/ui/Field";
import { Panel } from "@/components/ui/Panel";
import { reportsApi, toApiError } from "@/lib/api";
import type { AccessionRow, CatalogueGrowthDay, CirculationDay, ExpenditureRow, FineSummary, LossDamageRow, MemberSummary, OverdueRow, PopularTitle, StockRow } from "@/lib/api/types";
import { localDate } from "@/lib/dates";
import { downloadBase64File, downloadCsvFile } from "@/lib/files";
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
  | { kind: "growth"; rows: CatalogueGrowthDay[] }
  | { kind: "overdues"; rows: OverdueRow[] }
  | { kind: "accession"; rows: AccessionRow[] }
  | { kind: "expenditure"; rows: ExpenditureRow[] }
  | { kind: "loss"; rows: LossDamageRow[] };

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

interface ReportData {
  title: string;
  subtitle: string;
  headers: string[];
  rows: string[][];
  alignments: string[];
  baseFilename: string;
}

function formatReportData(report: Report, from: string, to: string): ReportData {
  switch (report.kind) {
    case "activity": {
      const headers = ["Date", ...COLUMNS.map(([, label]) => label)];
      const rows = report.rows.map((row) => [row.date, ...COLUMNS.map(([key]) => String(row[key]))]);
      rows.push(["Total", ...COLUMNS.map(([key]) => String(countTotal(report.rows, key)))]);
      return {
        title: "Circulation Activity",
        subtitle: `Date range: ${from} to ${to}`,
        headers,
        rows,
        alignments: ["left", "right", "right", "right", "right", "right"],
        baseFilename: `circulation-${from}-to-${to}`,
      };
    }
    case "popular": {
      const headers = ["Title", "Issues", "Catalogue status"];
      const rows = report.rows.map((row) => [row.title, String(row.issues), row.archived ? "Archived" : "Active"]);
      return {
        title: "Top 25 Borrowed Titles",
        subtitle: `Date range: ${from} to ${to}`,
        headers,
        rows,
        alignments: ["left", "right", "left"],
        baseFilename: `most-borrowed-${from}-to-${to}`,
      };
    }
    case "stock": {
      const headers = ["Location", "Location status", ...STOCK_COLUMNS.map(([, label]) => label)];
      const rows = report.rows.map((row) => [
        row.location ?? "Unassigned",
        row.archivedLocation ? "Archived" : row.location ? "Active" : "",
        ...STOCK_COLUMNS.map(([key]) => String(row[key])),
      ]);
      rows.push(["All locations", "", ...STOCK_COLUMNS.map(([key]) => String(countTotal(report.rows, key)))]);
      return {
        title: "Stock by Location",
        subtitle: `Active catalogue, excluding withdrawn copies · as of ${localDate()}`,
        headers,
        rows,
        alignments: ["left", "left", "right", "right", "right", "right", "right", "right", "right"],
        baseFilename: `stock-by-location-${localDate()}`,
      };
    }
    case "fines": {
      const headers = ["Currency", ...FINE_COLUMNS.map(([, label]) => label)];
      const rows = report.rows.map((row) => [row.currency, ...FINE_COLUMNS.map(([key]) => decimalAmount(row[key]))]);
      return {
        title: "Fine Summary",
        subtitle: `All-time fine summary by currency · as of ${localDate()}`,
        headers,
        rows,
        alignments: ["left", "right", "right", "right", "right"],
        baseFilename: `fine-summary-${localDate()}`,
      };
    }
    case "growth": {
      const headers = ["Date", "Titles added", "Copies added"];
      const rows = report.rows.map((row) => [row.date, String(row.titlesAdded), String(row.copiesAdded)]);
      rows.push(["Total", String(countTotal(report.rows, "titlesAdded")), String(countTotal(report.rows, "copiesAdded"))]);
      return {
        title: "Catalogue Growth",
        subtitle: `Titles and copies added from ${from} to ${to}`,
        headers,
        rows,
        alignments: ["left", "right", "right"],
        baseFilename: `catalogue-growth-${from}-to-${to}`,
      };
    }
    case "members": {
      const headers = ["Category", "Category status", ...MEMBER_COLUMNS.map(([, label]) => label)];
      const rows = report.rows.map((row) => [
        row.category,
        row.archivedCategory ? "Archived" : "Active",
        ...MEMBER_COLUMNS.map(([key]) => String(row[key])),
      ]);
      rows.push(["All categories", "", ...MEMBER_COLUMNS.map(([key]) => String(countTotal(report.rows, key)))]);
      return {
        title: "Members by Category",
        subtitle: `Member counts as of ${localDate()}`,
        headers,
        rows,
        alignments: ["left", "left", "right", "right", "right", "right"],
        baseFilename: `members-by-category-${localDate()}`,
      };
    }
    case "overdues": {
      const headers = ["Due date", "Days overdue", "Member", "Member barcode", "Accession no", "Barcode", "Title"];
      const rows = report.rows.map((row) => [
        row.dueDate,
        String(row.daysOverdue),
        row.memberName,
        row.memberBarcode,
        row.accessionNo,
        row.barcode,
        row.title,
      ]);
      return {
        title: "Overdue Loans",
        subtitle: `Loans due from ${from} to ${to}`,
        headers,
        rows,
        alignments: ["left", "right", "left", "left", "left", "left", "left"],
        baseFilename: `overdue-loans-${localDate()}`,
      };
    }
    case "accession": {
      const headers = ["Accession no", "Barcode", "Title", "Acquisition date", "Source", "Price", "Location", "Status"];
      const rows = report.rows.map((row) => [
        row.accessionNo,
        row.barcode,
        row.title,
        row.acquisitionDate ?? "",
        row.acquisitionSource,
        row.priceMinor != null ? decimalAmount(String(row.priceMinor)) : "",
        row.location ?? "Unassigned",
        row.status,
      ]);
      return {
        title: "Accession List",
        subtitle: `Copies added from ${from} to ${to}`,
        headers,
        rows,
        alignments: ["left", "left", "left", "left", "left", "right", "left", "left"],
        baseFilename: `accession-list-${from}-to-${to}`,
      };
    }
    case "expenditure": {
      const headers = ["Acquisition source", "Copies", "Total cost"];
      const rows = report.rows.map((row) => [
        row.acquisitionSource,
        String(row.copies),
        decimalAmount(String(row.totalMinor)),
      ]);
      rows.push([
        "All sources",
        String(countTotal(report.rows, "copies")),
        decimalAmount(String(report.rows.reduce((s, r) => s + r.totalMinor, 0))),
      ]);
      return {
        title: "Expenditure by Acquisition Source",
        subtitle: `Acquisitions from ${from} to ${to}`,
        headers,
        rows,
        alignments: ["left", "right", "right"],
        baseFilename: `expenditure-${from}-to-${to}`,
      };
    }
    case "loss": {
      const headers = ["Returned date", "Condition", "Member", "Accession no", "Title"];
      const rows = report.rows.map((row) => [
        row.returnedDate,
        row.returnCondition,
        row.memberName,
        row.accessionNo,
        row.title,
      ]);
      return {
        title: "Loss and Damage",
        subtitle: `Recorded returns from ${from} to ${to}`,
        headers,
        rows,
        alignments: ["left", "left", "left", "left", "left"],
        baseFilename: `loss-damage-${from}-to-${to}`,
      };
    }
  }
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
    case "overdues":
      header = "Due date,Days overdue,Member,Member barcode,Accession no,Barcode,Title";
      lines = report.rows.map((row) => [row.dueDate, row.daysOverdue, textCell(row.memberName), textCell(row.memberBarcode), textCell(row.accessionNo), textCell(row.barcode), textCell(row.title)].join(","));
      filename = `overdue-loans-${localDate()}.csv`;
      break;
    case "accession":
      header = "Accession no,Barcode,Title,Acquisition date,Source,Price,Location,Status";
      lines = report.rows.map((row) => [
        textCell(row.accessionNo), textCell(row.barcode), textCell(row.title),
        row.acquisitionDate ?? "", textCell(row.acquisitionSource),
        row.priceMinor != null ? decimalAmount(String(row.priceMinor)) : "",
        textCell(row.location ?? "Unassigned"), textCell(row.status),
      ].join(","));
      filename = `accession-list-${from}-to-${to}.csv`;
      break;
    case "expenditure":
      header = "Acquisition source,Copies,Total cost";
      lines = report.rows.map((row) => [textCell(row.acquisitionSource), row.copies, decimalAmount(String(row.totalMinor))].join(","));
      lines.push(["All sources", countTotal(report.rows, "copies"), decimalAmount(String(report.rows.reduce((s, r) => s + r.totalMinor, 0)))].join(","));
      filename = `expenditure-${from}-to-${to}.csv`;
      break;
    case "loss":
      header = "Returned date,Condition,Member,Accession no,Title";
      lines = report.rows.map((row) => [row.returnedDate, textCell(row.returnCondition), textCell(row.memberName), textCell(row.accessionNo), textCell(row.title)].join(","));
      filename = `loss-damage-${from}-to-${to}.csv`;
      break;
  }
  downloadCsvFile([header, ...lines].join("\r\n") + "\r\n", filename);
}

export default function ReportsPage() {
  const { session } = useSession();
  const allowed = can(session, "reports.read");
  const [range, setRange] = useState(recentRange);
  const [kind, setKind] = useState<Report["kind"]>("activity");
  const usesDateRange = kind === "activity" || kind === "popular" || kind === "growth" || kind === "overdues" || kind === "accession" || kind === "expenditure" || kind === "loss";
  const [simpleDesign, setSimpleDesign] = useState(false);
  const [exporting, setExporting] = useState<string | null>(null);
  const [saveMessage, setSaveMessage] = useState<string | null>(null);
  const [exportError, setExportError] = useState<string | null>(null);
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
        case "overdues": return { kind, rows: await reportsApi.overdueLoansList(range.from, range.to) };
        case "accession": return { kind, rows: await reportsApi.accessionList(range.from, range.to) };
        case "expenditure": return { kind, rows: await reportsApi.expenditureSummary(range.from, range.to) };
        case "loss": return { kind, rows: await reportsApi.lossDamageList(range.from, range.to) };
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
    copy.className = simpleDesign ? "report-print report-simple" : "report-print report-polished";
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

  async function exportDocument(format: "xlsx" | "pdf") {
    if (!result.data) return;
    setExporting(format);
    setSaveMessage(null);
    setExportError(null);
    try {
      const data = formatReportData(result.data, range.from, range.to);
      const file = await reportsApi.exportReport(
        format,
        `${data.baseFilename}.${format}`,
        data.title,
        data.subtitle,
        data.headers,
        data.rows,
        data.alignments,
        simpleDesign,
      );
      const saved = await reportsApi.saveExport(file.fileName, file.dataBase64);
      if (!saved.native) {
        downloadBase64File(file.dataBase64, file.fileName, file.mimeType);
        setSaveMessage("Report download started.");
      } else setSaveMessage(saved.path ? `Saved to ${saved.path}` : "Save cancelled.");
    } catch (e) {
      setExportError(toApiError(e).message);
    } finally {
      setExporting(null);
    }
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
          <option value="overdues">Overdue loans</option>
          <option value="accession">Accession list</option>
          <option value="expenditure">Expenditure summary</option>
          <option value="loss">Loss and damage</option>
        </SelectField>
        {usesDateRange && <>
          <TextField label="Start date" name="from" type="date" defaultValue={range.from} required />
          <TextField label="End date" name="to" type="date" defaultValue={range.to} required />
        </>}
        <Button type="submit" disabled={result.isFetching}>Run report</Button>
        <Button className="ml-auto" disabled={!canOutput || !!exporting}
          onClick={() => { if (result.data) downloadCsv(result.data, range.from, range.to); }}>Export CSV</Button>
        <Button disabled={!canOutput || !!exporting}
          onClick={() => void exportDocument("xlsx")}>{exporting === "xlsx" ? "Exporting XLSX…" : "Export XLSX"}</Button>
        <Button disabled={!canOutput || !!exporting}
          onClick={() => void exportDocument("pdf")}>{exporting === "pdf" ? "Exporting PDF…" : "Export PDF"}</Button>
        <Button disabled={!canOutput || !!exporting} onClick={printReport}>Print report</Button>
        <label className="flex cursor-pointer items-center gap-2 text-sm select-none">
          <input
            type="checkbox"
            checked={simpleDesign}
            onChange={(e) => setSimpleDesign(e.target.checked)}
            className="size-4 rounded border-line accent-primary"
          />
          Use simple report design
        </label>
      </form>
      {result.isPending && <p role="status" className="text-muted">Loading report.</p>}
      {result.isError && <p role="alert" className="text-danger">{toApiError(result.error).message}</p>}
      {saveMessage && <p role="status">{saveMessage}</p>}
      {exportError && <p role="alert" className="text-danger">{exportError}</p>}
      <div ref={reportTables} className={simpleDesign ? "report-simple space-y-4" : "report-polished space-y-4"}>
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
      {result.data?.kind === "overdues" && (
        result.data.rows.length === 0
          ? <Panel className="p-6 text-muted">No overdue loans in this date range.</Panel>
          : <Panel>
              <div className="overflow-x-auto">
                <table className="w-full border-collapse text-left text-sm">
                  <caption className="border-b border-line px-4 py-3 text-left text-sm font-medium">
                    Overdue loans · due {range.from} to {range.to}
                  </caption>
                  <thead className="border-b border-line text-xs text-muted"><tr>
                    <th scope="col" className="px-4 py-2">Due date</th>
                    <th scope="col" className="px-4 py-2 text-right">Days overdue</th>
                    <th scope="col" className="px-4 py-2">Member</th>
                    <th scope="col" className="px-4 py-2">Accession no</th>
                    <th scope="col" className="px-4 py-2">Title</th>
                  </tr></thead>
                  <tbody>{result.data.rows.map((row) => (
                    <tr key={row.loanId} className="border-b border-line last:border-b-0">
                      <th scope="row" className="whitespace-nowrap px-4 py-2 font-normal">{row.dueDate}</th>
                      <td className="px-4 py-2 text-right font-mono">{row.daysOverdue}</td>
                      <td className="px-4 py-2">{row.memberName}</td>
                      <td className="px-4 py-2 font-mono">{row.accessionNo}</td>
                      <td className="px-4 py-2">{row.title}</td>
                    </tr>
                  ))}</tbody>
                </table>
              </div>
            </Panel>
      )}
      {result.data?.kind === "accession" && (
        result.data.rows.length === 0
          ? <Panel className="p-6 text-muted">No copies added in this date range.</Panel>
          : <Panel>
              <div className="overflow-x-auto">
                <table className="w-full border-collapse text-left text-sm">
                  <caption className="border-b border-line px-4 py-3 text-left text-sm font-medium">
                    Accession list · {range.from} to {range.to}
                  </caption>
                  <thead className="border-b border-line text-xs text-muted"><tr>
                    <th scope="col" className="px-4 py-2">Accession no</th>
                    <th scope="col" className="px-4 py-2">Title</th>
                    <th scope="col" className="px-4 py-2">Acq. date</th>
                    <th scope="col" className="px-4 py-2">Source</th>
                    <th scope="col" className="px-4 py-2 text-right">Price</th>
                    <th scope="col" className="px-4 py-2">Location</th>
                    <th scope="col" className="px-4 py-2">Status</th>
                  </tr></thead>
                  <tbody>{result.data.rows.map((row) => (
                    <tr key={row.itemId} className="border-b border-line last:border-b-0">
                      <th scope="row" className="whitespace-nowrap px-4 py-2 font-mono font-normal">{row.accessionNo}</th>
                      <td className="px-4 py-2">{row.title}</td>
                      <td className="whitespace-nowrap px-4 py-2">{row.acquisitionDate ?? "—"}</td>
                      <td className="px-4 py-2">{row.acquisitionSource}</td>
                      <td className="px-4 py-2 text-right font-mono">
                        {row.priceMinor != null ? decimalAmount(String(row.priceMinor)) : "—"}
                      </td>
                      <td className="px-4 py-2">{row.location ?? "Unassigned"}</td>
                      <td className="px-4 py-2">{row.status}</td>
                    </tr>
                  ))}</tbody>
                </table>
              </div>
            </Panel>
      )}
      {result.data?.kind === "expenditure" && (
        result.data.rows.length === 0
          ? <Panel className="p-6 text-muted">No acquisitions in this date range.</Panel>
          : <Panel>
              <table className="w-full border-collapse text-left text-sm">
                <caption className="border-b border-line px-4 py-3 text-left text-sm font-medium">
                  Expenditure by acquisition source · {range.from} to {range.to}
                </caption>
                <thead className="border-b border-line text-xs text-muted"><tr>
                  <th scope="col" className="px-4 py-2">Acquisition source</th>
                  <th scope="col" className="px-4 py-2 text-right">Copies</th>
                  <th scope="col" className="px-4 py-2 text-right">Total cost</th>
                </tr></thead>
                <tbody>{result.data.rows.map((row) => (
                  <tr key={row.acquisitionSource} className="border-b border-line last:border-b-0">
                    <th scope="row" className="px-4 py-2 font-normal capitalize">{row.acquisitionSource}</th>
                    <td className="px-4 py-2 text-right font-mono">{row.copies}</td>
                    <td className="px-4 py-2 text-right font-mono">{decimalAmount(String(row.totalMinor))}</td>
                  </tr>
                ))}</tbody>
                <tfoot className="border-t border-line bg-band font-semibold"><tr>
                  <th scope="row" className="px-4 py-2">All sources</th>
                  <td className="px-4 py-2 text-right font-mono">{countTotal(result.data.rows, "copies")}</td>
                  <td className="px-4 py-2 text-right font-mono">{decimalAmount(String(result.data.rows.reduce((s, r) => s + r.totalMinor, 0)))}</td>
                </tr></tfoot>
              </table>
            </Panel>
      )}
      {result.data?.kind === "loss" && (
        result.data.rows.length === 0
          ? <Panel className="p-6 text-muted">No loss or damage recorded in this date range.</Panel>
          : <Panel>
              <table className="w-full border-collapse text-left text-sm">
                <caption className="border-b border-line px-4 py-3 text-left text-sm font-medium">
                  Loss and damage · {range.from} to {range.to}
                </caption>
                <thead className="border-b border-line text-xs text-muted"><tr>
                  <th scope="col" className="px-4 py-2">Returned date</th>
                  <th scope="col" className="px-4 py-2">Condition</th>
                  <th scope="col" className="px-4 py-2">Member</th>
                  <th scope="col" className="px-4 py-2">Accession no</th>
                  <th scope="col" className="px-4 py-2">Title</th>
                </tr></thead>
                <tbody>{result.data.rows.map((row) => (
                  <tr key={row.loanId} className="border-b border-line last:border-b-0">
                    <th scope="row" className="whitespace-nowrap px-4 py-2 font-normal">{row.returnedDate}</th>
                    <td className="px-4 py-2 capitalize">{row.returnCondition}</td>
                    <td className="px-4 py-2">{row.memberName}</td>
                    <td className="px-4 py-2 font-mono">{row.accessionNo}</td>
                    <td className="px-4 py-2">{row.title}</td>
                  </tr>
                ))}</tbody>
              </table>
            </Panel>
      )}
      </div>
    </div>
  );
}
