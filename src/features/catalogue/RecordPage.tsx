import { useTranslation } from "@/lib/translations";
import { useState } from "react";
import { BookCover } from "./BookCover";
import { Link, useParams } from "react-router-dom";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { PageHeader } from "@/components/PageHeader";
import { Button } from "@/components/ui/Button";
import { Chip } from "@/components/ui/Chip";
import { Panel, PanelHeader } from "@/components/ui/Panel";
import { ApiError, catalogueApi } from "@/lib/api";
import type { CopyRecord } from "@/lib/api/types";
import { formatMinor } from "@/lib/money";
import { can, useSession } from "@/lib/session";
import { AddCopiesDialog, EditCopyDialog, LabelDialog, StatusDialog } from "./CopyDialogs";
import { RecordFormDialog } from "./RecordFormDialog";
import { ROLE_LABELS, StatusChip, useConfig } from "./shared";

type Dialog =
  | { type: "edit" }
  | { type: "add" }
  | { type: "copy"; copy: CopyRecord }
  | { type: "status"; copy: CopyRecord }
  | { type: "label"; copy: CopyRecord }
  | null;

export function RecordPage() {
  const { t } = useTranslation();
  const id = Number(useParams().id);
  const { session } = useSession();
  const canWrite = can(session, "catalogue.write");
  const qc = useQueryClient();
  const config = useConfig();
  const record = useQuery({ queryKey: ["catalogue", "record", id], queryFn: () => catalogueApi.getRecord(id), retry: false });
  const [dialog, setDialog] = useState<Dialog>(null);
  const close = () => setDialog(null);

  const archive = useMutation({
    mutationFn: (archived: boolean) => catalogueApi.archiveRecord(id, archived),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["catalogue"] }),
  });

  if (record.isPending) return <div className="p-6 text-muted" role="status">{t("Loading record…")}</div>;
  if (record.isError || !record.data) {
    return (
      <div className="mx-auto max-w-[1100px] space-y-3 p-6" role="alert">
        <p className="text-danger">{record.error instanceof ApiError ? record.error.message : t("Could not load this record.")}</p>
        <Link to="/catalogue" className="underline">{t("Back to the catalogue")}</Link>
      </div>
    );
  }
  const r = record.data;
  const cfg = config.data;
  const name = (list: { id: number; name: string }[] | undefined, key: number | null) => list?.find((x) => x.id === key)?.name;
  const shelfCode = (key: number | null) => cfg?.shelves.find((s) => s.id === key)?.code;
  const currency = cfg?.library?.currency ?? "INR";
  const scheme = cfg?.classificationSchemes.find((s) => s.id === r.classificationSchemeId);

  const details: [string, string | number | null][] = [
    ["Publisher", [r.publisher, r.publicationPlace].filter(Boolean).join(", ") || null],
    ["Year", r.publicationYear],
    ["Edition", r.edition],
    ["ISBN", r.isbn13 ? `${r.isbn13}${r.isbn10 ? ` (ISBN-10 ${r.isbn10})` : ""}` : null],
    ["Language", r.language],
    ["Pagination", r.pagination],
    ["Physical description", r.physicalDescription],
    ["Series", r.series],
    ["Classification", r.classificationNumber ? `${r.classificationNumber}${scheme ? ` (${scheme.code.toUpperCase()})` : ""}` : null],
    ["Keywords", r.keywords],
  ];

  return (
    <div className="mx-auto max-w-[1100px] space-y-5 p-6">
      <p><Link to="/catalogue" className="text-muted hover:underline">{t("Catalogue")}</Link></p>
      <PageHeader
        title={r.title}
        description={r.subtitle ?? undefined}
        actions={canWrite && (
          <>
            <Button onClick={() => setDialog({ type: "edit" })}>{t("Edit record")}</Button>
            <Button disabled={archive.isPending} onClick={() => archive.mutate(!r.archived)}>{r.archived ? t("Restore") : t("Archive")}</Button>
            {!r.archived && <Button variant="primary" onClick={() => setDialog({ type: "add" })}>{t("Add copies")}</Button>}
          </>
        )}
      />
      {r.archived && <p><Chip>{t("Archived")}</Chip> <span className="text-muted">{t("This record is hidden from search.")}</span></p>}
      {archive.isError && <p role="alert" className="text-danger">{archive.error instanceof ApiError ? archive.error.message : t("Could not update.")}</p>}

      <BookCover key={id} id={id} title={r.title} isbn={r.isbn13 ?? r.isbn10} editable={canWrite && !r.archived} />
      <Panel aria-labelledby="details-h">
        <PanelHeader id="details-h" title={t("Details")} />
        <dl className="grid grid-cols-[11rem_1fr] gap-x-4 gap-y-2 p-4">
          <dt className="text-muted">{t("Contributors")}</dt>
          <dd>{r.contributors.length ? r.contributors.map((c, i) => <span key={i} className="mr-3 inline-block">{c.name}{c.role !== "author" && <span className="text-muted"> ({t(ROLE_LABELS[c.role])})</span>}</span>) : <span className="text-muted">{t("None")}</span>}</dd>
          {details.filter(([, v]) => v !== null && v !== "").map(([k, v]) => (
            <div key={k} className="contents"><dt className="text-muted">{t(k)}</dt><dd>{v}</dd></div>
          ))}
          <dt className="text-muted">{t("Subjects")}</dt>
          <dd>{r.subjects.length ? r.subjects.join(" · ") : <span className="text-muted">{t("None")}</span>}</dd>
          {r.abstract && <><dt className="text-muted">{t("Abstract")}</dt><dd className="max-w-prose">{r.abstract}</dd></>}
          {r.notes && <><dt className="text-muted">{t("Notes")}</dt><dd className="max-w-prose">{r.notes}</dd></>}
        </dl>
      </Panel>

      <Panel aria-labelledby="copies-h">
        <PanelHeader id="copies-h" title={t("Copies ({count})", { count: r.copies.length })} />
        {r.copies.length === 0 ? (
          <p className="px-4 py-8 text-center text-muted">{t("This title has no copies yet.")}{canWrite && !r.archived ? ` ${t("Add copies to put it on the shelf.")}` : ""}</p>
        ) : (
          <table className="w-full table-fixed border-collapse">
            <thead>
              <tr className="h-8 text-left text-[11px] font-semibold uppercase tracking-wide text-muted">
                <th className="w-32 border-b border-line px-4">{t("Accession")}</th>
                <th className="w-36 border-b border-line px-4">{t("Call number")}</th>
                <th className="border-b border-line px-4">{t("Where")}</th>
                <th className="w-28 border-b border-line px-4">{t("Status")}</th>
                <th className="w-52 border-b border-line px-4" />
              </tr>
            </thead>
            <tbody>
              {r.copies.map((c) => (
                <tr key={c.id}>
                  <td className="border-b border-line px-4 py-2 font-mono text-xs">{c.accessionNo}{c.barcode !== c.accessionNo && <span className="block text-muted">{t("Barcode")} {c.barcode}</span>}</td>
                  <td className="border-b border-line px-4 py-2">{c.callNumber ?? <span className="text-muted">{t("Not set")}</span>}</td>
                  <td className="border-b border-line px-4 py-2">
                    {[name(cfg?.locations, c.locationId), shelfCode(c.shelfId)].filter(Boolean).join(" · ") || <span className="text-muted">{t("Not set")}</span>}
                    <span className="block text-xs text-muted">
                      {[name(cfg?.collections, c.collectionId), name(cfg?.itemCategories, c.itemCategoryId), c.priceMinor != null ? formatMinor(c.priceMinor, currency) : null].filter(Boolean).join(" · ")}
                    </span>
                  </td>
                  <td className="border-b border-line px-4 py-2"><StatusChip status={c.status} /></td>
                  <td className="whitespace-nowrap border-b border-line px-4 py-2 text-right">
                    <Button size="sm" variant="ghost" onClick={() => setDialog({ type: "label", copy: c })}>{t("Label")}</Button>
                    {canWrite && cfg && (
                      <>
                        <Button size="sm" variant="ghost" onClick={() => setDialog({ type: "copy", copy: c })}>{t("Edit")}</Button>
                        <Button size="sm" variant="ghost" onClick={() => setDialog({ type: "status", copy: c })}>{t("Status")}</Button>
                      </>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </Panel>

      {dialog?.type === "edit" && <RecordFormDialog record={r} onClose={close} onSaved={() => {}} />}
      {dialog?.type === "add" && cfg && <AddCopiesDialog record={r} config={cfg} onClose={close} />}
      {dialog?.type === "copy" && cfg && <EditCopyDialog copy={dialog.copy} config={cfg} onClose={close} />}
      {dialog?.type === "status" && <StatusDialog copy={dialog.copy} onClose={close} />}
      {dialog?.type === "label" && <LabelDialog copy={dialog.copy} title={r.title} schemeCode={scheme?.code} onClose={close} />}
    </div>
  );
}
