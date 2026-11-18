import { useState, type FormEvent, type ReactNode } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { CallNumberLabel } from "@/components/CallNumberLabel";
import { Button } from "@/components/ui/Button";
import { SelectField, TextField } from "@/components/ui/Field";
import { Modal } from "@/components/ui/Modal";
import { ApiError, catalogueApi } from "@/lib/api";
import type { AcquisitionSource, ConfigOverview, CopyBatchInput, CopyRecord, CopyStatus, RecordDetail } from "@/lib/api/types";
import { suggestCallNumber } from "@/lib/callNumber";
import { localDate } from "@/lib/dates";
import { minorToInput, parseMinor } from "@/lib/money";
import { BarcodeSvg, MANUAL_STATUSES, SOURCE_LABELS, STATUS_LABELS } from "./shared";
import { useLabelHeading } from "./LabelHeading";

function useSave<T>(onClose: () => void, run: () => Promise<T>) {
  const qc = useQueryClient();
  const m = useMutation({
    mutationFn: run,
    onSuccess: async () => {
      await qc.invalidateQueries({ queryKey: ["catalogue"] });
      onClose();
    },
  });
  const error = m.isError ? (m.error instanceof ApiError ? m.error.message : "Could not save.") : null;
  return { m, error };
}

function Form({ title, onClose, onSubmit, pending, error, children, submit = "Save", wide }: {
  title: string; onClose: () => void; onSubmit: () => void; pending: boolean; error: string | null;
  children: ReactNode; submit?: string; wide?: boolean;
}) {
  const handle = (e: FormEvent) => {
    e.preventDefault();
    onSubmit();
  };
  return (
    <Modal title={title} onClose={onClose} wide={wide}>
      <form onSubmit={handle} className="space-y-4" noValidate>
        {children}
        {error && <p role="alert" className="text-xs text-danger">{error}</p>}
        <div className="flex justify-end gap-2 pt-1">
          <Button type="button" onClick={onClose}>Cancel</Button>
          <Button type="submit" variant="primary" disabled={pending}>{pending ? "Saving…" : submit}</Button>
        </div>
      </form>
    </Modal>
  );
}

/** The fields shared by "add copies" and "edit copy". */
function useCopyFields(config: ConfigOverview, initial?: CopyRecord, defaults?: { callNumber: string }) {
  const [f, setF] = useState({
    category: String(initial?.itemCategoryId ?? config.itemCategories.find((c) => !c.archived)?.id ?? ""),
    collection: String(initial?.collectionId ?? ""),
    location: String(initial?.locationId ?? config.locations.find((l) => !l.archived)?.id ?? ""),
    shelf: String(initial?.shelfId ?? ""),
    callNumber: initial?.callNumber ?? defaults?.callNumber ?? "",
    date: initial?.acquisitionDate ?? localDate(),
    source: (initial?.acquisitionSource ?? "purchase") as AcquisitionSource,
    price: initial?.priceMinor == null ? "" : minorToInput(initial.priceMinor),
    vendor: initial?.vendor ?? "",
    invoice: initial?.invoiceRef ?? "",
    notes: initial?.notes ?? "",
  });
  const set = (k: keyof typeof f) => (e: { target: { value: string } }) => setF((s) => ({ ...s, [k]: e.target.value }));
  const shelves = config.shelves.filter((s) => !s.archived && String(s.locationId) === f.location);

  const check = (): string | null => {
    if (!f.category) return "Add an item category in Library Settings first.";
    if (f.price.trim() !== "" && parseMinor(f.price) === null) return "The price must look like 250 or 249.50.";
    return null;
  };
  const payload = () => ({
    itemCategoryId: Number(f.category),
    collectionId: f.collection ? Number(f.collection) : null,
    locationId: f.location ? Number(f.location) : null,
    shelfId: f.shelf ? Number(f.shelf) : null,
    callNumber: f.callNumber.trim() || null,
    acquisitionDate: f.date.trim() || null,
    acquisitionSource: f.source,
    priceMinor: f.price.trim() === "" ? null : parseMinor(f.price),
    vendor: f.vendor.trim() || null,
    invoiceRef: f.invoice.trim() || null,
    notes: f.notes.trim() || null,
  });

  const fields = (
    <div className="grid grid-cols-2 gap-3">
      <SelectField label="Item category" value={f.category} onChange={set("category")}>
        {config.itemCategories.filter((c) => !c.archived).map((c) => <option key={c.id} value={c.id}>{c.name}</option>)}
      </SelectField>
      <SelectField label="Collection" value={f.collection} onChange={set("collection")}>
        <option value="">None</option>
        {config.collections.filter((c) => !c.archived).map((c) => <option key={c.id} value={c.id}>{c.name}</option>)}
      </SelectField>
      <SelectField label="Location" value={f.location} onChange={(e) => setF((s) => ({ ...s, location: e.target.value, shelf: "" }))}>
        <option value="">Not set</option>
        {config.locations.filter((l) => !l.archived).map((l) => <option key={l.id} value={l.id}>{l.name}</option>)}
      </SelectField>
      <SelectField label="Shelf" value={f.shelf} onChange={set("shelf")}>
        <option value="">Not set</option>
        {shelves.map((s) => <option key={s.id} value={s.id}>{s.code}</option>)}
      </SelectField>
      <TextField label="Call number" value={f.callNumber} onChange={set("callNumber")} hint="For example 894.8123 BAS" />
      <TextField label="Date acquired" value={f.date} onChange={set("date")} hint="YYYY-MM-DD" />
      <SelectField label="Source" value={f.source} onChange={set("source")}>
        {Object.entries(SOURCE_LABELS).map(([v, l]) => <option key={v} value={v}>{l}</option>)}
      </SelectField>
      <TextField label={`Price (${config.library?.currency ?? ""})`} value={f.price} onChange={set("price")} inputMode="decimal" />
      <TextField label="Vendor or donor" value={f.vendor} onChange={set("vendor")} />
      <TextField label="Invoice reference" value={f.invoice} onChange={set("invoice")} />
    </div>
  );
  return { f, set, check, payload, fields };
}

export function AddCopiesDialog({ record, config, onClose }: { record: RecordDetail; config: ConfigOverview; onClose: () => void }) {
  const firstAuthor = record.contributors.find((c) => c.role === "author")?.name ?? null;
  const cf = useCopyFields(config, undefined, { callNumber: suggestCallNumber(record.classificationNumber, firstAuthor) });
  const [count, setCount] = useState("1");
  const [manual, setManual] = useState(false);
  const [numbers, setNumbers] = useState("");
  const [local, setLocal] = useState<string | null>(null);

  const { m, error } = useSave(onClose, () => {
    const lines = numbers.split(/\r?\n/).map((s) => s.trim()).filter(Boolean);
    const input: CopyBatchInput = {
      count: Number(count),
      accessionNumbers: manual ? lines : null,
      barcodes: null,
      ...cf.payload(),
    };
    return catalogueApi.addCopies(record.id, input);
  });

  const submit = () => {
    const n = Number(count);
    if (!/^\d+$/.test(count.trim()) || n < 1 || n > 500) return setLocal("Enter how many copies, from 1 to 500.");
    if (manual && numbers.split(/\r?\n/).map((s) => s.trim()).filter(Boolean).length !== n) {
      return setLocal("Type one accession number on each line, matching the number of copies.");
    }
    const problem = cf.check();
    if (problem) return setLocal(problem);
    setLocal(null);
    m.mutate();
  };

  return (
    <Form title={`Add copies of "${record.title}"`} onClose={onClose} onSubmit={submit} pending={m.isPending} error={local ?? error} submit="Add copies" wide>
      <div className="flex items-end gap-4">
        <TextField label="Number of copies" value={count} onChange={(e) => setCount(e.target.value)} inputMode="numeric" className="w-28" />
        <label className="flex items-center gap-2 pb-2 text-[13px]">
          <input type="checkbox" checked={manual} onChange={(e) => setManual(e.target.checked)} className="size-4" />
          I will type the accession numbers myself
        </label>
      </div>
      {manual ? (
        <div className="space-y-1">
          <label htmlFor="acc-numbers" className="block text-[13px] font-medium">Accession numbers</label>
          <textarea id="acc-numbers" value={numbers} onChange={(e) => setNumbers(e.target.value)} rows={3}
            className="w-full rounded-md border border-line-strong bg-surface px-3 py-2 font-mono text-[13px]" />
          <p className="text-xs text-muted">One per line. The barcode will be the same number.</p>
        </div>
      ) : (
        <p className="text-xs text-muted">Accession numbers and barcodes are numbered automatically using your accession number format.</p>
      )}
      {cf.fields}
    </Form>
  );
}

export function EditCopyDialog({ copy, config, onClose }: { copy: CopyRecord; config: ConfigOverview; onClose: () => void }) {
  const cf = useCopyFields(config, copy);
  const [acc, setAcc] = useState(copy.accessionNo);
  const [bar, setBar] = useState(copy.barcode);
  const [local, setLocal] = useState<string | null>(null);
  const { m, error } = useSave(onClose, () => catalogueApi.updateCopy(copy.id, { accessionNo: acc, barcode: bar, ...cf.payload() }));
  const submit = () => {
    const problem = cf.check();
    if (problem) return setLocal(problem);
    if (!acc.trim() || !bar.trim()) return setLocal("The accession number and barcode cannot be empty.");
    setLocal(null);
    m.mutate();
  };
  return (
    <Form title={`Edit copy ${copy.accessionNo}`} onClose={onClose} onSubmit={submit} pending={m.isPending} error={local ?? error} wide>
      <div className="grid grid-cols-2 gap-3">
        <TextField label="Accession number" value={acc} onChange={(e) => setAcc(e.target.value)} />
        <TextField label="Barcode" value={bar} onChange={(e) => setBar(e.target.value)} />
      </div>
      {cf.fields}
      <TextField label="Notes" value={cf.f.notes} onChange={cf.set("notes")} />
    </Form>
  );
}

export function StatusDialog({ copy, onClose }: { copy: CopyRecord; onClose: () => void }) {
  const locked = copy.status === "on_loan" || copy.status === "reserved_hold";
  const [status, setStatus] = useState<CopyStatus>(locked ? "available" : copy.status);
  const [note, setNote] = useState("");
  const { m, error } = useSave(onClose, () => catalogueApi.setCopyStatus(copy.id, status, note.trim() || null));
  return (
    <Form title={`Change status of ${copy.accessionNo}`} onClose={onClose} onSubmit={() => m.mutate()} pending={m.isPending} error={error} submit="Change status">
      {locked ? (
        <p>This copy is {STATUS_LABELS[copy.status].toLowerCase()}. Return it first, then change its status.</p>
      ) : (
        <>
          <SelectField label="Status" value={status} onChange={(e) => setStatus(e.target.value as CopyStatus)}>
            {MANUAL_STATUSES.map((s) => <option key={s} value={s}>{STATUS_LABELS[s]}</option>)}
          </SelectField>
          <TextField label="Note (optional)" value={note} onChange={(e) => setNote(e.target.value)} hint="Recorded in the copy's history." />
        </>
      )}
    </Form>
  );
}

/** A printable label: spine call number, title and a Code 128 barcode. Full label sheets come with the Printing centre. */
export function LabelDialog({ copy, title, schemeCode, onClose }: {
  copy: CopyRecord; title: string; schemeCode?: string; onClose: () => void;
}) {
  const { heading, field } = useLabelHeading();
  return (
    <Modal
      title={`Label for ${copy.accessionNo}`}
      onClose={onClose}
      footer={<><Button onClick={onClose}>Close</Button><Button variant="primary" onClick={() => window.print()}>Print label</Button></>}
    >
      {field}
      <div className="print-area flex items-center gap-4 rounded-md border border-line p-4">
        {copy.callNumber && <CallNumberLabel callNumber={copy.callNumber} schemeCode={schemeCode} size="lg" />}
        <div className="min-w-0">
          {heading && <div className="break-words font-semibold">{heading}</div>}
          <div className="truncate font-semibold">{title}</div>
          <BarcodeSvg value={copy.barcode} displayValue={false} />
          <div className="font-mono text-xs">{copy.barcode}</div>
          {copy.accessionNo !== copy.barcode && <div className="font-mono text-xs">Accession {copy.accessionNo}</div>}
        </div>
      </div>
    </Modal>
  );
}
