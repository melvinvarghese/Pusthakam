import { useTranslation } from "@/lib/translations";
import { useState, type FormEvent } from "react";
import { Link } from "react-router-dom";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Button } from "@/components/ui/Button";
import { SelectField, TextField } from "@/components/ui/Field";
import { Modal } from "@/components/ui/Modal";
import { ApiError, catalogueApi } from "@/lib/api";
import type { Contributor, ContributorRole, DuplicateHit, RecordDetail, RecordInput } from "@/lib/api/types";
import { ROLE_LABELS, useConfig } from "./shared";

interface Props {
  record?: RecordDetail;
  onClose: () => void;
  onSaved: (id: number) => void;
}

const emptyToNull = (s: string) => (s.trim() === "" ? null : s.trim());

/** Create or edit a bibliographic record. New records are checked for duplicates first; staff may still save. */
export function RecordFormDialog({ record, onClose, onSaved }: Props) {
  const { t } = useTranslation();
  const qc = useQueryClient();
  const config = useConfig();
  const schemes = config.data?.classificationSchemes ?? [];
  const [f, setF] = useState({
    title: record?.title ?? "",
    subtitle: record?.subtitle ?? "",
    isbn: record?.isbn13 ?? record?.isbn10 ?? "",
    publisher: record?.publisher ?? "",
    place: record?.publicationPlace ?? "",
    year: record?.publicationYear ? String(record.publicationYear) : "",
    edition: record?.edition ?? "",
    language: record?.language ?? "",
    pagination: record?.pagination ?? "",
    physical: record?.physicalDescription ?? "",
    series: record?.series ?? "",
    scheme: String(record?.classificationSchemeId ?? config.data?.classificationSchemes[0]?.id ?? ""),
    classification: record?.classificationNumber ?? "",
    subjects: (record?.subjects ?? []).join("\n"),
    keywords: record?.keywords ?? "",
    abstract: record?.abstract ?? "",
    notes: record?.notes ?? "",
  });
  const [people, setPeople] = useState<Contributor[]>(record?.contributors.length ? record.contributors : [{ name: "", role: "author" }]);
  const [error, setError] = useState<string | null>(null);
  const [duplicates, setDuplicates] = useState<DuplicateHit[] | null>(null);
  const set = (k: keyof typeof f) => (e: { target: { value: string } }) => setF({ ...f, [k]: e.target.value });

  const build = (): RecordInput => ({
    id: record?.id ?? null,
    title: f.title,
    subtitle: emptyToNull(f.subtitle),
    edition: emptyToNull(f.edition),
    publisher: emptyToNull(f.publisher),
    publicationPlace: emptyToNull(f.place),
    publicationYear: f.year.trim() === "" ? null : Number(f.year),
    language: emptyToNull(f.language),
    physicalDescription: emptyToNull(f.physical),
    pagination: emptyToNull(f.pagination),
    series: emptyToNull(f.series),
    abstract: emptyToNull(f.abstract),
    notes: emptyToNull(f.notes),
    isbn: emptyToNull(f.isbn),
    classificationSchemeId: f.classification.trim() && f.scheme ? Number(f.scheme) : null,
    classificationNumber: emptyToNull(f.classification),
    keywords: emptyToNull(f.keywords),
    contributors: people.filter((p) => p.name.trim() !== ""),
    subjects: f.subjects.split(/\r?\n|;/).map((s) => s.trim()).filter(Boolean),
  });

  const save = useMutation({
    mutationFn: () => catalogueApi.saveRecord(build()),
    onSuccess: async (id) => {
      await qc.invalidateQueries({ queryKey: ["catalogue"] });
      onSaved(id);
      onClose();
    },
    onError: (e) => setError(e instanceof ApiError ? e.message : "Could not save the record."),
  });

  const check = useMutation({
    mutationFn: () => {
      const firstAuthor = people.find((p) => p.role === "author" && p.name.trim())?.name ?? null;
      return catalogueApi.findDuplicates({
        id: null, title: f.title, isbn: emptyToNull(f.isbn), firstAuthor, publicationYear: f.year.trim() === "" ? null : Number(f.year),
      });
    },
    onSuccess: (hits) => (hits.length > 0 ? setDuplicates(hits) : save.mutate()),
    onError: (e) => setError(e instanceof ApiError ? e.message : "Could not check for duplicates."),
  });

  const submit = (e: FormEvent) => {
    e.preventDefault();
    setError(null);
    if (!f.title.trim()) return setError("Enter the title.");
    if (f.year.trim() !== "" && !/^\d{1,4}$/.test(f.year.trim())) return setError("The publication year must be a number such as 1999.");
    if (record) save.mutate();
    else check.mutate();
  };

  const busy = save.isPending || check.isPending;
  const setPerson = (i: number, patch: Partial<Contributor>) => setPeople(people.map((p, j) => (j === i ? { ...p, ...patch } : p)));

  return (
    <Modal title={record ? t("Edit record") : t("Add record")} onClose={onClose} wide>
      <form onSubmit={submit} className="space-y-4" noValidate>
        {duplicates && (
          <div role="alert" className="rounded-md border border-warn/40 bg-warn-bg p-3 text-warn">
            <p className="font-semibold">{t("This looks like a record you already have:")}</p>
            <ul className="mt-1 list-disc pl-5">
              {duplicates.map((d) => (
                <li key={d.id}>
                  <Link to={`/catalogue/${d.id}`} className="underline" onClick={onClose}>{d.title}</Link>
                  {d.authors && ` by ${d.authors}`}{d.publicationYear ? `, ${d.publicationYear}` : ""} ({d.reason.toLowerCase()})
                </li>
              ))}
            </ul>
            <p className="mt-2">{t("If this is another copy, open that record and add a copy instead. Otherwise save it as a new record.")}</p>
            <div className="mt-2 flex gap-2">
              <Button type="button" onClick={() => setDuplicates(null)}>{t("Keep editing")}</Button>
              <Button type="button" variant="primary" disabled={busy} onClick={() => save.mutate()}>{t("Save as a new record")}</Button>
            </div>
          </div>
        )}

        <div className="grid grid-cols-2 gap-3">
          <div className="col-span-2"><TextField label={t("Title")} value={f.title} onChange={set("title")} /></div>
          <div className="col-span-2"><TextField label={t("Subtitle")} value={f.subtitle} onChange={set("subtitle")} /></div>
        </div>

        <fieldset className="space-y-2">
          <legend className="text-[13px] font-medium">{t("Authors and other contributors")}</legend>
          {people.map((p, i) => (
            <div key={i} className="flex items-end gap-2">
              <div className="flex-1"><TextField label={t("Name {number}", { number: i + 1 })} value={p.name} onChange={(e) => setPerson(i, { name: e.target.value })} /></div>
              <div className="w-44">
                <SelectField label={t("Role")} value={p.role} onChange={(e) => setPerson(i, { role: e.target.value as ContributorRole })}>
                  {Object.entries(ROLE_LABELS).map(([v, l]) => <option key={v} value={v}>{t(l)}</option>)}
                </SelectField>
              </div>
              <Button type="button" variant="ghost" aria-label={t("Remove contributor {number}", { number: i + 1 })} onClick={() => setPeople(people.filter((_, j) => j !== i))} disabled={people.length === 1 && !p.name}>
                {t("Remove")}</Button>
            </div>
          ))}
          <Button type="button" size="sm" onClick={() => setPeople([...people, { name: "", role: people.some((p) => p.role === "author") ? "additional_author" : "author" }])}>
            {t("Add contributor")}</Button>
        </fieldset>

        <div className="grid grid-cols-3 gap-3">
          <TextField label="ISBN" value={f.isbn} onChange={set("isbn")} hint={t("10 or 13 digits")} />
          <TextField label={t("Publisher")} value={f.publisher} onChange={set("publisher")} />
          <TextField label={t("Place of publication")} value={f.place} onChange={set("place")} />
          <TextField label={t("Year")} value={f.year} onChange={set("year")} inputMode="numeric" />
          <TextField label={t("Edition")} value={f.edition} onChange={set("edition")} />
          <TextField label={t("Language")} value={f.language} onChange={set("language")} />
          <TextField label={t("Pagination")} value={f.pagination} onChange={set("pagination")} hint={t("For example 320 p.")} />
          <TextField label={t("Physical description")} value={f.physical} onChange={set("physical")} />
          <TextField label={t("Series")} value={f.series} onChange={set("series")} />
          <SelectField label={t("Classification scheme")} value={f.scheme} onChange={set("scheme")}>
            {schemes.map((s) => <option key={s.id} value={s.id}>{s.name}</option>)}
          </SelectField>
          <TextField label={t("Classification number")} value={f.classification} onChange={set("classification")} hint={t("For example 894.8123")} />
          <TextField label={t("Keywords")} value={f.keywords} onChange={set("keywords")} />
        </div>

        <div className="space-y-1">
          <label htmlFor="subjects" className="block text-[13px] font-medium">{t("Subjects")}</label>
          <textarea id="subjects" value={f.subjects} onChange={set("subjects")} rows={3}
            className="w-full rounded-md border border-line-strong bg-surface px-3 py-2 text-[13px]" />
          <p className="text-xs text-muted">{t("One subject heading per line.")}</p>
        </div>
        <div className="grid grid-cols-2 gap-3">
          <div className="space-y-1">
            <label htmlFor="abstract" className="block text-[13px] font-medium">{t("Abstract")}</label>
            <textarea id="abstract" value={f.abstract} onChange={set("abstract")} rows={3} className="w-full rounded-md border border-line-strong bg-surface px-3 py-2 text-[13px]" />
          </div>
          <div className="space-y-1">
            <label htmlFor="notes" className="block text-[13px] font-medium">{t("Notes")}</label>
            <textarea id="notes" value={f.notes} onChange={set("notes")} rows={3} className="w-full rounded-md border border-line-strong bg-surface px-3 py-2 text-[13px]" />
          </div>
        </div>

        {error && <p role="alert" className="text-xs text-danger">{t(error)}</p>}
        <div className="flex justify-end gap-2 pt-1">
          <Button type="button" onClick={onClose}>{t("Cancel")}</Button>
          <Button type="submit" variant="primary" disabled={busy}>{busy ? t("Saving…") : record ? t("Save changes") : t("Save record")}</Button>
        </div>
      </form>
    </Modal>
  );
}
