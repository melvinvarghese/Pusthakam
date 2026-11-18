import { useEffect, useId, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Button } from "@/components/ui/Button";
import { TextField } from "@/components/ui/Field";
import { Modal } from "@/components/ui/Modal";
import { membersApi, toApiError } from "@/lib/api";

export function FindMember({ onClose, onSelect }: { onClose: () => void; onSelect: (number: string) => void }) {
  const [name, setName] = useState("");
  const [search, setSearch] = useState("");
  const [page, setPage] = useState(1);
  const [active, setActive] = useState(-1);
  const listId = useId();
  const term = name.trim();
  useEffect(() => {
    const timer = setTimeout(() => setSearch(term), 200);
    return () => clearTimeout(timer);
  }, [term]);
  const current = term.length > 0 && term === search;
  const members = useQuery({
    queryKey: ["circulation", "member-lookup", search, page],
    queryFn: () => membersApi.list(search, page, false),
    enabled: current,
    retry: false,
  });
  const result = current && !members.isError ? members.data : undefined;
  const rows = result?.rows ?? [];
  const selected = rows[active];
  const selectedId = selected?.id;
  useEffect(() => {
    if (selectedId !== undefined) document.getElementById(`${listId}-${selectedId}`)?.scrollIntoView?.({ block: "nearest" });
  }, [selectedId, listId]);
  return (
    <Modal title="Find member by name" onClose={onClose} footer={<Button onClick={onClose}>Close</Button>}>
      <TextField label="Member name" role="combobox" value={name} autoComplete="off" maxLength={120}
        aria-autocomplete="list" aria-haspopup="listbox" aria-controls={listId} aria-expanded={rows.length > 0}
        aria-activedescendant={selected ? `${listId}-${selected.id}` : undefined}
        hint="Type a name or member number, then choose a match."
        onChange={(event) => { setName(event.target.value); setPage(1); setActive(-1); }}
        onKeyDown={(event) => {
          if (event.key === "ArrowDown" && rows.length) { event.preventDefault(); setActive((i) => Math.min(i + 1, rows.length - 1)); }
          if (event.key === "ArrowUp" && rows.length) { event.preventDefault(); setActive((i) => Math.max(i - 1, 0)); }
          if (event.key === "Enter") { event.preventDefault(); if (selected) onSelect(selected.memberNumber); }
        }} />
      {term && (!current || members.isFetching) && <p role="status">Searching members…</p>}
      {current && members.isError && <div role="alert"><p className="text-danger">{toApiError(members.error).message}</p><Button onClick={() => void members.refetch()}>Retry search</Button></div>}
      {result && !members.isError && <>
        {rows.length === 0 ? <p role="status">No matching members.</p> : <ul id={listId} role="listbox" aria-label="Matching members" className="max-h-64 overflow-auto rounded-md border border-line-strong bg-surface shadow-sm">
          {rows.map((member, index) => <li key={member.id} role="presentation">
            <button type="button" role="option" id={`${listId}-${member.id}`} aria-selected={active === index} tabIndex={-1}
              className={`block w-full px-3 py-2 text-left text-sm hover:bg-band ${active === index ? "bg-band" : ""}`}
              onMouseDown={(event) => event.preventDefault()}
              onClick={() => onSelect(member.memberNumber)}>
              {member.name} ({member.memberNumber})
            </button>
          </li>)}
        </ul>}
        {result.total > result.pageSize && <div className="flex items-center gap-2">
          <Button disabled={page <= 1 || members.isFetching} onClick={() => { setPage((p) => p - 1); setActive(-1); }}>Previous matches</Button>
          <span className="text-xs text-muted">Page {result.page} of {Math.ceil(result.total / result.pageSize)}</span>
          <Button disabled={page * result.pageSize >= result.total || members.isFetching} onClick={() => { setPage((p) => p + 1); setActive(-1); }}>Next matches</Button>
        </div>}
      </>}
    </Modal>
  );
}
