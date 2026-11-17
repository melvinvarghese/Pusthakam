import { useState, type FormEvent } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { PageHeader } from "@/components/PageHeader";
import { Pager } from "@/components/Pager";
import { Button } from "@/components/ui/Button";
import { SelectField, TextField } from "@/components/ui/Field";
import { Modal } from "@/components/ui/Modal";
import { Panel } from "@/components/ui/Panel";
import { useConfig } from "@/features/catalogue/shared";
import { inventoryApi, toApiError } from "@/lib/api";
import type { StockTransfer } from "@/lib/api/types";

const STATUSES = { in_transit: "In transit", received: "Received", cancelled: "Cancelled" };

function SendDialog({ onClose, onSaved }: { onClose: () => void; onSaved: () => Promise<void> }) {
  const config = useConfig();
  const [location, setLocation] = useState("");
  const [shelf, setShelf] = useState("");
  const send = useMutation({
    mutationFn: (barcode: string) => inventoryApi.sendTransfer(barcode, Number(location), shelf ? Number(shelf) : null),
    onSuccess: onSaved,
  });
  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const barcode = String(new FormData(event.currentTarget).get("barcode") ?? "").trim();
    if (barcode && location && !send.isPending) send.mutate(barcode);
  }
  return (
    <Modal title="Send copy transfer" onClose={() => { if (!send.isPending) onClose(); }}>
      <form className="space-y-4" onSubmit={submit}>
        <TextField label="Copy barcode" name="barcode" required maxLength={120} autoFocus disabled={send.isPending} />
        <SelectField label="Destination location" value={location} required disabled={send.isPending}
          onChange={(event) => { setLocation(event.target.value); setShelf(""); }}>
          <option value="">Select location</option>
          {config.data?.locations.filter((entry) => !entry.archived).map((entry) => <option key={entry.id} value={entry.id}>{entry.name}</option>)}
        </SelectField>
        <SelectField label="Destination shelf" value={shelf} disabled={!location || send.isPending}
          onChange={(event) => setShelf(event.target.value)}>
          <option value="">Unassigned shelf</option>
          {config.data?.shelves.filter((entry) => !entry.archived && String(entry.locationId) === location)
            .map((entry) => <option key={entry.id} value={entry.id}>{entry.code}</option>)}
        </SelectField>
        {config.isError && <p role="alert" className="text-danger">{toApiError(config.error).message}</p>}
        {send.isError && <p role="alert" className="text-danger">{toApiError(send.error).message}</p>}
        <div className="flex justify-end gap-2">
          <Button onClick={onClose} disabled={send.isPending}>Cancel</Button>
          <Button type="submit" variant="primary" disabled={!location || send.isPending}>{send.isPending ? "Sending…" : "Send transfer"}</Button>
        </div>
      </form>
    </Modal>
  );
}

export function TransfersPanel({ writable, onBack }: { writable: boolean; onBack: () => void }) {
  const client = useQueryClient();
  const [page, setPage] = useState(1);
  const [sending, setSending] = useState(false);
  const [selection, setSelection] = useState<{ row: StockTransfer; receive: boolean } | null>(null);
  const list = useQuery({ queryKey: ["stock-transfers", page], queryFn: () => inventoryApi.transfers(page), retry: false });
  async function refresh() {
    await Promise.all([
      client.invalidateQueries({ queryKey: ["stock-transfers"] }),
      client.invalidateQueries({ queryKey: ["catalogue"] }),
    ]);
  }
  const finish = useMutation({
    mutationFn: ({ row, receive }: { row: StockTransfer; receive: boolean }) => inventoryApi.finishTransfer(row.id, receive),
    onSuccess: async () => { await refresh(); setSelection(null); },
  });
  function review(row: StockTransfer, receive: boolean) {
    finish.reset();
    setSelection({ row, receive });
  }

  return (
    <div className="mx-auto max-w-[1100px] space-y-5 p-6">
      <PageHeader title="Copy transfers" actions={<>
        <Button onClick={onBack}>Stock checks</Button>
        {writable && <Button variant="primary" onClick={() => setSending(true)}>Send copy transfer</Button>}
      </>} />
      {list.isPending && <p role="status" className="text-muted">Loading transfers.</p>}
      {list.isError && <p role="alert" className="text-danger">{toApiError(list.error).message}</p>}
      {list.data && <Panel>
        {list.data.rows.length === 0 ? <p className="p-4 text-muted">No copy transfers yet.</p> : (
          <ul className="divide-y divide-line">
            {list.data.rows.map((row) => (
              <li key={row.id} className="flex flex-wrap items-center justify-between gap-3 px-4 py-3">
                <div className="space-y-1 text-sm">
                  <p className="font-medium">{row.title} <span className="font-mono text-xs text-muted">{row.barcode}</span></p>
                  <p>{row.source} → {row.destination}</p>
                  <p className="text-xs text-muted">{STATUSES[row.status]} · Sent {new Date(row.sentAt).toLocaleString()}
                    {row.finishedAt && ` · Closed ${new Date(row.finishedAt).toLocaleString()}`}
                  </p>
                </div>
                {writable && row.status === "in_transit" && <div className="flex gap-2">
                  <Button size="sm" variant="primary" onClick={() => review(row, true)}>Receive</Button>
                  <Button size="sm" onClick={() => review(row, false)}>Cancel transfer</Button>
                </div>}
              </li>
            ))}
          </ul>
        )}
        <Pager page={list.data.page} pageSize={list.data.pageSize} total={list.data.total} onPage={setPage} />
      </Panel>}
      {sending && <SendDialog onClose={() => setSending(false)} onSaved={async () => {
        await refresh(); setPage(1); setSending(false);
      }} />}
      {selection && <Modal title={selection.receive ? "Receive copy" : "Cancel transfer"} onClose={() => { if (!finish.isPending) setSelection(null); }}
        footer={<>
          <Button disabled={finish.isPending} onClick={() => setSelection(null)}>Back</Button>
          <Button variant="primary" disabled={finish.isPending} onClick={() => finish.mutate(selection)}>
            {selection.receive ? "Confirm receipt" : "Confirm cancellation"}
          </Button>
        </>}>
        <p className="text-sm">{selection.row.title} <span className="font-mono">{selection.row.barcode}</span></p>
        <p className="text-sm">{selection.receive ? `Receive at ${selection.row.destination}.` : `Return to ${selection.row.source}.`}</p>
        {finish.isError && <p role="alert" className="text-danger">{toApiError(finish.error).message}</p>}
      </Modal>}
    </div>
  );
}
