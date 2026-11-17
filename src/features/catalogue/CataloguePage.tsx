import { useEffect, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { keepPreviousData, useQuery } from "@tanstack/react-query";
import { CallNumberLabel } from "@/components/CallNumberLabel";
import { PageHeader } from "@/components/PageHeader";
import { Pager } from "@/components/Pager";
import { Button } from "@/components/ui/Button";
import { Chip } from "@/components/ui/Chip";
import { Panel } from "@/components/ui/Panel";
import { ApiError, catalogueApi } from "@/lib/api";
import type { SearchQuery } from "@/lib/api/types";
import { can, useSession } from "@/lib/session";
import { ImportDialog } from "./ImportDialog";
import { RecordFormDialog } from "./RecordFormDialog";
import { useConfig, useDebounced } from "./shared";

const PAGE_SIZE = 25;

export function CataloguePage() {
  const { session } = useSession();
  const canWrite = can(session, "catalogue.write");
  const navigate = useNavigate();
  const config = useConfig();
  const [text, setText] = useState("");
  const debounced = useDebounced(text, 250);
  const [collectionId, setCollectionId] = useState<number | null>(null);
  const [onlyAvailable, setOnlyAvailable] = useState(false);
  const [includeArchived, setIncludeArchived] = useState(false);
  const [sort, setSort] = useState<NonNullable<SearchQuery["sort"]>>("relevance");
  const [page, setPage] = useState(1);
  const [adding, setAdding] = useState(false);
  const [importing, setImporting] = useState(false);

  useEffect(() => setPage(1), [debounced, collectionId, onlyAvailable, includeArchived, sort]);

  const query: SearchQuery = { text: debounced, collectionId, onlyAvailable, includeArchived, sort, page, pageSize: PAGE_SIZE };
  const result = useQuery({
    queryKey: ["catalogue", "search", query],
    queryFn: () => catalogueApi.search(query),
    placeholderData: keepPreviousData,
    retry: false,
  });

  const rows = result.data?.rows ?? [];
  const collections = (config.data?.collections ?? []).filter((c) => !c.archived);

  return (
    <div className="mx-auto max-w-[1100px] space-y-5 p-6">
      <PageHeader
        title="Catalogue"
        description="Search every title, author, subject, ISBN, accession number or barcode."
        actions={canWrite && (
          <>
            <Button onClick={() => setImporting(true)}>Import</Button>
            <Button variant="primary" onClick={() => setAdding(true)}>Add record</Button>
          </>
        )}
      />

      <Panel aria-label="Catalogue search">
        <div className="flex flex-wrap items-center gap-3 border-b border-line p-3">
          <input
            type="search"
            aria-label="Search the catalogue"
            placeholder="Title, author, subject, ISBN, accession number or barcode"
            value={text}
            onChange={(e) => setText(e.target.value)}
            onKeyDown={(e) => {
              // A scanned barcode ends with Enter: go straight to the record when it is the only match.
              if (e.key === "Enter" && result.data?.total === 1 && result.data.rows[0]) {
                navigate(`/catalogue/${result.data.rows[0].id}`);
              }
            }}
            autoFocus
            className="h-9 min-w-72 flex-1 rounded-md border border-line-strong bg-surface px-3 text-[13px] placeholder:text-muted"
          />
          <select
            aria-label="Collection"
            value={collectionId ?? ""}
            onChange={(e) => setCollectionId(e.target.value ? Number(e.target.value) : null)}
            className="h-9 rounded-md border border-line-strong bg-surface px-2 text-[13px]"
          >
            <option value="">All collections</option>
            {collections.map((c) => <option key={c.id} value={c.id}>{c.name}</option>)}
          </select>
          <select
            aria-label="Sort by"
            value={sort}
            onChange={(e) => setSort(e.target.value as typeof sort)}
            className="h-9 rounded-md border border-line-strong bg-surface px-2 text-[13px]"
          >
            <option value="relevance">Best match</option>
            <option value="title">Title</option>
            <option value="year">Newest publication</option>
            <option value="recent">Recently added</option>
          </select>
          <label className="flex items-center gap-1.5 text-[13px]">
            <input type="checkbox" checked={onlyAvailable} onChange={(e) => setOnlyAvailable(e.target.checked)} className="size-4" />
            Available now
          </label>
          <label className="flex items-center gap-1.5 text-[13px]">
            <input type="checkbox" checked={includeArchived} onChange={(e) => setIncludeArchived(e.target.checked)} className="size-4" />
            Include archived
          </label>
        </div>

        {result.isError && (
          <p role="alert" className="px-4 py-6 text-danger">
            {result.error instanceof ApiError ? result.error.message : "Could not search the catalogue."}
          </p>
        )}
        {result.isPending && <p role="status" className="px-4 py-6 text-muted">Searching…</p>}

        {result.data && (
          rows.length === 0 ? (
            <p className="px-4 py-10 text-center text-muted">
              {debounced.trim() || collectionId || onlyAvailable
                ? "No records match. Try fewer words or clear a filter."
                : canWrite ? "The catalogue is empty. Add your first record, or import your existing catalogue." : "The catalogue is empty."}
            </p>
          ) : (
            <table className="w-full table-fixed border-collapse">
              <thead>
                <tr className="h-8 text-left text-[11px] font-semibold uppercase tracking-wide text-muted">
                  <th className="border-b border-line px-4">Title and author</th>
                  <th className="w-40 border-b border-line px-4">Publisher</th>
                  <th className="w-24 border-b border-line px-4">Class</th>
                  <th className="w-32 border-b border-line px-4">Copies</th>
                </tr>
              </thead>
              <tbody>
                {rows.map((r) => (
                  <tr key={r.id} className={r.archived ? "text-muted" : "even:bg-band"}>
                    <td className="border-b border-line px-4 py-2.5">
                      <Link to={`/catalogue/${r.id}`} className="font-semibold hover:underline">{r.title}</Link>
                      {r.subtitle && <span className="text-muted">: {r.subtitle}</span>}
                      <span className="block text-xs text-muted">{r.authors || "No author"}{r.isbn13 ? ` · ISBN ${r.isbn13}` : ""}</span>
                    </td>
                    <td className="border-b border-line px-4 py-2.5">
                      {r.publisher ?? <span className="text-muted">Not set</span>}
                      {r.publicationYear && <span className="block text-xs text-muted">{r.publicationYear}</span>}
                    </td>
                    <td className="border-b border-line px-4 py-2.5">
                      {r.classificationNumber ? <CallNumberLabel callNumber={r.classificationNumber}
                        schemeCode={config.data?.classificationSchemes.find((scheme) => scheme.id === r.classificationSchemeId)?.code} />
                        : <span className="text-muted">None</span>}
                    </td>
                    <td className="border-b border-line px-4 py-2.5">
                      {r.archived ? <Chip>Archived</Chip> : r.totalCopies === 0 ? <span className="text-muted">No copies</span> : `${r.availableCopies} of ${r.totalCopies} available`}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )
        )}
        {result.data && <Pager page={result.data.page} pageSize={result.data.pageSize} total={result.data.total} onPage={setPage} />}
      </Panel>

      {importing && <ImportDialog onClose={() => setImporting(false)} />}
      {adding && <RecordFormDialog onClose={() => setAdding(false)} onSaved={(id) => navigate(`/catalogue/${id}`)} />}
    </div>
  );
}
