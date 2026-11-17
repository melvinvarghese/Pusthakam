import { useState, type FormEvent } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Button } from "@/components/ui/Button";
import { CheckField, SelectField, TextField } from "@/components/ui/Field";
import { Panel } from "@/components/ui/Panel";
import { appApi, toApiError } from "@/lib/api";

type LibraryField =
  | { label: string; kind: "text"; value: string }
  | { label: string; kind: "number"; value: number }
  | { label: string; kind: "checkbox"; value: boolean };

const SETTING = "library.custom_fields";
const QUERY_KEY = ["library-custom-fields"];

export function LibraryCustomFields({ canEdit }: { canEdit: boolean }) {
  const client = useQueryClient();
  const [editing, setEditing] = useState<number | null>(null);
  const [kind, setKind] = useState<LibraryField["kind"]>("text");
  const fields = useQuery({
    queryKey: QUERY_KEY,
    queryFn: async () => JSON.parse((await appApi.setting(SETTING)) ?? "[]") as LibraryField[],
    retry: false,
  });
  const save = useMutation({
    mutationFn: (next: LibraryField[]) => appApi.setSetting(SETTING, JSON.stringify(next)),
    onSuccess: (_, next) => {
      client.setQueryData(QUERY_KEY, next);
      setEditing(null);
    },
  });
  const current = fields.data ?? [];
  const field = editing === null ? undefined : current[editing];
  const error = save.error ?? fields.error;

  function edit(index: number) {
    save.reset();
    setKind(current[index]?.kind ?? "text");
    setEditing(index);
  }

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!canEdit || editing === null || save.isPending) return;
    const data = new FormData(event.currentTarget);
    const label = String(data.get("label") ?? "").trim();
    const value = String(data.get("value") ?? "");
    let next: LibraryField;
    switch (kind) {
      case "number": next = { label, kind, value: Number(value) }; break;
      case "checkbox": next = { label, kind, value: data.has("value") }; break;
      case "text": next = { label, kind, value }; break;
    }
    const updated = [...current];
    updated[editing] = next;
    save.mutate(updated);
  }

  return (
    <Panel aria-labelledby="library-custom-fields" className="space-y-3 p-4">
      <h2 id="library-custom-fields" className="font-semibold">Custom library fields</h2>
      {fields.isPending && <p role="status">Loading custom fields.</p>}
      {fields.isSuccess && current.length === 0 && <p className="text-sm text-muted">No custom fields added.</p>}
      {current.length > 0 && <dl className="space-y-2">
        {current.map((item, index) => (
          <div key={item.label} className="flex flex-wrap items-baseline gap-x-3">
            <dt className="font-medium">{item.label}</dt>
            <dd className="break-all text-muted">{item.kind === "checkbox" ? (item.value ? "Yes" : "No") : String(item.value)}</dd>
            {canEdit && <dd><Button size="sm" disabled={save.isPending} aria-label={`Edit ${item.label}`}
              onClick={() => edit(index)}>Edit</Button></dd>}
          </div>
        ))}
      </dl>}
      {canEdit && fields.isSuccess && editing === null &&
        <Button size="sm" onClick={() => edit(current.length)}>Add custom field</Button>}
      {canEdit && editing !== null && (
        <form key={editing} className="space-y-3" onSubmit={submit}>
          <fieldset disabled={save.isPending} className="space-y-2">
            <TextField label="Field label" name="label" required maxLength={80} defaultValue={field?.label} />
            <SelectField label="Field type" value={kind} onChange={(event) => setKind(event.target.value as LibraryField["kind"])}>
              <option value="text">Text</option>
              <option value="number">Number</option>
              <option value="checkbox">Yes or no</option>
            </SelectField>
            {kind === "checkbox"
              ? <CheckField label="Field value" name="value" defaultChecked={field?.kind === "checkbox" && field.value} />
              : <TextField key={kind} label="Field value" name="value" type={kind} step={kind === "number" ? "any" : undefined}
                  required={kind === "number"} maxLength={500} defaultValue={field?.kind === kind ? String(field.value) : undefined} />}
          </fieldset>
          <div className="flex gap-2">
            <Button type="submit" disabled={save.isPending}>Save custom field</Button>
            <Button disabled={save.isPending} onClick={() => { save.reset(); setEditing(null); }}>Cancel</Button>
          </div>
        </form>
      )}
      {error && <p role="alert" className="text-sm text-danger">{toApiError(error).message}</p>}
    </Panel>
  );
}
