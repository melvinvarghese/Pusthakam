import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { keepPreviousData, useMutation, useQuery } from "@tanstack/react-query";
import { PageHeader } from "@/components/PageHeader";
import { Pager } from "@/components/Pager";
import { Button } from "@/components/ui/Button";
import { Modal } from "@/components/ui/Modal";
import { Panel } from "@/components/ui/Panel";
import { ApiError, catalogueApi, toApiError } from "@/lib/api";
import type { CopyQuery, CopyRow, CopyStatus } from "@/lib/api/types";
import { formatMinor } from "@/lib/money";
import { localDate } from "@/lib/dates";
import { downloadCsvFile } from "@/lib/files";
import { LabelSheetDialog } from "./LabelSheetDialog";
import { StatusChip, STATUS_LABELS, useConfig, useDebounced } from "./shared";

const PAGE_SIZE = 25;

function InventorySheet({ rows, page, library, onClose }: { rows: CopyRow[]; page: number; library?: string; onClose: () => void }) {
  return (
    <Modal title="Inventory sheet" onClose={onClose} wide footer={<>
      <Button onClick={onClose}>Close</Button>
      <Button variant="primary" onClick={() => window.print()}>Print inventory sheet</Button>
    </>}>
      <section className="print-area bg-white p-3 text-black">
        <h3 className="font-semibold">{library ?? "Library"} inventory sheet</h3>
        <p className="mb-3 text-xs">Accession Register page {page}</p>
        <table className="w-full border-collapse text-left text-xs [&_td]:border [&_td]:border-black/30 [&_td]:p-1 [&_th]:p-1">
          <thead><tr>{["Accession", "Title", "Call number", "Location / shelf", "Status", "Checked"].map((label) => (
            <th key={label} scope="col">{label}</th>
          ))}</tr></thead>
          <tbody>{rows.map((copy) => (
            <tr key={copy.id} className="break-inside-avoid">
              <td className="font-mono">{copy.accessionNo}</td><td>{copy.title}</td><td>{copy.callNumber}</td>
              <td>{[copy.location, copy.shelf].filter(Boolean).join(" / ")}</td><td>{STATUS_LABELS[copy.status]}</td>
              <td aria-label="Mark when checked" />
            </tr>
          ))}</tbody>
        </table>
      </section>
    </Modal>
  );
}

/** Every physical copy in the library, searchable by accession number, barcode, title or call number. */
export function AccessionPage() {
  const config = useConfig();
  const [text, setText] = useState("");
  const debounced = useDebounced(text, 250);
  const [status, setStatus] = useState<CopyStatus | "">("");
  const [collectionId, setCollectionId] = useState<number | null>(null);
  const [page, setPage] = useState(1);
  const [selected, setSelected] = useState(new Map<number, CopyRow>());
  const [previewOpen, setPreviewOpen] = useState(false);
  const [inventory, setInventory] = useState<{ rows: CopyRow[]; page: number } | null>(null);
  useEffect(() => setPage(1), [debounced, status, collectionId]);

  const query: CopyQuery = { text: debounced, status: status || null, collectionId, page, pageSize: PAGE_SIZE };
  const result = useQuery({ queryKey: ["catalogue", "copies", query], queryFn: () => catalogueApi.listCopies(query), placeholderData: keepPreviousData, retry: false });
  const exportCopies = useMutation({
    mutationFn: (filters: CopyQuery) => catalogueApi.exportCopies(filters),
    onSuccess: (csv) => downloadCsvFile("\uFEFF" + csv, `accession-register-${localDate()}.csv`),
  });
  const currency = config.data?.library?.currency ?? "INR";
  const rows = result.data?.rows ?? [];
  const selectedOnPage = rows.filter((copy) => selected.has(copy.id)).length;
  const pageSelected = rows.length > 0 && selectedOnPage === rows.length;

  function selectCopies(copies: CopyRow[], checked: boolean) {
    setSelected((previous) => {
      const next = new Map(previous);
      for (const copy of copies) {
        if (checked) next.set(copy.id, copy);
        else next.delete(copy.id);
      }
      return next;
    });
  }

  return (
    <div className="mx-auto max-w-[1100px] space-y-5 p-6">
      <PageHeader title="Accession Register" description="Every copy in the library, in accession order."
        actions={<>
          {selected.size > 0 && <Button onClick={() => setSelected(new Map())}>Clear selection</Button>}
          <Button disabled={selected.size === 0} onClick={() => setPreviewOpen(true)}>
            {selected.size ? `Preview labels (${selected.size})` : "Preview labels"}
          </Button>
        </>}
      />
      <Panel aria-label="Accession register">
        <div className="flex flex-wrap items-center gap-3 border-b border-line p-3">
          <input
            type="search"
            aria-label="Search copies"
            placeholder="Accession number, barcode, title or call number"
            value={text}
            onChange={(e) => setText(e.target.value)}
            autoFocus
            className="h-9 min-w-72 flex-1 rounded-md border border-line-strong bg-surface px-3 text-[13px] placeholder:text-muted"
          />
          <select aria-label="Status" value={status} onChange={(e) => setStatus(e.target.value as CopyStatus | "")} className="h-9 rounded-md border border-line-strong bg-surface px-2 text-[13px]">
            <option value="">Any status</option>
            {(Object.keys(STATUS_LABELS) as CopyStatus[]).map((s) => <option key={s} value={s}>{STATUS_LABELS[s]}</option>)}
          </select>
          <select aria-label="Collection" value={collectionId ?? ""} onChange={(e) => setCollectionId(e.target.value ? Number(e.target.value) : null)} className="h-9 rounded-md border border-line-strong bg-surface px-2 text-[13px]">
            <option value="">All collections</option>
            {(config.data?.collections ?? []).filter((c) => !c.archived).map((c) => <option key={c.id} value={c.id}>{c.name}</option>)}
          </select>
          <Button disabled={!rows.length || result.isPlaceholderData} onClick={() => {
            if (result.data) setInventory({ rows, page: result.data.page });
          }}>Inventory sheet for this page</Button>
          <Button disabled={!result.data?.total || result.isPlaceholderData || debounced !== text || exportCopies.isPending}
            onClick={() => exportCopies.mutate(query)}>
            {exportCopies.isPending ? "Preparing CSV…" : "Export matching copies"}
          </Button>
        </div>
        {exportCopies.isError && <p role="alert" className="px-4 py-2 text-danger">{toApiError(exportCopies.error).message}</p>}
        {result.isError && <p role="alert" className="px-4 py-6 text-danger">{result.error instanceof ApiError ? result.error.message : "Could not load the register."}</p>}
        {result.isPending && <p role="status" className="px-4 py-6 text-muted">Loading…</p>}
        {result.data && (result.data.rows.length === 0 ? (
          <p className="px-4 py-10 text-center text-muted">{debounced.trim() || status || collectionId ? "No copies match." : "No copies yet. Add copies from a catalogue record."}</p>
        ) : (
          <table className="w-full table-fixed border-collapse">
            <thead>
              <tr className="h-8 text-left text-[11px] font-semibold uppercase tracking-wide text-muted">
                <th className="w-10 border-b border-line text-center">
                  <input type="checkbox" aria-label="Select copies on this page" className="size-4 align-middle"
                    checked={pageSelected} disabled={result.isPlaceholderData}
                    ref={(checkbox) => { if (checkbox) checkbox.indeterminate = selectedOnPage > 0 && !pageSelected; }}
                    onChange={(event) => selectCopies(rows, event.target.checked)} />
                </th>
                <th className="w-32 border-b border-line px-4">Accession</th>
                <th className="border-b border-line px-4">Title</th>
                <th className="w-36 border-b border-line px-4">Call number</th>
                <th className="w-44 border-b border-line px-4">Where</th>
                <th className="w-24 border-b border-line px-4">Price</th>
                <th className="w-28 border-b border-line px-4">Status</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((c) => (
                <tr key={c.id} className="even:bg-band">
                  <td className="border-b border-line text-center">
                    <input type="checkbox" aria-label={`Select copy ${c.accessionNo}`} className="size-4 align-middle"
                      checked={selected.has(c.id)} disabled={result.isPlaceholderData}
                      onChange={(event) => selectCopies([c], event.target.checked)} />
                  </td>
                  <td className="border-b border-line px-4 py-2 font-mono text-xs">{c.accessionNo}</td>
                  <td className="border-b border-line px-4 py-2"><Link to={`/catalogue/${c.recordId}`} className="font-medium hover:underline">{c.title}</Link></td>
                  <td className="border-b border-line px-4 py-2">{c.callNumber ?? <span className="text-muted">Not set</span>}</td>
                  <td className="border-b border-line px-4 py-2">
                    {[c.location, c.shelf].filter(Boolean).join(" · ") || <span className="text-muted">Not set</span>}
                    {c.collection && <span className="block text-xs text-muted">{c.collection}</span>}
                  </td>
                  <td className="border-b border-line px-4 py-2">{c.priceMinor != null ? formatMinor(c.priceMinor, currency) : <span className="text-muted">Not set</span>}</td>
                  <td className="border-b border-line px-4 py-2"><StatusChip status={c.status} /></td>
                </tr>
              ))}
            </tbody>
          </table>
        ))}
        {result.data && <Pager page={result.data.page} pageSize={result.data.pageSize} total={result.data.total} onPage={setPage} />}
      </Panel>
      {previewOpen && <LabelSheetDialog copies={[...selected.values()]} onClose={() => setPreviewOpen(false)} />}
      {inventory && <InventorySheet {...inventory} library={config.data?.library?.name} onClose={() => setInventory(null)} />}
    </div>
  );
}
