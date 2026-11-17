import { useState, type FormEvent } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Pager } from "@/components/Pager";
import { Button } from "@/components/ui/Button";
import { TextField } from "@/components/ui/Field";
import { Modal } from "@/components/ui/Modal";
import { Panel } from "@/components/ui/Panel";
import { circulationApi, toApiError } from "@/lib/api";
import type { Reservation } from "@/lib/api/types";

export default function Reservations() {
  const client = useQueryClient();
  const [page, setPage] = useState(1);
  const [barcode, setBarcode] = useState("");
  const [message, setMessage] = useState("");
  const [selected, setSelected] = useState<Reservation | null>(null);
  const list = useQuery({
    queryKey: ["circulation", "reservations", page],
    queryFn: () => circulationApi.listReservations(page),
    retry: false,
    refetchInterval: 60_000,
  });

  function invalidate() {
    void client.invalidateQueries({ queryKey: ["circulation", "reservations"] });
    void client.invalidateQueries({ queryKey: ["catalogue"] });
  }

  const place = useMutation({
    mutationFn: ({ memberNumber, barcode }: { memberNumber: string; barcode: string }) =>
      circulationApi.placeReservation(memberNumber, barcode),
    onSuccess: () => {
      setPage(1);
      setBarcode("");
      setMessage("Reservation placed.");
      invalidate();
    },
  });
  const cancel = useMutation({
    mutationFn: circulationApi.cancelReservation,
    onSuccess: () => {
      setSelected(null);
      setMessage("Reservation cancelled.");
      invalidate();
    },
  });

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const fields = new FormData(event.currentTarget);
    setMessage("");
    place.mutate({ memberNumber: String(fields.get("memberNumber") ?? ""), barcode });
  }

  return (
    <>
      <Panel className="max-w-xl p-4">
        <form onSubmit={submit} className="space-y-4">
          <fieldset disabled={place.isPending} className="grid gap-3 sm:grid-cols-2">
            <TextField label="Member number" name="memberNumber" required maxLength={64} autoComplete="off"
              onKeyDown={(event) => {
                if (event.key !== "Enter" || !event.currentTarget.value.trim()) return;
                event.preventDefault();
                event.currentTarget.form?.querySelector<HTMLInputElement>('input[name="barcode"]')?.focus();
              }} />
            <TextField label="Book barcode or accession number" name="barcode" required maxLength={40} autoComplete="off"
              value={barcode} onChange={(event) => setBarcode(event.target.value)} />
          </fieldset>
          <Button type="submit" variant="primary" disabled={place.isPending}>Place reservation</Button>
        </form>
        {place.error && <p role="alert" className="mt-3 text-danger">{toApiError(place.error).message}</p>}
      </Panel>
      {message && <p role="status">{message}</p>}
      {list.isPending && <p role="status" className="text-muted">Loading reservations.</p>}
      {list.isError && <p role="alert" className="text-danger">{toApiError(list.error).message}</p>}
      {list.data?.total === 0 && <Panel className="p-6 text-muted">No active reservations.</Panel>}
      {list.data && list.data.total > 0 && (
        <Panel>
          <div className="overflow-x-auto">
            <table className="w-full border-collapse text-left text-sm">
              <caption className="sr-only">Active copy reservations</caption>
              <thead className="border-b border-line text-xs text-muted">
                <tr>{["Title", "Member", "Status", "Hold expires", "Action"].map((label) => (
                  <th key={label} scope="col" className="px-4 py-2">{label}</th>
                ))}</tr>
              </thead>
              <tbody>
                {list.data.rows.map((row) => (
                  <tr key={row.id} className="border-b border-line last:border-b-0">
                    <td className="px-4 py-2">{row.title}<div className="font-mono text-xs text-muted">{row.barcode}</div></td>
                    <td className="px-4 py-2">{row.memberName}<div className="font-mono text-xs text-muted">{row.memberNumber}</div></td>
                    <td className="whitespace-nowrap px-4 py-2">{row.status === "ready" ? "Ready for pickup" : "Waiting"}</td>
                    <td className="whitespace-nowrap px-4 py-2">{row.expiresAt ? new Date(row.expiresAt).toLocaleString() : "—"}</td>
                    <td className="px-4 py-2">
                      <Button size="sm" variant="ghost" aria-label={`Cancel reservation for ${row.memberNumber}, copy ${row.barcode}`}
                        onClick={() => { cancel.reset(); setSelected(row); }}>Cancel</Button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <Pager page={list.data.page} pageSize={list.data.pageSize} total={list.data.total} onPage={setPage} />
        </Panel>
      )}
      {selected && (
        <Modal title="Cancel reservation" onClose={() => { if (!cancel.isPending) setSelected(null); }}
          footer={<>
            <Button disabled={cancel.isPending} onClick={() => setSelected(null)}>Keep reservation</Button>
            <Button variant="primary" disabled={cancel.isPending} onClick={() => cancel.mutate(selected.id)}>Cancel reservation</Button>
          </>}>
          <p>{selected.title} for {selected.memberName} ({selected.memberNumber}).</p>
          {cancel.error && <p role="alert" className="text-danger">{toApiError(cancel.error).message}</p>}
        </Modal>
      )}
    </>
  );
}
