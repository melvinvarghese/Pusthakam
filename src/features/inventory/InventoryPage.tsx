import { useRef, useState, type FormEvent } from "react";
import { keepPreviousData, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { PageHeader } from "@/components/PageHeader";
import { Pager } from "@/components/Pager";
import { Button } from "@/components/ui/Button";
import { SelectField } from "@/components/ui/Field";
import { Modal } from "@/components/ui/Modal";
import { Panel } from "@/components/ui/Panel";
import { useConfig } from "@/features/catalogue/shared";
import { inventoryApi, toApiError } from "@/lib/api";
import type { StockCheckView } from "@/lib/api/types";
import { can, useSession } from "@/lib/session";
import { TransfersPanel } from "./TransfersPanel";

function StartCheck({ onOpen, onClose }: { onOpen: (id: number) => void; onClose: () => void }) {
  const config = useConfig();
  const [location, setLocation] = useState("");
  const [shelf, setShelf] = useState("");
  const start = useMutation({
    mutationFn: () => inventoryApi.start(Number(location), shelf ? Number(shelf) : null),
    onSuccess: onOpen,
  });
  const locations = config.data?.locations.filter((entry) => !entry.archived) ?? [];
  const shelves = config.data?.shelves.filter((entry) => !entry.archived && String(entry.locationId) === location) ?? [];
  return (
    <Modal title="Start stock check" onClose={() => { if (!start.isPending) onClose(); }}>
      <form className="space-y-4" onSubmit={(event) => { event.preventDefault(); if (location && !start.isPending) start.mutate(); }}>
        <p className="text-sm text-muted">The check starts with the available copies recorded at the selected location or shelf.</p>
        <SelectField label="Location" value={location} required disabled={start.isPending}
          onChange={(event) => { setLocation(event.target.value); setShelf(""); }}>
          <option value="">Select location</option>
          {locations.map((entry) => <option key={entry.id} value={entry.id}>{entry.name}</option>)}
        </SelectField>
        <SelectField label="Shelf" value={shelf} disabled={!location || start.isPending}
          onChange={(event) => setShelf(event.target.value)}>
          <option value="">All shelves</option>
          {shelves.map((entry) => <option key={entry.id} value={entry.id}>{entry.code}</option>)}
        </SelectField>
        {config.isError && <p role="alert" className="text-danger">{toApiError(config.error).message}</p>}
        {config.data && !locations.length && <p className="text-muted">Add a location in Library Settings to start.</p>}
        {start.isError && <p role="alert" className="text-danger">{toApiError(start.error).message}</p>}
        <div className="flex justify-end gap-2">
          <Button disabled={start.isPending} onClick={onClose}>Cancel</Button>
          <Button type="submit" variant="primary" disabled={!location || start.isPending}>
            {start.isPending ? "Starting…" : "Start check"}
          </Button>
        </div>
      </form>
    </Modal>
  );
}

const SCAN_MESSAGES = { matched: "Matched an expected copy.", duplicate: "This barcode was already scanned.", unexpected: "Recorded an unexpected barcode." };

function CheckReview({ id, writable, onBack }: { id: number; writable: boolean; onBack: () => void }) {
  const client = useQueryClient();
  const input = useRef<HTMLInputElement>(null);
  const [view, setView] = useState<StockCheckView>("missing");
  const [page, setPage] = useState(1);
  const [barcode, setBarcode] = useState("");
  const [closing, setClosing] = useState(false);
  const review = useQuery({
    queryKey: ["stock-check", id, view, page],
    queryFn: () => inventoryApi.review(id, view, page),
    placeholderData: keepPreviousData,
    retry: false,
  });
  const refresh = () => client.invalidateQueries({ queryKey: ["stock-check", id] });
  const scan = useMutation({
    mutationFn: (code: string) => inventoryApi.scan(id, code),
    onSuccess: () => { setBarcode(""); input.current?.focus(); void refresh(); },
  });
  const close = useMutation({
    mutationFn: () => inventoryApi.close(id),
    onSuccess: async () => { await refresh(); setClosing(false); },
  });
  const check = review.data?.check;
  function submitScan(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (barcode.trim() && !scan.isPending) scan.mutate(barcode);
  }

  return (
    <div className="mx-auto max-w-[1100px] space-y-5 p-6">
      <PageHeader title={check ? [check.location, check.shelf].filter(Boolean).join(" / ") : "Stock check"}
        actions={<Button onClick={onBack}>All stock checks</Button>} />
      {review.isPending && <p role="status" className="text-muted">Loading stock check.</p>}
      {review.isError && <p role="alert" className="text-danger">{toApiError(review.error).message}</p>}
      {check && <>
        <p className="text-sm text-muted">{check.closedAt ? "Closed" : "In progress"} · Started {new Date(check.startedAt).toLocaleString()}</p>
        {writable && !check.closedAt && (
          <Panel className="space-y-2 p-4">
            <form className="flex flex-wrap items-end gap-3" onSubmit={submitScan}>
              <label className="min-w-48 flex-1 text-sm">
                <span className="mb-1 block font-medium">Scan barcode</span>
                <input ref={input} autoFocus value={barcode} readOnly={scan.isPending} maxLength={120}
                  onChange={(event) => setBarcode(event.target.value)}
                  className="h-9 w-full rounded-md border border-line-strong bg-surface px-3 font-mono" />
              </label>
              <Button type="submit" variant="primary" disabled={!barcode.trim() || scan.isPending}>Record scan</Button>
              <Button disabled={scan.isPending} onClick={() => setClosing(true)}>Close stock check</Button>
            </form>
            {scan.data && <p role="status" className="text-sm">{SCAN_MESSAGES[scan.data]}</p>}
            {scan.isError && <p role="alert" className="text-danger">{toApiError(scan.error).message}</p>}
          </Panel>
        )}
        <div className="flex flex-wrap items-center gap-2">
          {([
            ["missing", "Missing", check.expected - check.matched],
            ["matched", "Matched", check.matched],
            ["unexpected", "Unexpected", check.unexpected],
          ] as const).map(([key, label, count]) => (
            <Button key={key} aria-pressed={view === key} variant={view === key ? "primary" : "ghost"}
              onClick={() => { setView(key); setPage(1); }}>{label} ({count})</Button>
          ))}
        </div>
        {review.isPlaceholderData && <p role="status" className="text-muted">Loading copies.</p>}
        {!review.isPlaceholderData && review.data?.entries && <Panel>
          {review.data.entries.rows.length === 0 ? <p className="p-4 text-muted">No {view} copies.</p> : (
            <ul className="divide-y divide-line" aria-label={`${view} copies`}>
              {review.data.entries.rows.map((entry) => (
                <li key={entry.barcode} className="flex flex-wrap justify-between gap-2 px-4 py-3 text-sm">
                  <span>{entry.title ?? "Unknown barcode"}</span>
                  <span className="font-mono text-xs text-muted">{entry.accessionNo && `${entry.accessionNo} · `}{entry.barcode}</span>
                </li>
              ))}
            </ul>
          )}
          <Pager page={review.data.entries.page} total={review.data.entries.total} pageSize={review.data.entries.pageSize} onPage={setPage} />
        </Panel>}
      </>}
      {closing && (
        <Modal title="Close stock check" onClose={() => { if (!close.isPending) setClosing(false); }}
          footer={<>
            <Button disabled={close.isPending} onClick={() => setClosing(false)}>Cancel</Button>
            <Button variant="primary" disabled={close.isPending} onClick={() => close.mutate()}>Close check</Button>
          </>}>
          <p className="text-sm">Closing saves the final matched, missing and unexpected counts for review.</p>
          {close.isError && <p role="alert" className="text-danger">{toApiError(close.error).message}</p>}
        </Modal>
      )}
    </div>
  );
}

export function InventoryPage() {
  const { session } = useSession();
  const allowed = can(session, "catalogue.read");
  const writable = can(session, "catalogue.write");
  const [page, setPage] = useState(1);
  const [active, setActive] = useState<number | null>(null);
  const [starting, setStarting] = useState(false);
  const [transfersOpen, setTransfersOpen] = useState(false);
  const checks = useQuery({ queryKey: ["stock-checks", page], queryFn: () => inventoryApi.list(page), enabled: allowed && active === null && !transfersOpen, retry: false });
  if (!allowed) return <p role="alert" className="p-6">You do not have permission to view stock checks.</p>;
  if (active !== null) return <CheckReview key={active} id={active} writable={writable} onBack={() => setActive(null)} />;
  if (transfersOpen) return <TransfersPanel writable={writable} onBack={() => setTransfersOpen(false)} />;

  return (
    <div className="mx-auto max-w-[1100px] space-y-5 p-6">
      <PageHeader title="Inventory" description="Check the copies on a shelf or at a location."
        actions={<>
          <Button onClick={() => setTransfersOpen(true)}>Copy transfers</Button>
          {writable && <Button variant="primary" onClick={() => setStarting(true)}>Start stock check</Button>}
        </>} />
      {checks.isPending && <p role="status" className="text-muted">Loading stock checks.</p>}
      {checks.isError && <p role="alert" className="text-danger">{toApiError(checks.error).message}</p>}
      {checks.data && <Panel>
        {checks.data.rows.length === 0 ? <p className="p-4 text-muted">No stock checks yet.</p> : (
          <ul className="divide-y divide-line">
            {checks.data.rows.map((row) => (
              <li key={row.id} className="flex flex-wrap items-center justify-between gap-3 px-4 py-3">
                <div>
                  <p className="font-medium">{[row.location, row.shelf].filter(Boolean).join(" / ")}</p>
                  <p className="text-xs text-muted">{row.closedAt ? "Closed" : "In progress"} · {new Date(row.startedAt).toLocaleString()} · {row.matched} of {row.expected} matched</p>
                </div>
                <Button size="sm" onClick={() => setActive(row.id)}>{row.closedAt ? "Review" : "Open check"}</Button>
              </li>
            ))}
          </ul>
        )}
        <Pager page={checks.data.page} total={checks.data.total} pageSize={checks.data.pageSize} onPage={setPage} />
      </Panel>}
      {starting && <StartCheck onClose={() => setStarting(false)} onOpen={(id) => { setStarting(false); setActive(id); }} />}
    </div>
  );
}
