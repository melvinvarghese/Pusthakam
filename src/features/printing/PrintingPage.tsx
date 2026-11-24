import { useState, type FormEvent } from "react";
import { useMutation, useQuery } from "@tanstack/react-query";
import { PageHeader } from "@/components/PageHeader";
import { Button } from "@/components/ui/Button";
import { TextField } from "@/components/ui/Field";
import { Panel } from "@/components/ui/Panel";
import { Pager } from "@/components/Pager";
import { AccessionPage } from "@/features/catalogue/AccessionPage";
import { useConfig, useDebounced } from "@/features/catalogue/shared";
import { MemberCard } from "@/features/members/MemberCard";
import { DueSlip } from "@/features/circulation/DueSlip";
import { circulationApi, membersApi, toApiError } from "@/lib/api";
import type { Member } from "@/lib/api/types";
import { can, useSession } from "@/lib/session";

function Cards() {
  const config = useConfig();
  const [name, setName] = useState("");
  const search = useDebounced(name.trim(), 200);
  const [page, setPage] = useState(1);
  const [card, setCard] = useState<Member | null>(null);
  const result = useQuery({ queryKey: ["printing", "members", search, page], queryFn: () => membersApi.list(search, page, false), retry: false });
  const current = search === name.trim();
  return <Panel className="space-y-3 p-4">
    <TextField label="Find member card" value={name} onChange={(e) => { setName(e.target.value); setPage(1); }} hint="Search active members by name or member number." />
    {(!current || result.isFetching) && <p role="status">Loading members.</p>}
    {result.isError && <p role="alert">{toApiError(result.error).message}</p>}
    {current && result.data && <>
      {result.data.rows.length === 0 && <p>No matching members.</p>}
      <ul className="space-y-2">{result.data.rows.map((member) => <li key={member.id} className="flex items-center justify-between gap-3">
        <span>{member.name} ({member.memberNumber})</span>
        <Button disabled={!config.data?.library} onClick={() => setCard(member)}>Preview card for {member.memberNumber}</Button>
      </li>)}</ul>
      <Pager page={result.data.page} pageSize={result.data.pageSize} total={result.data.total} onPage={setPage} />
    </>}
    {config.isError && <p role="alert">{toApiError(config.error).message}</p>}
    {card && config.data?.library && <MemberCard member={card} libraryName={config.data.library.name} onClose={() => setCard(null)} />}
  </Panel>;
}
function Slips() {
  const [barcode, setBarcode] = useState("");
  const [open, setOpen] = useState(false);
  const lookup = useMutation({ mutationFn: () => circulationApi.prepareReturn(barcode.trim()), onSuccess: () => setOpen(true) });
  function submit(e: FormEvent) { e.preventDefault(); lookup.mutate(); }
  return <Panel className="space-y-3 p-4">
    <p>Reprint the current due slip for an open loan. This does not return, renew or issue a copy.</p>
    <form onSubmit={submit} className="space-y-3">
      <TextField label="Book barcode or accession number" required maxLength={40} value={barcode} onChange={(e) => { setBarcode(e.target.value); lookup.reset(); }} />
      <Button type="submit" disabled={lookup.isPending || !barcode.trim()}>{lookup.isPending ? "Finding loan." : "Preview due slip"}</Button>
    </form>
    {lookup.error && <p role="alert">{toApiError(lookup.error).message}</p>}
    {open && lookup.data && <DueSlip loan={lookup.data} renewed={false} reprint onClose={() => setOpen(false)} />}
  </Panel>;
}
export default function PrintingPage() {
  const { session } = useSession();
  const [tab, setTab] = useState("labels");
  const permissions = { labels: "catalogue.read", cards: "members.read", slips: "circulation.issue" };
  return <div className="mx-auto max-w-[1100px] space-y-4 p-6">
    <PageHeader title="Barcode & Printing" description="Preview and print copy labels, spine labels, member cards and loan due slips." />
    <div role="group" aria-label="Print tools" className="flex flex-wrap gap-2">
      {[["labels", "Book labels"], ["cards", "Member cards"], ["slips", "Loan due slips"]].map(([value, label]) => <Button key={value} aria-pressed={tab === value} variant={tab === value ? "primary" : "secondary"} onClick={() => setTab(value!)}>{label}</Button>)}
    </div>
    {!can(session, permissions[tab as keyof typeof permissions]) ? <p role="alert">You do not have permission to use this print tool.</p> :
      tab === "labels" ? <><p className="text-sm text-muted">Search and select copies below, then preview labels. Choose barcode labels or spine labels in the preview.</p><AccessionPage printing /></> : tab === "cards" ? <Cards /> : <Slips />}
  </div>;
}
