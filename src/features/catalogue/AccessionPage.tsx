import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { keepPreviousData, useQuery } from "@tanstack/react-query";
import { PageHeader } from "@/components/PageHeader";
import { Pager } from "@/components/Pager";
import { Panel } from "@/components/ui/Panel";
import { ApiError, catalogueApi } from "@/lib/api";
import type { CopyQuery, CopyStatus } from "@/lib/api/types";
import { formatMinor } from "@/lib/money";
import { StatusChip, STATUS_LABELS, useConfig, useDebounced } from "./shared";

const PAGE_SIZE = 25;

/** Every physical copy in the library, searchable by accession number, barcode, title or call number. */
export function AccessionPage() {
  const config = useConfig();
  const [text, setText] = useState("");
  const debounced = useDebounced(text, 250);
  const [status, setStatus] = useState<CopyStatus | "">("");
  const [collectionId, setCollectionId] = useState<number | null>(null);
  const [page, setPage] = useState(1);
  useEffect(() => setPage(1), [debounced, status, collectionId]);

  const query: CopyQuery = { text: debounced, status: status || null, collectionId, page, pageSize: PAGE_SIZE };
  const result = useQuery({ queryKey: ["catalogue", "copies", query], queryFn: () => catalogueApi.listCopies(query), placeholderData: keepPreviousData, retry: false });
  const currency = config.data?.library?.currency ?? "INR";

  return (
    <div className="mx-auto max-w-[1100px] space-y-5 p-6">
      <PageHeader title="Accession Register" description="Every copy in the library, in accession order." />
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
        </div>
        {result.isError && <p role="alert" className="px-4 py-6 text-danger">{result.error instanceof ApiError ? result.error.message : "Could not load the register."}</p>}
        {result.isPending && <p role="status" className="px-4 py-6 text-muted">Loading…</p>}
        {result.data && (result.data.rows.length === 0 ? (
          <p className="px-4 py-10 text-center text-muted">{debounced.trim() || status || collectionId ? "No copies match." : "No copies yet. Add copies from a catalogue record."}</p>
        ) : (
          <table className="w-full table-fixed border-collapse">
            <thead>
              <tr className="h-8 text-left text-[11px] font-semibold uppercase tracking-wide text-muted">
                <th className="w-32 border-b border-line px-4">Accession</th>
                <th className="border-b border-line px-4">Title</th>
                <th className="w-36 border-b border-line px-4">Call number</th>
                <th className="w-44 border-b border-line px-4">Where</th>
                <th className="w-24 border-b border-line px-4">Price</th>
                <th className="w-28 border-b border-line px-4">Status</th>
              </tr>
            </thead>
            <tbody>
              {result.data.rows.map((c) => (
                <tr key={c.id} className="even:bg-band">
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
    </div>
  );
}
