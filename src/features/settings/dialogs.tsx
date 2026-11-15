import { useState, type FormEvent, type ReactNode } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Button } from "@/components/ui/Button";
import { SelectField, TextField } from "@/components/ui/Field";
import { Modal } from "@/components/ui/Modal";
import { ApiError, catalogueApi, configApi } from "@/lib/api";
import type { AccessionFormat, ConfigOverview, Library, MemberCategory, Named, Policy, ProfileCode, Shelf } from "@/lib/api/types";
import { minorToInput, parseMinor } from "@/lib/money";

export type ListKind = "collection" | "location" | "itemCategory" | "memberCategory";

export const LIST_LABELS: Record<ListKind, { singular: string; plural: string }> = {
  collection: { singular: "collection", plural: "Collections" },
  location: { singular: "location", plural: "Locations" },
  itemCategory: { singular: "item category", plural: "Item categories" },
  memberCategory: { singular: "member category", plural: "Member categories" },
};

export const PROFILE_LABELS: Record<ProfileCode, string> = {
  school: "School library", college: "College library", university: "University library", public: "Public library",
  special: "Special library", research: "Research library", institutional: "Institutional library", private: "Private collection",
};

/** Shared save wrapper: runs the call, refreshes the configuration, closes on success, shows the backend's message on failure. */
function useSave(onClose: () => void, run: () => Promise<unknown>) {
  const qc = useQueryClient();
  const m = useMutation({
    mutationFn: run,
    onSuccess: async () => {
      await qc.invalidateQueries({ queryKey: ["config"] });
      onClose();
    },
  });
  const error = m.isError ? (m.error instanceof ApiError ? m.error.message : "Could not save.") : null;
  return { m, error };
}

function FormModal({
  title, onClose, onSubmit, pending, error, children, submitLabel = "Save",
}: {
  title: string; onClose: () => void; onSubmit: () => void; pending: boolean; error: string | null;
  children: ReactNode; submitLabel?: string;
}) {
  const submit = (e: FormEvent) => {
    e.preventDefault();
    onSubmit();
  };
  return (
    <Modal title={title} onClose={onClose}>
      <form onSubmit={submit} className="space-y-4" noValidate>
        {children}
        {error && <p role="alert" className="text-xs text-danger">{error}</p>}
        <div className="flex justify-end gap-2 pt-1">
          <Button onClick={onClose}>Cancel</Button>
          <Button type="submit" variant="primary" disabled={pending}>{pending ? "Saving…" : submitLabel}</Button>
        </div>
      </form>
    </Modal>
  );
}

function wholeNumber(text: string): number | null {
  return /^\d+$/.test(text.trim()) ? Number(text.trim()) : null;
}

export function ListDialog({ kind, item, onClose }: { kind: ListKind; item?: Named | MemberCategory; onClose: () => void }) {
  const label = LIST_LABELS[kind].singular;
  const [name, setName] = useState(item?.name ?? "");
  const [description, setDescription] = useState(item?.description ?? "");
  const [days, setDays] = useState(item && "membershipDays" in item ? String(item.membershipDays) : "365");
  const [local, setLocal] = useState<string | null>(null);

  const { m, error } = useSave(onClose, () => {
    const base = { id: item?.id ?? null, name, description: description.trim() || null };
    if (kind === "memberCategory") return configApi.saveMemberCategory({ ...base, membershipDays: Number(days) });
    return configApi.saveNamed(kind, base);
  });

  const submit = () => {
    if (!name.trim()) return setLocal("Enter a name.");
    if (kind === "memberCategory" && (wholeNumber(days) === null || Number(days) < 1)) return setLocal("Membership period must be a whole number of days.");
    setLocal(null);
    m.mutate();
  };

  return (
    <FormModal title={`${item ? "Edit" : "Add"} ${label}`} onClose={onClose} onSubmit={submit} pending={m.isPending} error={local ?? error}>
      <TextField label="Name" value={name} onChange={(e) => setName(e.target.value)} />
      <TextField label="Description (optional)" value={description} onChange={(e) => setDescription(e.target.value)} />
      {kind === "memberCategory" && (
        <TextField label="Membership period (days)" value={days} onChange={(e) => setDays(e.target.value)} inputMode="numeric" className="w-32" hint="How long a membership lasts before it needs renewing." />
      )}
    </FormModal>
  );
}

export function LibraryDialog({ library, onClose }: { library: Library; onClose: () => void }) {
  const [f, setF] = useState({ name: library.name, profile: library.profile, address: library.address ?? "", contact: library.contact ?? "", currency: library.currency });
  const [local, setLocal] = useState<string | null>(null);
  const { m, error } = useSave(onClose, () =>
    configApi.updateLibrary({ name: f.name, profile: f.profile, address: f.address.trim() || null, contact: f.contact.trim() || null, currency: f.currency, locale: library.locale }),
  );
  const submit = () => {
    if (!f.name.trim()) return setLocal("Enter your library's name.");
    if (!/^[A-Za-z]{3}$/.test(f.currency.trim())) return setLocal("Use a 3-letter currency code, such as INR or USD.");
    setLocal(null);
    m.mutate();
  };
  return (
    <FormModal title="Edit library details" onClose={onClose} onSubmit={submit} pending={m.isPending} error={local ?? error}>
      <TextField label="Library name" value={f.name} onChange={(e) => setF({ ...f, name: e.target.value })} />
      <SelectField label="Type of library" value={f.profile} onChange={(e) => setF({ ...f, profile: e.target.value as ProfileCode })}>
        {Object.entries(PROFILE_LABELS).map(([code, label]) => <option key={code} value={code}>{label}</option>)}
      </SelectField>
      <TextField label="Address (optional)" value={f.address} onChange={(e) => setF({ ...f, address: e.target.value })} />
      <TextField label="Contact (optional)" value={f.contact} onChange={(e) => setF({ ...f, contact: e.target.value })} />
      <TextField label="Currency" value={f.currency} onChange={(e) => setF({ ...f, currency: e.target.value.toUpperCase() })} maxLength={3} className="w-28" />
    </FormModal>
  );
}

export function ShelfDialog({ shelf, locations, onClose }: { shelf?: Shelf; locations: Named[]; onClose: () => void }) {
  const choices = locations.filter((l) => !l.archived || l.id === shelf?.locationId);
  const [locationId, setLocationId] = useState(String(shelf?.locationId ?? choices[0]?.id ?? ""));
  const [code, setCode] = useState(shelf?.code ?? "");
  const [description, setDescription] = useState(shelf?.description ?? "");
  const [local, setLocal] = useState<string | null>(null);
  const { m, error } = useSave(onClose, () =>
    configApi.saveShelf({ id: shelf?.id ?? null, locationId: Number(locationId), code, description: description.trim() || null }),
  );
  const submit = () => {
    if (!locationId) return setLocal("Add a location first.");
    if (!code.trim()) return setLocal("Enter a shelf code.");
    setLocal(null);
    m.mutate();
  };
  return (
    <FormModal title={`${shelf ? "Edit" : "Add"} shelf`} onClose={onClose} onSubmit={submit} pending={m.isPending} error={local ?? error}>
      <SelectField label="Location" value={locationId} onChange={(e) => setLocationId(e.target.value)}>
        {choices.map((l) => <option key={l.id} value={l.id}>{l.name}</option>)}
      </SelectField>
      <TextField label="Shelf code" value={code} onChange={(e) => setCode(e.target.value)} hint="For example A1 or 'Biography 2'." />
      <TextField label="Description (optional)" value={description} onChange={(e) => setDescription(e.target.value)} />
    </FormModal>
  );
}

export function PolicyDialog({ policy, config, onClose }: { policy?: Policy; config: ConfigOverview; onClose: () => void }) {
  const members = config.memberCategories.filter((c) => !c.archived || c.id === policy?.memberCategoryId);
  const items = config.itemCategories.filter((c) => !c.archived || c.id === policy?.itemCategoryId);
  const [f, setF] = useState({
    member: String(policy?.memberCategoryId ?? members[0]?.id ?? ""),
    item: String(policy?.itemCategoryId ?? items[0]?.id ?? ""),
    loanDays: String(policy?.loanDays ?? 14),
    maxLoans: String(policy?.maxLoans ?? 3),
    maxRenewals: String(policy?.maxRenewals ?? 1),
    renewalDays: String(policy?.renewalDays ?? 7),
    fine: minorToInput(policy?.finePerDayMinor ?? 0),
    grace: String(policy?.fineGraceDays ?? 0),
    cap: policy?.fineCapMinor == null ? "" : minorToInput(policy.fineCapMinor),
    hold: String(policy?.reservationHoldDays ?? 3),
  });
  const [local, setLocal] = useState<string | null>(null);
  const set = (k: keyof typeof f) => (e: { target: { value: string } }) => setF({ ...f, [k]: e.target.value });

  const { m, error } = useSave(onClose, () =>
    configApi.savePolicy({
      id: policy?.id ?? null,
      memberCategoryId: Number(f.member),
      itemCategoryId: Number(f.item),
      loanDays: Number(f.loanDays),
      maxLoans: Number(f.maxLoans),
      maxRenewals: Number(f.maxRenewals),
      renewalDays: Number(f.renewalDays),
      finePerDayMinor: parseMinor(f.fine) ?? 0,
      fineGraceDays: Number(f.grace),
      fineCapMinor: f.cap.trim() === "" ? null : parseMinor(f.cap),
      reservationHoldDays: Number(f.hold),
    }),
  );

  const submit = () => {
    if (!f.member || !f.item) return setLocal("Add a member category and an item category first.");
    const whole = [f.loanDays, f.maxLoans, f.maxRenewals, f.renewalDays, f.grace, f.hold];
    if (whole.some((v) => wholeNumber(v) === null)) return setLocal("Days and counts must be whole numbers.");
    if (parseMinor(f.fine) === null || (f.cap.trim() !== "" && parseMinor(f.cap) === null)) return setLocal("Fine amounts must look like 0, 1 or 2.50.");
    setLocal(null);
    m.mutate();
  };

  return (
    <FormModal title={`${policy ? "Edit" : "Add"} loan rule`} onClose={onClose} onSubmit={submit} pending={m.isPending} error={local ?? error}>
      <div className="grid grid-cols-2 gap-3">
        <SelectField label="Member category" value={f.member} onChange={set("member")}>
          {members.map((c) => <option key={c.id} value={c.id}>{c.name}</option>)}
        </SelectField>
        <SelectField label="Item category" value={f.item} onChange={set("item")}>
          {items.map((c) => <option key={c.id} value={c.id}>{c.name}</option>)}
        </SelectField>
        <TextField label="Loan period (days)" value={f.loanDays} onChange={set("loanDays")} inputMode="numeric" />
        <TextField label="Items at a time" value={f.maxLoans} onChange={set("maxLoans")} inputMode="numeric" />
        <TextField label="Renewals allowed" value={f.maxRenewals} onChange={set("maxRenewals")} inputMode="numeric" />
        <TextField label="Renewal period (days)" value={f.renewalDays} onChange={set("renewalDays")} inputMode="numeric" />
        <TextField label={`Fine per day (${config.library?.currency ?? ""})`} value={f.fine} onChange={set("fine")} inputMode="decimal" />
        <TextField label="Grace days" value={f.grace} onChange={set("grace")} inputMode="numeric" />
        <TextField label="Fine limit (optional)" value={f.cap} onChange={set("cap")} inputMode="decimal" hint="Leave empty for no limit." />
        <TextField label="Reservation hold (days)" value={f.hold} onChange={set("hold")} inputMode="numeric" />
      </div>
    </FormModal>
  );
}

export function AccessionFormatDialog({ format, onClose }: { format: AccessionFormat; onClose: () => void }) {
  const [prefix, setPrefix] = useState(format.prefix);
  const [padding, setPadding] = useState(String(format.padding));
  const [next, setNext] = useState(String(format.next));
  const [local, setLocal] = useState<string | null>(null);
  const qc = useQueryClient();
  const m = useMutation({
    mutationFn: () => catalogueApi.setAccessionFormat({ prefix, padding: Number(padding), next: Number(next) }),
    onSuccess: async () => {
      await qc.invalidateQueries({ queryKey: ["accession-format"] });
      onClose();
    },
  });
  const error = m.isError ? (m.error instanceof ApiError ? m.error.message : "Could not save.") : null;
  const example = `${prefix.trim()}${String(Math.max(1, Number(next) || 1)).padStart(Number(padding) || 6, "0")}`;
  const submit = () => {
    if (wholeNumber(padding) === null || wholeNumber(next) === null) return setLocal("Digits and next number must be whole numbers.");
    setLocal(null);
    m.mutate();
  };
  return (
    <FormModal title="Accession number format" onClose={onClose} onSubmit={submit} pending={m.isPending} error={local ?? error}>
      <TextField label="Prefix (optional)" value={prefix} onChange={(e) => setPrefix(e.target.value)} hint="Letters, digits, - _ / . only, such as LIB-" className="w-40" />
      <TextField label="Number of digits" value={padding} onChange={(e) => setPadding(e.target.value)} inputMode="numeric" className="w-28" hint="3 to 12" />
      <TextField label="Next number" value={next} onChange={(e) => setNext(e.target.value)} inputMode="numeric" className="w-40" hint="Numbers already in use are skipped automatically." />
      <p className="text-muted">The next copy will be numbered <span className="font-mono text-fg">{example}</span>.</p>
    </FormModal>
  );
}

export function ConfirmDeletePolicy({ onConfirm, onClose, pending, error }: { onConfirm: () => void; onClose: () => void; pending: boolean; error: string | null }) {
  return (
    <Modal
      title="Delete this loan rule?"
      onClose={onClose}
      footer={
        <>
          <Button onClick={onClose}>Cancel</Button>
          <Button variant="primary" onClick={onConfirm} disabled={pending}>{pending ? "Deleting…" : "Delete rule"}</Button>
        </>
      }
    >
      <p>Members in that category will no longer be able to borrow that kind of item. Loans already issued keep their own due dates.</p>
      {error && <p role="alert" className="text-xs text-danger">{error}</p>}
    </Modal>
  );
}
