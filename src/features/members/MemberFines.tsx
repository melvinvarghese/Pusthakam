import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { membersApi, toApiError } from "@/lib/api";
import type { Fine, Member } from "@/lib/api/types";
import { formatMinor } from "@/lib/money";
import { Pager } from "@/components/Pager";
import { Button } from "@/components/ui/Button";
import { Modal } from "@/components/ui/Modal";

const METHODS: Record<string, string> = { cash: "Cash", card: "Card", bank_transfer: "Bank transfer", other: "Other" };

function Settlements({ memberId, fine }: { memberId: number; fine: Fine }) {
  const [page, setPage] = useState(1);
  const result = useQuery({ queryKey: ["member-fines", "history", memberId, fine.id, page], queryFn: () => membersApi.fineHistory(memberId, fine.id, page), retry: false });
  return <div className="space-y-3">
    <p>{fine.title} · Fine #{fine.id} · {fine.currency}</p>
    {result.isPending && <p role="status">Loading settlement history…</p>}
    {result.isError && <div role="alert"><p className="text-danger">{toApiError(result.error).message}</p><Button onClick={() => void result.refetch()}>Retry history</Button></div>}
    {result.data && !result.isError && <>
      {result.data.total === 0 ? <p>No payments or waivers recorded for this fine.</p> : <ul className="divide-y divide-line">
        {result.data.rows.map((entry) => <li key={entry.id} className="py-3">
          <p className="font-semibold">{entry.kind === "payment" ? "Payment" : "Waiver"}: {formatMinor(entry.amountMinor, fine.currency)}</p>
          <p className="text-xs text-muted">{new Date(entry.recordedAt).toLocaleString()} · {entry.staffName}{entry.method ? ` · ${METHODS[entry.method] ?? entry.method}` : ""}</p>
          {entry.reason && <p className="text-sm">{entry.reason}</p>}
        </li>)}
      </ul>}
      <Pager page={result.data.page} pageSize={result.data.pageSize} total={result.data.total} onPage={setPage} />
    </>}
  </div>;
}

export function MemberFines({ member, onClose }: { member: Member; onClose: () => void }) {
  const [page, setPage] = useState(1);
  const [selected, setSelected] = useState<Fine | null>(null);
  const account = useQuery({ queryKey: ["member-fines", "account", member.id, page], queryFn: () => membersApi.fineAccount(member.id, page), retry: false });
  return <Modal title={`${selected ? "Fine history" : "Fines"} for ${member.name}`} wide onClose={selected ? () => setSelected(null) : onClose}
    footer={<>{selected && <Button onClick={() => setSelected(null)}>Back to fines</Button>}<Button onClick={onClose}>Close</Button></>}>
    <p className="font-mono text-xs text-muted">{member.memberNumber}</p>
    {selected ? <Settlements key={selected.id} memberId={member.id} fine={selected} /> : <>
      <p className="text-xs text-muted">Assessed fines only. Overdue estimates may be added when a copy is returned. Totals include all pages and are kept separate by currency. Payments and waivers are recorded in Circulation.</p>
      {account.isPending && <p role="status">Loading member fines…</p>}
      {account.isError && <div role="alert"><p className="text-danger">{toApiError(account.error).message}</p><Button onClick={() => void account.refetch()}>Retry fines</Button></div>}
      {account.data && !account.isError && <>
        {account.data.total === 0 ? <p>This member has no assessed fines.</p> : <>
          <div className="overflow-auto"><table className="w-full text-left text-sm">
            <caption className="mb-2 text-left font-semibold">All assessed fines by currency</caption>
            <thead><tr>{["Currency", "Assessed", "Paid", "Waived", "Outstanding"].map((label) => <th key={label} className="px-2 py-2">{label}</th>)}</tr></thead>
            <tbody>{account.data.balances.map((balance) => <tr key={balance.currency} className="border-t border-line">
              <td className="px-2 py-2">{balance.currency}</td>
              {[balance.assessedMinor, balance.paidMinor, balance.waivedMinor, balance.outstandingMinor].map((value, index) => <td key={index} className="whitespace-nowrap px-2 py-2">{formatMinor(value, balance.currency)}</td>)}
            </tr>)}</tbody>
          </table></div>
          <div className="overflow-auto"><table className="w-full text-left text-sm">
            <caption className="sr-only">Member assessed fine ledger, including settled fines</caption>
            <thead><tr>{["Title / copy", "Assessed", "Fine", "Paid", "Waived", "Outstanding", "History"].map((label) => <th key={label} className="px-2 py-2">{label}</th>)}</tr></thead>
            <tbody>{account.data.rows.map((fine) => <tr key={fine.id} className="border-t border-line">
              <td className="px-2 py-2">{fine.title}<span className="block font-mono text-xs text-muted">{fine.barcode} · #{fine.id}</span></td>
              <td className="px-2 py-2 text-xs">{new Date(fine.assessedAt).toLocaleString()}</td>
              {[fine.amountMinor, fine.paidMinor, fine.waivedMinor, fine.outstandingMinor].map((value, index) => <td key={index} className="whitespace-nowrap px-2 py-2">{formatMinor(value, fine.currency)}</td>)}
              <td className="px-2 py-2"><Button size="sm" variant="ghost" aria-label={`Settlement history for fine ${fine.id}`} onClick={() => setSelected(fine)}>View</Button></td>
            </tr>)}</tbody>
          </table></div>
          <Pager page={account.data.page} pageSize={account.data.pageSize} total={account.data.total} onPage={setPage} />
        </>}
      </>}
    </>}
  </Modal>;
}
