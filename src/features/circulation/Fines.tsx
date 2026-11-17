import { useState, type FormEvent } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Pager } from "@/components/Pager";
import { Button } from "@/components/ui/Button";
import { SelectField, TextField } from "@/components/ui/Field";
import { Modal } from "@/components/ui/Modal";
import { Panel } from "@/components/ui/Panel";
import { circulationApi, toApiError } from "@/lib/api";
import type { Fine, FineSettlement } from "@/lib/api/types";
import { formatMinor, minorToInput, parseMinor } from "@/lib/money";
import { can, useSession } from "@/lib/session";

type Selection = { fine: Fine; kind: FineSettlement["kind"] };
const PAYMENT_METHODS: Record<string, string> = {
  cash: "Cash", card: "Card", bank_transfer: "Bank transfer", other: "Other",
};

function FineHistory({ fine, onClose }: { fine: Fine; onClose: () => void }) {
  const [page, setPage] = useState(1);
  const history = useQuery({
    queryKey: ["circulation", "fines", "history", fine.id, page],
    queryFn: () => circulationApi.fineHistory(fine.id, page),
    retry: false,
  });
  return (
    <Modal title="Fine history" onClose={onClose} wide footer={<Button onClick={onClose}>Close</Button>}>
      <p>{fine.title} · {fine.memberName} ({fine.memberNumber})</p>
      {history.isPending && <p role="status" className="text-muted">Loading fine history.</p>}
      {history.isError && <p role="alert" className="text-danger">{toApiError(history.error).message}</p>}
      {history.data?.total === 0 && <p className="text-muted">No payments or waivers recorded.</p>}
      {history.data && history.data.total > 0 && (
        <>
          <ul className="divide-y divide-line">
            {history.data.rows.map((entry) => (
              <li key={entry.id} className="py-2 text-sm">
                <div className="flex justify-between gap-4 font-medium">
                  <span>{entry.kind === "payment" ? `Payment · ${PAYMENT_METHODS[entry.method ?? ""] ?? "Other"}` : "Waiver"}</span>
                  <span className="shrink-0 font-mono">{formatMinor(entry.amountMinor, fine.currency)}</span>
                </div>
                <div className="text-xs text-muted">{entry.staffName} · {new Date(entry.recordedAt).toLocaleString()}</div>
                {entry.reason && <p className="break-words">{entry.reason}</p>}
              </li>
            ))}
          </ul>
          <Pager page={history.data.page} pageSize={history.data.pageSize} total={history.data.total} onPage={setPage} />
        </>
      )}
    </Modal>
  );
}

function SettlementDialog({ fine, kind, onClose, onSaved }: Selection & {
  onClose: () => void;
  onSaved: () => void;
}) {
  const [amountError, setAmountError] = useState<string | null>(null);
  const client = useQueryClient();
  const isPayment = kind === "payment";
  const title = isPayment ? "Record payment" : "Record waiver";
  const save = useMutation({
    mutationFn: ({ amount, entry }: { amount: number; entry: FineSettlement }) =>
      circulationApi.settleFine(fine.id, amount, fine.outstandingMinor, entry),
    onSuccess: onSaved,
    onError: (error) => {
      if (toApiError(error).code === "CONFLICT") void client.invalidateQueries({ queryKey: ["circulation", "fines"] });
    },
  });

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    const amount = parseMinor(String(form.get("amount") ?? ""));
    if (amount === null || !Number.isSafeInteger(amount) || amount <= 0 || amount > fine.outstandingMinor) {
      setAmountError("Enter an amount greater than zero and no more than the outstanding balance.");
      return;
    }
    setAmountError(null);
    const entry: FineSettlement = isPayment
      ? { kind: "payment", method: String(form.get("method") ?? "") }
      : { kind: "waiver", reason: String(form.get("reason") ?? "") };
    save.mutate({ amount, entry });
  }

  return (
    <Modal title={title} onClose={() => { if (!save.isPending) onClose(); }}>
      <p>{fine.memberName} ({fine.memberNumber})</p>
      <p className="text-sm text-muted">Outstanding {formatMinor(fine.outstandingMinor, fine.currency)}</p>
      <form onSubmit={submit} className="space-y-4">
        <fieldset disabled={save.isPending} className="space-y-3">
          <TextField label={`Amount (${fine.currency})`} name="amount" inputMode="decimal" required
            defaultValue={minorToInput(fine.outstandingMinor)} error={amountError} />
          {isPayment ? (
            <SelectField label="Payment method" name="method">
              {Object.entries(PAYMENT_METHODS).map(([key, label]) => <option key={key} value={key}>{label}</option>)}
            </SelectField>
          ) : <TextField label="Waiver reason" name="reason" maxLength={500} required />}
        </fieldset>
        {save.error && <p role="alert" className="text-danger">{toApiError(save.error).message}</p>}
        <div className="flex justify-end gap-2">
          <Button disabled={save.isPending} onClick={onClose}>Cancel</Button>
          <Button type="submit" variant="primary" disabled={save.isPending}>{title}</Button>
        </div>
      </form>
    </Modal>
  );
}

export default function Fines() {
  const { session } = useSession();
  const client = useQueryClient();
  const [page, setPage] = useState(1);
  const [includeSettled, setIncludeSettled] = useState(false);
  const [selection, setSelection] = useState<Selection | null>(null);
  const [history, setHistory] = useState<Fine | null>(null);
  const [message, setMessage] = useState("");
  const result = useQuery({
    queryKey: ["circulation", "fines", includeSettled, page],
    queryFn: () => circulationApi.listFines(page, includeSettled),
    retry: false,
  });

  return (
    <>
      <div className="max-w-52">
        <SelectField label="Fine status" value={includeSettled ? "all" : "outstanding"}
          onChange={(event) => { setIncludeSettled(event.target.value === "all"); setPage(1); }}>
          <option value="outstanding">Outstanding</option>
          <option value="all">All fines</option>
        </SelectField>
      </div>
      {message && <p role="status">{message}</p>}
      {result.isPending && <p role="status" className="text-muted">Loading fines.</p>}
      {result.isError && <p role="alert" className="text-danger">{toApiError(result.error).message}</p>}
      {result.data?.total === 0 && (
        <Panel className="p-6 text-muted">{includeSettled ? "No fines recorded." : "No outstanding fines."}</Panel>
      )}
      {result.data && result.data.total > 0 && (
        <Panel>
          <div className="overflow-x-auto">
            <table className="w-full border-collapse text-left text-sm">
              <caption className="sr-only">Overdue fines</caption>
              <thead className="border-b border-line text-xs text-muted">
                <tr>{["Member", "Title", "Fine", "Paid", "Waived", "Outstanding", "Actions"].map((label) => (
                  <th key={label} scope="col" className="px-4 py-2">{label}</th>
                ))}</tr>
              </thead>
              <tbody>
                {result.data.rows.map((fine) => (
                  <tr key={fine.id} className="border-b border-line last:border-b-0">
                    <td className="px-4 py-2">{fine.memberName}<div className="font-mono text-xs text-muted">{fine.memberNumber}</div></td>
                    <td className="px-4 py-2">{fine.title}</td>
                    {[fine.amountMinor, fine.paidMinor, fine.waivedMinor, fine.outstandingMinor].map((amount, index) => (
                      <td key={index} className="whitespace-nowrap px-4 py-2">{formatMinor(amount, fine.currency)}</td>
                    ))}
                    <td className="whitespace-nowrap px-4 py-2">
                      <Button size="sm" variant="ghost" aria-label={`Fine history for ${fine.memberNumber}`}
                        onClick={() => setHistory(fine)}>History</Button>
                      {fine.outstandingMinor > 0 && <>
                        <Button size="sm" variant="ghost" onClick={() => setSelection({ fine, kind: "payment" })}>Record payment</Button>
                        {can(session, "circulation.manage") &&
                          <Button size="sm" variant="ghost" onClick={() => setSelection({ fine, kind: "waiver" })}>Waive</Button>}
                      </>}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <Pager page={result.data.page} pageSize={result.data.pageSize} total={result.data.total} onPage={setPage} />
        </Panel>
      )}
      {selection && <SettlementDialog {...selection} onClose={() => setSelection(null)} onSaved={() => {
        setMessage(selection.kind === "payment" ? "Payment recorded." : "Waiver recorded.");
        setSelection(null);
        void client.invalidateQueries({ queryKey: ["circulation", "fines"] });
      }} />}
      {history && <FineHistory fine={history} onClose={() => setHistory(null)} />}
    </>
  );
}
