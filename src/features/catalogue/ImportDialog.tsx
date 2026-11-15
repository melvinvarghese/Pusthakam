import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Button } from "@/components/ui/Button";
import { CheckField, SelectField } from "@/components/ui/Field";
import { Modal } from "@/components/ui/Modal";
import { ApiError, importApi } from "@/lib/api";
import type { ImportFormat, ImportOptions, ImportPreview, ImportProfile, ImportReport } from "@/lib/api/types";
import { useConfig } from "./shared";

const SOURCES = [
  { id: "koha", label: "Koha (MARC file with items)", format: "marc", profile: "koha" },
  { id: "marc", label: "Another library system (MARC file, titles only)", format: "marc", profile: "generic" },
  { id: "csv", label: "Spreadsheet (CSV file)", format: "csv", profile: "generic" },
] as const;

const FIELD_LABELS: Record<string, string> = {
  title: "Title", subtitle: "Subtitle", author: "Author", additionalAuthors: "Other authors", publisher: "Publisher",
  place: "Place of publication", year: "Year", isbn: "ISBN", edition: "Edition", language: "Language", pagination: "Pages",
  series: "Series", subjects: "Subjects", keywords: "Keywords", classification: "Classification number", callNumber: "Call number",
  barcode: "Barcode", accession: "Accession number", price: "Price", location: "Location", shelf: "Shelf", collection: "Collection",
  category: "Item category", dateAcquired: "Date acquired", vendor: "Vendor or donor", status: "Status", copies: "Number of copies",
  notes: "Notes", abstract: "Abstract",
};

/** File bytes to base64 without building one giant string argument (large MARC files are tens of megabytes). */
export async function fileToBase64(file: File): Promise<string> {
  const bytes = new Uint8Array(await file.arrayBuffer());
  let binary = "";
  for (let i = 0; i < bytes.length; i += 0x8000) binary += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  return btoa(binary);
}

function errMsg(e: unknown, fallback: string) {
  return e instanceof ApiError ? e.message : fallback;
}

function Stat({ label, value }: { label: string; value: number }) {
  return (
    <div className="rounded-md border border-line p-3">
      <div className="text-xl font-bold leading-7">{value}</div>
      <div className="text-xs text-muted">{label}</div>
    </div>
  );
}

/** Import wizard: choose a file, check the preview (nothing is saved yet), confirm, read the result. */
export function ImportDialog({ onClose }: { onClose: () => void }) {
  const qc = useQueryClient();
  const config = useConfig();
  const [source, setSource] = useState<(typeof SOURCES)[number]["id"]>("koha");
  const [file, setFile] = useState<File | null>(null);
  const [data, setData] = useState<string | null>(null);
  const [preview, setPreview] = useState<ImportPreview | null>(null);
  const [report, setReport] = useState<ImportReport | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [opts, setOpts] = useState<ImportOptions>({ duplicatePolicy: "skip", createMissing: true, defaultItemCategoryId: null, barcodeAsAccession: true });
  const categories = (config.data?.itemCategories ?? []).filter((c) => !c.archived);

  const chosen = SOURCES.find((s) => s.id === source)!;

  const runPreview = useMutation({
    mutationFn: async (mapping: Record<string, string> | null) => {
      if (!file) throw new ApiError({ code: "VALIDATION", message: "Choose a file first." });
      const base64 = data ?? (await fileToBase64(file));
      setData(base64);
      return importApi.preview({ fileName: file.name, dataBase64: base64, format: chosen.format as ImportFormat, profile: chosen.profile as ImportProfile, mapping });
    },
    onSuccess: (p) => {
      setPreview(p);
      setError(null);
      setOpts((o) => ({ ...o, defaultItemCategoryId: o.defaultItemCategoryId ?? categories[0]?.id ?? null }));
    },
    onError: (e) => setError(errMsg(e, "Could not read the file.")),
  });

  const commit = useMutation({
    mutationFn: () => importApi.commit(preview!.token, opts),
    onSuccess: async (r) => {
      setReport(r);
      setError(null);
      await qc.invalidateQueries({ queryKey: ["catalogue"] });
      await qc.invalidateQueries({ queryKey: ["config"] });
    },
    onError: (e) => setError(errMsg(e, "The import could not be completed.")),
  });

  const pick = (f: File | null) => {
    setFile(f);
    setData(null);
    setPreview(null);
    setError(null);
    if (f && /\.csv$/i.test(f.name)) setSource("csv");
    else if (f && /\.(mrc|marc|mrd)$/i.test(f.name) && source === "csv") setSource("koha");
  };

  const cancelAndClose = () => {
    if (preview && !report) void importApi.cancel().catch(() => {});
    onClose();
  };

  // ----- step 3: result -----
  if (report) {
    return (
      <Modal title="Import finished" onClose={onClose} wide footer={<Button variant="primary" onClick={onClose}>Done</Button>}>
        <div className="grid grid-cols-4 gap-3">
          <Stat label="Records added" value={report.createdRecords} />
          <Stat label="Copies added" value={report.copiesAdded} />
          <Stat label="Added to existing records" value={report.mergedRecords} />
          <Stat label="Skipped or failed" value={report.skippedRecords + report.failedRecords} />
        </div>
        <ul className="list-disc pl-5 text-muted">
          {report.skippedRecords > 0 && <li>{report.skippedRecords} records were already in the catalogue and were left alone.</li>}
          {report.copiesSkipped > 0 && <li>{report.copiesSkipped} copies were skipped (for example, their barcode was already in use).</li>}
          {report.createdLookups > 0 && <li>{report.createdLookups} new locations, shelves, collections or item categories were created. You can rename them in Library Settings.</li>}
        </ul>
        {report.problems.length > 0 && (
          <div>
            <h3 className="mb-1 text-[13px] font-semibold">Records that could not be imported</h3>
            <ul className="max-h-40 overflow-auto rounded-md border border-line p-2 text-xs">
              {report.problems.map((p, i) => <li key={i}>Row {p.row}: {p.message}</li>)}
            </ul>
          </div>
        )}
      </Modal>
    );
  }

  // ----- step 1: choose a file -----
  if (!preview) {
    return (
      <Modal
        title="Import records"
        onClose={cancelAndClose}
        wide
        footer={
          <>
            <Button onClick={cancelAndClose}>Cancel</Button>
            <Button variant="primary" disabled={!file || runPreview.isPending} onClick={() => runPreview.mutate(null)}>
              {runPreview.isPending ? "Reading…" : "Preview"}
            </Button>
          </>
        }
      >
        <SelectField label="Where is the data from?" value={source} onChange={(e) => setSource(e.target.value as typeof source)}>
          {SOURCES.map((s) => <option key={s.id} value={s.id}>{s.label}</option>)}
        </SelectField>
        {source === "koha" && (
          <p className="text-xs text-muted">
            In Koha, export your catalogue as a MARC file (ISO 2709, .mrc) <strong>including items</strong>. Pusthakam reads each item
            (field 952) as a copy with its barcode, call number, location, collection, price and lost/withdrawn status. Nothing is
            saved until you have checked the preview.
          </p>
        )}
        {source === "csv" && <p className="text-xs text-muted">The first row must contain column names. Pusthakam recognises common names (Title, Author, ISBN, Barcode…) and lets you fix the rest.</p>}
        <div className="space-y-1">
          <label htmlFor="import-file" className="block text-[13px] font-medium">File</label>
          <input
            id="import-file"
            type="file"
            accept=".mrc,.marc,.mrd,.csv,.txt"
            onChange={(e) => pick(e.target.files?.[0] ?? null)}
            className="block w-full text-[13px] file:mr-3 file:h-8 file:rounded-md file:border file:border-line-strong file:bg-surface file:px-3"
          />
        </div>
        {error && <p role="alert" className="text-xs text-danger">{error}</p>}
      </Modal>
    );
  }

  // ----- step 2: preview and options -----
  const t = preview.totals;
  const importable = t.records - t.withErrors;
  const needsCategory = preview.sample.some((r) => r.copies.some((c) => !c.itemCategory)) || preview.format === "csv";
  return (
    <Modal
      title={`Check before importing: ${preview.fileName}`}
      onClose={cancelAndClose}
      wide
      footer={
        <>
          <Button onClick={() => setPreview(null)}>Back</Button>
          <Button variant="primary" disabled={commit.isPending || importable === 0} onClick={() => commit.mutate()}>
            {commit.isPending ? "Importing…" : `Import ${importable} record${importable === 1 ? "" : "s"}`}
          </Button>
        </>
      }
    >
      <div className="grid grid-cols-4 gap-3">
        <Stat label="Records in the file" value={t.records} />
        <Stat label="Copies" value={t.copies} />
        <Stat label="Already in the catalogue" value={t.duplicates} />
        <Stat label="Cannot be imported" value={t.withErrors} />
      </div>

      {preview.parseErrors.length > 0 && (
        <div role="alert" className="rounded-md border border-danger p-3 text-danger">
          <p className="font-semibold">{preview.parseErrors.length} records in the file could not be read and will be left out:</p>
          <ul className="list-disc pl-5 text-xs">{preview.parseErrors.slice(0, 8).map((e, i) => <li key={i}>{e}</li>)}</ul>
        </div>
      )}

      {preview.format === "csv" && preview.headers && preview.mappingFields && (
        <div>
          <h3 className="mb-1 text-[13px] font-semibold">Columns</h3>
          <div className="grid max-h-48 grid-cols-3 gap-2 overflow-auto">
            {preview.mappingFields.map((field) => (
              <label key={field} className="flex flex-col gap-0.5 text-xs">
                {FIELD_LABELS[field] ?? field}
                <select
                  aria-label={`Column for ${FIELD_LABELS[field] ?? field}`}
                  value={preview.mapping?.[field] ?? ""}
                  onChange={(e) => {
                    const next = { ...(preview.mapping ?? {}) };
                    if (e.target.value) next[field] = e.target.value;
                    else delete next[field];
                    runPreview.mutate(next);
                  }}
                  className="h-8 rounded-md border border-line-strong bg-surface px-2 text-[13px]"
                >
                  <option value="">Not in the file</option>
                  {preview.headers!.map((h) => <option key={h} value={h}>{h}</option>)}
                </select>
              </label>
            ))}
          </div>
        </div>
      )}

      {(preview.newLookups.locations.length + preview.newLookups.collections.length + preview.newLookups.itemCategories.length) > 0 && (
        <p className="text-xs text-muted">
          New in your library: {[
            preview.newLookups.locations.length ? `locations ${preview.newLookups.locations.join(", ")}` : "",
            preview.newLookups.collections.length ? `collections ${preview.newLookups.collections.join(", ")}` : "",
            preview.newLookups.itemCategories.length ? `item categories ${preview.newLookups.itemCategories.join(", ")}` : "",
          ].filter(Boolean).join("; ")}.
        </p>
      )}

      <div className="max-h-56 overflow-auto rounded-md border border-line">
        <table className="w-full table-fixed border-collapse text-xs">
          <thead>
            <tr className="h-7 text-left font-semibold uppercase tracking-wide text-muted">
              <th className="w-12 border-b border-line px-2">Row</th>
              <th className="border-b border-line px-2">Title</th>
              <th className="w-14 border-b border-line px-2">Copies</th>
              <th className="w-2/5 border-b border-line px-2">Notes</th>
            </tr>
          </thead>
          <tbody>
            {preview.sample.map((r) => (
              <tr key={r.row} className="align-top">
                <td className="border-b border-line px-2 py-1">{r.row}</td>
                <td className="border-b border-line px-2 py-1">
                  <span className="font-medium">{r.title || "No title"}</span>
                  <span className="block text-muted">{r.contributors[0]?.name}</span>
                </td>
                <td className="border-b border-line px-2 py-1">{r.copies.length}</td>
                <td className="border-b border-line px-2 py-1">
                  {r.errors.map((m, i) => <span key={`e${i}`} className="block text-danger">{m}</span>)}
                  {r.warnings.map((m, i) => <span key={`w${i}`} className="block text-muted">{m}</span>)}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {t.records > preview.sample.length && <p className="text-xs text-muted">Showing {preview.sample.length} of {t.records} records: the first ones plus those with notes.</p>}

      <div className="grid grid-cols-2 gap-3">
        <SelectField
          label="If a title is already in the catalogue (same ISBN)"
          value={opts.duplicatePolicy}
          onChange={(e) => setOpts({ ...opts, duplicatePolicy: e.target.value as ImportOptions["duplicatePolicy"] })}
        >
          <option value="skip">Leave it alone</option>
          <option value="merge">Add the file's new copies to it</option>
          <option value="new">Create a second record</option>
        </SelectField>
        {needsCategory && (
          <SelectField
            label="Item category for copies that have none"
            value={String(opts.defaultItemCategoryId ?? "")}
            onChange={(e) => setOpts({ ...opts, defaultItemCategoryId: e.target.value ? Number(e.target.value) : null })}
          >
            <option value="">Choose…</option>
            {categories.map((c) => <option key={c.id} value={c.id}>{c.name}</option>)}
          </SelectField>
        )}
      </div>
      <CheckField label="Create the locations, collections and categories named in the file" checked={opts.createMissing} onChange={(e) => setOpts({ ...opts, createMissing: e.target.checked })} />
      <CheckField
        label="Use each barcode as the accession number"
        hint="Keeps the identifiers your library already uses. Without a barcode, numbers follow your accession format."
        checked={opts.barcodeAsAccession}
        onChange={(e) => setOpts({ ...opts, barcodeAsAccession: e.target.checked })}
      />
      {error && <p role="alert" className="text-xs text-danger">{error}</p>}
    </Modal>
  );
}
