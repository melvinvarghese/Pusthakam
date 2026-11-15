/**
 * Development-only visual reference (route /design, never registered in production builds).
 * All records below are SYNTHETIC sample data for checking components; they are not library data.
 */
import { CallNumberLabel } from "@/components/CallNumberLabel";
import { PageHeader } from "@/components/PageHeader";
import { Button } from "@/components/ui/Button";
import { Chip } from "@/components/ui/Chip";
import { Panel, PanelHeader } from "@/components/ui/Panel";

const CLASSES = [
  ["000", "General works"], ["100", "Philosophy, psychology"], ["200", "Religion"], ["300", "Social sciences"],
  ["400", "Language"], ["500", "Science"], ["600", "Technology"], ["700", "Arts, recreation"],
  ["800", "Literature"], ["900", "History, geography"],
] as const;

/** Fixed "today" so the sample table is stable; relative notes are derived from the dates, never typed. */
const TODAY = Date.UTC(2026, 10, 15);
const DAY = 86_400_000;
const FMT = new Intl.DateTimeFormat("en-GB", { day: "2-digit", month: "short", year: "numeric", timeZone: "UTC" });

function loan(title: string, author: string, call: string, acc: string, due: [number, number, number]) {
  const dueMs = Date.UTC(...due);
  const days = Math.round((dueMs - TODAY) / DAY);
  const od = days < 0;
  const n = Math.abs(days);
  return {
    title, author, call, acc, od,
    due: FMT.format(dueMs),
    status: od ? "Overdue" : "On loan",
    note: od ? `${n} day${n === 1 ? "" : "s"} overdue` : `${n} day${n === 1 ? "" : "s"} left`,
  };
}

const LOANS = [
  loan("Balyakalasakhi", "Vaikom Muhammad Basheer", "894.8123 BAS", "004217", [2026, 10, 16]),
  loan("The God of Small Things", "Arundhati Roy", "823 ROY", "003981", [2026, 10, 5]),
  loan("Wings of Fire", "A. P. J. Abdul Kalam", "629.1092 KAL", "005530", [2026, 10, 23]),
];

export function DesignGallery() {
  return (
    <div className="mx-auto max-w-[1100px] space-y-5 p-6">
      <PageHeader
        title="Design gallery"
        description="Development only. Synthetic sample data."
        actions={<><Button>Secondary</Button><Button variant="primary">Primary</Button></>}
      />

      <Panel aria-labelledby="labels-h">
        <PanelHeader id="labels-h" title="Spine labels by Dewey class" />
        <ul className="grid grid-cols-4 gap-x-6 gap-y-5 p-4">
          {CLASSES.map(([n, name]) => (
            <li key={n} className="flex items-center gap-3">
              <CallNumberLabel callNumber={`${n}.12 ABC`} />
              <span className="text-xs text-muted">{n} {name}</span>
            </li>
          ))}
          <li className="flex items-center gap-3">
            <CallNumberLabel callNumber="PR9199.3 .R69 2001" />
            <span className="text-xs text-muted">Non-Dewey scheme</span>
          </li>
        </ul>
      </Panel>

      <Panel aria-labelledby="loans-h">
        <PanelHeader id="loans-h" title="Current loans" aside="3 of 5 loans used" />
        <table className="w-full table-fixed border-collapse">
          <thead>
            <tr className="h-8 text-left text-[11px] font-semibold uppercase tracking-wide text-muted">
              <th className="w-64 border-b border-line px-3">Title / Author</th>
              <th className="w-24 border-b border-line px-3">Call no.</th>
              <th className="w-24 border-b border-line px-3">Accession</th>
              <th className="w-28 border-b border-line px-3">Due</th>
              <th className="border-b border-line px-3">Status</th>
            </tr>
          </thead>
          <tbody>
            {LOANS.map((l) => (
              <tr key={l.acc} className={l.od ? "bg-danger-bg" : "even:bg-band"}>
                <td className="h-[62px] border-b border-line px-3">
                  <b className="block font-semibold">{l.title}</b>
                  <span className="text-xs text-muted">{l.author}</span>
                </td>
                <td className="border-b border-line px-3"><CallNumberLabel callNumber={l.call} /></td>
                <td className="border-b border-line px-3 font-mono text-xs">{l.acc}</td>
                <td className="border-b border-line px-3">{l.due}</td>
                <td className="border-b border-line px-3">
                  <Chip tone={l.od ? "overdue" : "neutral"}>{l.status}</Chip>
                  <span className={`mt-0.5 block text-xs ${l.od ? "text-danger" : "text-muted"}`}>{l.note}</span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        <div className="flex gap-3 px-4 py-3 text-xs text-muted">
          Outstanding fine: <Chip tone="fine">₹8.00</Chip>
        </div>
      </Panel>
    </div>
  );
}
