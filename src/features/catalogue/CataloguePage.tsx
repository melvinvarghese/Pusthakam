import { useTranslation } from "@/lib/translations";
import { useEffect, useState } from "react";
import { SavedSearches } from "./SavedSearches";
import { BookCover } from "./BookCover";
import { Link, useNavigate } from "react-router-dom";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { CallNumberLabel } from "@/components/CallNumberLabel";
import { PageHeader } from "@/components/PageHeader";
import { Pager } from "@/components/Pager";
import { Button } from "@/components/ui/Button";
import { Chip } from "@/components/ui/Chip";
import { Panel } from "@/components/ui/Panel";
import { Modal } from "@/components/ui/Modal";
import { ApiError, catalogueApi, toApiError } from "@/lib/api";
import type { SearchQuery } from "@/lib/api/types";
import { announce } from "@/lib/announcer";
import { can, useSession } from "@/lib/session";
import { ExportDialog } from "./ExportDialog";
import { ImportDialog } from "./ImportDialog";
import { RecordFormDialog } from "./RecordFormDialog";
import { useConfig } from "./shared";

const PAGE_SIZE = 25;

export function CataloguePage() {
  const { t } = useTranslation();
  const { session } = useSession();
  const canWrite = can(session, "catalogue.write");
  const navigate = useNavigate();
  const config = useConfig();
  const [text, setText] = useState("");
  const [debounced, setDebounced] = useState("");
  useEffect(() => {
    const timer = setTimeout(() => setDebounced(text), 250);
    return () => clearTimeout(timer);
  }, [text]);
  const [collectionId, setCollectionId] = useState<number | null>(null);
  const [onlyAvailable, setOnlyAvailable] = useState(false);
  const [includeArchived, setIncludeArchived] = useState(false);
  const [sort, setSort] = useState<NonNullable<SearchQuery["sort"]>>("relevance");
  const [page, setPage] = useState(1);
  const [searchRun, setSearchRun] = useState(0);
  const [adding, setAdding] = useState(false);
  const [exporting, setExporting] = useState(false);
  const [importing, setImporting] = useState(false);
  const [selected, setSelected] = useState<number[]>([]);
  const [confirmArchive, setConfirmArchive] = useState<{ id: number; title: string }[] | null>(null);
  const client = useQueryClient();
  const archive = useMutation({
    mutationFn: (ids: number[]) => catalogueApi.archiveRecords(ids),
    onSuccess: async () => {
      setSelected([]);
      setConfirmArchive(null);
      await client.invalidateQueries({ queryKey: ["catalogue"] });
    },
  });

  useEffect(() => setPage(1), [debounced, collectionId, onlyAvailable, includeArchived, sort]);

  const query: SearchQuery = { text: debounced, collectionId, onlyAvailable, includeArchived, sort, page, pageSize: PAGE_SIZE };
  const result = useQuery({
    queryKey: ["catalogue", "search", query, searchRun],
    queryFn: () => catalogueApi.search(query),
    retry: false,
  });

  useEffect(() => {
    if (result.data) {
      announce(t(result.data.total === 1 ? "Found {count} catalogue record." : "Found {count} catalogue records.", { count: result.data.total }), "polite");
    }
  }, [result.data, t]);

  const rows = result.data?.rows ?? [];
  const selectedRows = rows.filter((row) => !row.archived && selected.includes(row.id));
  const collections = (config.data?.collections ?? []).filter((c) => !c.archived);

  return (
    <div className="mx-auto max-w-[1100px] space-y-5 p-6">
      <PageHeader
        title={t("Catalogue")}
        description={t("Search every title, author, subject, ISBN, accession number or barcode.")}
        actions={(
          <>
            <Button disabled={selected.length === 0} onClick={() => setExporting(true)}>{t("Export selected ({count})", { count: selected.length })}</Button>
            {canWrite && <Button onClick={() => setImporting(true)}>{t("Import")}</Button>}
            {canWrite && <Button variant="primary" onClick={() => setAdding(true)}>{t("Add record")}</Button>}
          </>
        )}
      />

      <Panel aria-label={t("Catalogue search")}>

        <div className="flex flex-wrap items-center gap-3 border-b border-line p-3">
          <input
            type="search"
            aria-label={t("Search the catalogue")}
            placeholder={t("Title, author, subject, ISBN, accession number or barcode")}
            value={text}
            onChange={(e) => setText(e.target.value)}
            onKeyDown={(e) => {
              // A scanned barcode ends with Enter: go straight to the record when it is the only match.
              if (e.key === "Enter" && result.data?.total === 1 && result.data.rows[0]) {
                navigate(`/catalogue/${result.data.rows[0].id}`);
              }
            }}
            autoFocus={typeof window.matchMedia === "function" && window.matchMedia("(min-width: 1024px)").matches}
            className="h-9 min-w-0 w-full sm:min-w-72 flex-1 rounded-md border border-line-strong bg-surface px-3 text-[13px] placeholder:text-muted"
          />
          <select
            aria-label={t("Collection")}
            value={collectionId ?? ""}
            onChange={(e) => setCollectionId(e.target.value ? Number(e.target.value) : null)}
            className="h-9 rounded-md border border-line-strong bg-surface px-2 text-[13px]"
          >
            <option value="">{t("All collections")}</option>
            {collections.map((c) => <option key={c.id} value={c.id}>{c.name}</option>)}
          </select>
          <select
            aria-label={t("Sort by")}
            value={sort}
            onChange={(e) => setSort(e.target.value as typeof sort)}
            className="h-9 rounded-md border border-line-strong bg-surface px-2 text-[13px]"
          >
            <option value="relevance">{t("Best match")}</option>
            <option value="title">{t("Title")}</option>
            <option value="year">{t("Newest publication")}</option>
            <option value="recent">{t("Recently added")}</option>
          </select>
          <label className="flex items-center gap-1.5 text-[13px]">
            <input type="checkbox" checked={onlyAvailable} onChange={(e) => setOnlyAvailable(e.target.checked)} className="size-4" />
            {t("Available now")}</label>
          <label className="flex items-center gap-1.5 text-[13px]">
            <input type="checkbox" checked={includeArchived} onChange={(e) => setIncludeArchived(e.target.checked)} className="size-4" />
            {t("Include archived")}</label>
        </div>

        <SavedSearches key={session.userId} userId={session.userId}
          filter={{ text, collectionId, onlyAvailable, includeArchived, sort }}
          onApply={(filter) => {
            setText(filter.text ?? "");
            setDebounced(filter.text ?? "");
            setCollectionId(filter.collectionId ?? null);
            setOnlyAvailable(filter.onlyAvailable ?? false);
            setIncludeArchived(filter.includeArchived ?? false);
            setSort(filter.sort ?? "relevance");
            setPage(1);
            setSearchRun((run) => run + 1);
          }} />

        {result.isError && (
          <p role="alert" className="px-4 py-6 text-danger">
            {result.error instanceof ApiError ? result.error.message : t("Could not search the catalogue.")}
          </p>
        )}
        {result.isFetching && <p role="status" className="px-4 py-6 text-muted">{t("Searching…")}</p>}

        {canWrite && selectedRows.length > 0 && (
          <div className="px-4 py-2">
            <Button disabled={result.isFetching || archive.isPending} onClick={() => { archive.reset(); setConfirmArchive(selectedRows); }}>
              {t("Archive selected on this page ({count})", { count: selectedRows.length })}
            </Button>
          </div>
        )}
        {result.data && (
          rows.length === 0 ? (
            <p className="px-4 py-10 text-center text-muted">
              {debounced.trim() || collectionId || onlyAvailable
                ? t("No records match. Try fewer words or clear a filter.")
                : canWrite ? t("The catalogue is empty. Add your first record, or import your existing catalogue.") : t("The catalogue is empty.")}
            </p>
          ) : (
            <table role="table" className="staff-card-table w-full table-fixed border-collapse">
              <thead>
                <tr className="h-8 text-left text-[11px] font-semibold uppercase tracking-wide text-muted">
                  {<th scope="col" className="w-12 border-b border-line"><span className="sr-only">{t("Select records")}</span></th>}
                  <th className="border-b border-line px-4">{t("Title and author")}</th>
                  <th className="w-40 border-b border-line px-4">{t("Publisher")}</th>
                  <th className="w-24 border-b border-line px-4">{t("Class")}</th>
                  <th className="w-32 border-b border-line px-4">{t("Copies")}</th>
                </tr>
              </thead>
              <tbody>
                {rows.map((r) => (
                  <tr key={r.id} className={r.archived ? "text-muted" : "even:bg-band"}>
                    {<td className="border-b border-line px-4">
                      <input type="checkbox" aria-label={t("Select {title}", { title: r.title })} disabled={result.isFetching || archive.isPending || (!selected.includes(r.id) && selected.length >= 100)}
                        checked={selected.includes(r.id)} onChange={(event) => {
                          const checked = event.target.checked;
                          setSelected((ids) => checked ? [...ids, r.id] : ids.filter((id) => id !== r.id));
                        }} />
                    </td>}
                    <td className="border-b border-line px-4 py-2.5">
                      <div className="flex items-start gap-3">
                      <BookCover key={`${r.id}:${r.isbn13}`} id={r.id} title={r.title} isbn={r.isbn13} editable={canWrite && !r.archived} compact />
                      <div className="min-w-0">
                      <Link to={`/catalogue/${r.id}`} className="font-semibold hover:underline">{r.title}</Link>
                      {r.subtitle && <span className="text-muted">: {r.subtitle}</span>}
                      <span className="block text-xs text-muted">{r.authors || t("No author")}{r.isbn13 ? ` · ISBN ${r.isbn13}` : ""}</span>
                      </div>
                      </div>
                    </td>
                    <td data-mobile-label={t("Publisher")} className="border-b border-line px-4 py-2.5">
                      {r.publisher ?? <span className="text-muted">{t("Not set")}</span>}
                      {r.publicationYear && <span className="block text-xs text-muted">{r.publicationYear}</span>}
                    </td>
                    <td data-mobile-label={t("Class")} className="border-b border-line px-4 py-2.5">
                      {r.classificationNumber ? <CallNumberLabel callNumber={r.classificationNumber}
                        schemeCode={config.data?.classificationSchemes.find((scheme) => scheme.id === r.classificationSchemeId)?.code} />
                        : <span className="text-muted">{t("None")}</span>}
                    </td>
                    <td data-mobile-label={t("Copies")} className="border-b border-line px-4 py-2.5">
                      {r.archived ? <Chip>{t("Archived")}</Chip> : r.totalCopies === 0 ? <span className="text-muted">{t("No copies")}</span> : t("{available} of {total} available", { available: r.availableCopies, total: r.totalCopies })}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )
        )}
        {result.data && <Pager page={result.data.page} pageSize={result.data.pageSize} total={result.data.total} onPage={setPage} />}
      </Panel>

      {exporting && <ExportDialog ids={selected} onClose={() => setExporting(false)} />}
      {importing && <ImportDialog onClose={() => setImporting(false)} />}
      {canWrite && confirmArchive && (
        <Modal title={t("Archive selected catalogue records")} onClose={() => { if (!archive.isPending) setConfirmArchive(null); }}
          footer={<>
            <Button disabled={archive.isPending} onClick={() => setConfirmArchive(null)}>{t("Cancel")}</Button>
            <Button variant="primary" disabled={archive.isPending}
              onClick={() => archive.mutate(confirmArchive.map((row) => row.id))}>{t("Archive records")}</Button>
          </>}>
          <ul className="list-disc pl-5">{confirmArchive.map((row) => <li key={row.id}>{row.title}</li>)}</ul>
          <p className="text-sm">{t("Copies and loan history will be retained.")}</p>
          {archive.isError && <p role="alert" className="text-danger">{toApiError(archive.error).message}</p>}
        </Modal>
      )}
      {adding && <RecordFormDialog onClose={() => setAdding(false)} onSaved={(id) => navigate(`/catalogue/${id}`)} />}
    </div>
  );
}
