import { useEffect, useId, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Button } from "@/components/ui/Button";
import { TextField } from "@/components/ui/Field";
import { Modal } from "@/components/ui/Modal";
import { catalogueApi, toApiError } from "@/lib/api";
import type { RecordSummary } from "@/lib/api/types";
import { STATUS_LABELS } from "@/features/catalogue/shared";

export function FindBook({ onClose, onSelect, purpose = "issue" }: { purpose?: "issue" | "reservation"; onClose: () => void; onSelect: (barcode: string) => void }) {
  const [name, setName] = useState("");
  const [search, setSearch] = useState("");
  const [page, setPage] = useState(1);
  const [active, setActive] = useState(-1);
  const [title, setTitle] = useState<RecordSummary | null>(null);
  const listId = useId();
  const term = name.trim();
  useEffect(() => { const timer = setTimeout(() => setSearch(term), 200); return () => clearTimeout(timer); }, [term]);
  const current = term.length > 0 && term === search;
  const matches = useQuery({ queryKey: ["catalogue", "issue-lookup", search, page],
    queryFn: () => catalogueApi.search({ text: search, page, pageSize: 25, includeArchived: false, sort: "title" }), enabled: current, retry: false });
  const result = current && !matches.isError ? matches.data : undefined;
  const rows = result?.rows ?? [];
  const selected = rows[active];
  const selectedId = selected?.id;
  useEffect(() => {
    if (selectedId !== undefined) document.getElementById(`${listId}-${selectedId}`)?.scrollIntoView?.({ block: "nearest" });
  }, [selectedId, listId]);
  const detail = useQuery({ queryKey: ["catalogue", "issue-copies", title?.id],
    queryFn: () => catalogueApi.getRecord(title!.id), enabled: title !== null, retry: false });
  const copies = title && !detail.data?.archived ? detail.data?.copies : undefined;
  return <Modal title="Find book by name" onClose={onClose} footer={<Button onClick={onClose}>Close</Button>}>
    <TextField label="Book title" role="combobox" value={name} autoComplete="off" maxLength={120}
      aria-autocomplete="list" aria-haspopup="listbox" aria-controls={listId} aria-expanded={rows.length > 0 && !title}
      aria-activedescendant={!title && selected ? `${listId}-${selected.id}` : undefined}
      hint={`Type part of a title, choose a match, then select the copy to ${purpose === "issue" ? "issue" : "reserve"}.`}
      onChange={(e) => { setName(e.target.value); setPage(1); setActive(-1); setTitle(null); }}
      onKeyDown={(e) => {
        if (e.key === "ArrowDown" && rows.length && !title) { e.preventDefault(); setActive((i) => Math.min(i + 1, rows.length - 1)); }
        if (e.key === "ArrowUp" && rows.length && !title) { e.preventDefault(); setActive((i) => Math.max(i - 1, 0)); }
        if (e.key === "Enter") { e.preventDefault(); if (selected && !title) setTitle(selected); }
      }} />
    {term && (!current || matches.isFetching) && <p role="status">Searching books.</p>}
    {current && matches.isError && <div role="alert">{toApiError(matches.error).message}<Button onClick={() => void matches.refetch()}>Retry search</Button></div>}
    {result && !title && <>
      {rows.length === 0 ? <p role="status">No matching books.</p> : <ul id={listId} role="listbox" aria-label="Matching books" className="max-h-64 overflow-auto rounded-md border border-line-strong">
        {rows.map((row, index) => <li key={row.id} role="presentation"><button type="button" role="option" id={`${listId}-${row.id}`} aria-selected={active === index}
          className={`block w-full px-3 py-2 text-left text-sm hover:bg-band ${active === index ? "bg-band" : ""}`} onClick={() => setTitle(row)}>
          <span className="block font-semibold">{row.title}</span><span className="text-muted">{row.authors || "Author not recorded"}{row.publicationYear ? ` · ${row.publicationYear}` : ""} · {row.availableCopies} available</span>
        </button></li>)}
      </ul>}
      {result.total > result.pageSize && <div className="flex items-center gap-2">
        <Button disabled={page <= 1 || matches.isFetching} onClick={() => { setPage((p) => p - 1); setActive(-1); }}>Previous matches</Button>
        <span className="text-xs text-muted">Page {result.page} of {Math.ceil(result.total / result.pageSize)}</span>
        <Button disabled={page * result.pageSize >= result.total || matches.isFetching} onClick={() => { setPage((p) => p + 1); setActive(-1); }}>Next matches</Button>
      </div>}
    </>}
    {title && <section aria-label="Choose a copy" className="space-y-2">
      <h3 className="font-semibold">{title.title}</h3>
      {detail.isPending && <p role="status">Loading copies.</p>}
      {detail.isError && <div role="alert">{toApiError(detail.error).message}<Button onClick={() => void detail.refetch()}>Retry copies</Button></div>}
      {detail.data?.archived && <p role="alert">This title is archived. Choose another book.</p>}
      {copies?.length === 0 && <p role="status">This title has no copies.</p>}
      <ul className="max-h-64 space-y-2 overflow-auto">{copies?.map((copy) => <li key={copy.id}>
        <Button disabled={!["available", "reserved_hold", ...(purpose === "reservation" ? ["on_loan"] : [])].includes(copy.status)} onClick={() => onSelect(copy.barcode)}>
          {copy.accessionNo} · {copy.barcode} · {STATUS_LABELS[copy.status]}{copy.callNumber ? ` · ${copy.callNumber}` : ""}
        </Button>
      </li>)}</ul>
      <p className="text-xs text-muted">{purpose === "issue" ? "Availability and reservation eligibility are checked again when you review the loan." : "Member and copy eligibility are checked when you place the reservation."}</p>
      <Button onClick={() => setTitle(null)}>Choose another title</Button>
    </section>}
  </Modal>;
}
