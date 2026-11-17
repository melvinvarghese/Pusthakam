import { useState, type FormEvent } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { PageHeader } from "@/components/PageHeader";
import { Button } from "@/components/ui/Button";
import { SelectField, TextField } from "@/components/ui/Field";
import { Modal } from "@/components/ui/Modal";
import { Panel } from "@/components/ui/Panel";
import { staffApi, toApiError } from "@/lib/api";
import type { StaffDirectory, StaffRow } from "@/lib/api/types";
import { can, useSession } from "@/lib/session";
import { RecoveryCodeDialog } from "./RecoveryCodeDialog";

type Selection = { kind: "create" | "ownPassword" } | { kind: "edit" | "reset"; row: StaffRow };
const TITLES = { create: "Add staff account", edit: "Edit staff account", reset: "Reset staff password", ownPassword: "Change your password" };

function StaffDialog({ selection, roles, onClose, onSaved }: {
  selection: Selection;
  roles: StaffDirectory["roles"];
  onClose: () => void;
  onSaved: () => Promise<void>;
}) {
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");
  const { kind } = selection;
  const row = "row" in selection ? selection.row : undefined;
  const editingProfile = kind === "create" || kind === "edit";

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (saving) return;
    const data = new FormData(event.currentTarget);
    const string = (name: string) => String(data.get(name) ?? "");
    const password = string("password");
    if (kind !== "edit" && password !== string("confirmPassword")) {
      setError("The new passwords do not match.");
      return;
    }
    setSaving(true);
    setError("");
    try {
      if (kind === "create") {
        await staffApi.create({ username: string("username"), displayName: string("displayName"), role: string("role"), password });
      } else if (kind === "edit" && row) {
        await staffApi.update(row.id, { displayName: string("displayName"), role: string("role"), active: data.has("active") });
      } else if (kind === "reset" && row) {
        await staffApi.resetPassword(row.id, password);
      } else if (kind === "ownPassword") {
        await staffApi.changePassword(string("currentPassword"), password);
      }
      await onSaved();
    } catch (error) {
      setError(toApiError(error).message);
    } finally {
      setSaving(false);
    }
  }

  return (
    <Modal title={TITLES[kind]} onClose={() => { if (!saving) onClose(); }}>
      {row && <p className="text-sm">{row.displayName} <span className="text-muted">({row.username})</span></p>}
      <form onSubmit={(event) => void submit(event)} className="space-y-4">
        <fieldset disabled={saving} className="space-y-3">
          {kind === "create" && <TextField label="Username" name="username" autoComplete="off" required maxLength={40} />}
          {editingProfile && <>
            <TextField label="Display name" name="displayName" required maxLength={80} defaultValue={row?.displayName ?? ""} />
            <SelectField label="Role" name="role" defaultValue={row?.role ?? "viewer"} required>
              {roles.map((role) => <option key={role.code} value={role.code}>{role.name}</option>)}
            </SelectField>
          </>}
          {kind === "edit" && <label className="flex items-center gap-2 text-sm">
            <input name="active" type="checkbox" defaultChecked={row?.active} /> Active account
          </label>}
          {kind === "ownPassword" && <TextField label="Current password" name="currentPassword" type="password" autoComplete="current-password" required />}
          {kind !== "edit" && <>
            <TextField label="New password" name="password" type="password" autoComplete="new-password" required maxLength={200} />
            <TextField label="Confirm new password" name="confirmPassword" type="password" autoComplete="new-password" required maxLength={200} />
          </>}
        </fieldset>
        {error && <p role="alert" className="text-sm text-danger">{error}</p>}
        <div className="flex justify-end gap-2">
          <Button disabled={saving} onClick={onClose}>Cancel</Button>
          <Button type="submit" variant="primary" disabled={saving}>{saving ? "Saving…" : TITLES[kind]}</Button>
        </div>
      </form>
    </Modal>
  );
}

export function StaffPage() {
  const { session, signOut } = useSession();
  const allowed = can(session, "admin.staff");
  const client = useQueryClient();
  const [selection, setSelection] = useState<Selection | null>(null);
  const [recoveryOpen, setRecoveryOpen] = useState(false);
  const staff = useQuery({ queryKey: ["staff-directory", session.userId], queryFn: staffApi.list, enabled: allowed, retry: false });
  const directory = staff.data;
  const refresh = () => client.invalidateQueries({ queryKey: ["staff-directory"] });
  const unlock = useMutation({ mutationFn: (id: number) => staffApi.unlock(id), onSuccess: refresh });

  return (
    <div className="mx-auto max-w-[1040px] space-y-5 p-6">
      <PageHeader title={allowed ? "Staff & Permissions" : "Your Account"} actions={<>
        <Button onClick={() => setSelection({ kind: "ownPassword" })}>Change your password</Button>
        {allowed && <Button onClick={() => setRecoveryOpen(true)}>Recovery code</Button>}
        {allowed && staff.data && <Button variant="primary" onClick={() => setSelection({ kind: "create" })}>Add staff account</Button>}
      </>} />
      {!allowed && <Panel className="p-4"><p>{session.displayName} <span className="text-muted">({session.username})</span></p></Panel>}
      {allowed && staff.isPending && <p role="status" className="text-muted">Loading staff accounts.</p>}
      {staff.isError && <p role="alert" className="text-danger">{toApiError(staff.error).message}</p>}
      {unlock.isError && <p role="alert" className="text-danger">{toApiError(unlock.error).message}</p>}
      {allowed && directory && (
        <Panel>
          <div className="overflow-x-auto">
            <table aria-label="Staff accounts" className="w-full border-collapse text-left text-sm">
              <thead className="border-b border-line text-xs text-muted"><tr>
                {["Name", "Username", "Role", "Status", "Actions"].map((label) => <th key={label} scope="col" className="px-4 py-2">{label}</th>)}
              </tr></thead>
              <tbody>{directory.users.map((row) => (
                <tr key={row.id} className="border-b border-line last:border-b-0">
                  <th scope="row" className="px-4 py-2 font-medium">{row.displayName}{row.id === session.userId && <span className="text-muted"> (you)</span>}</th>
                  <td className="px-4 py-2 font-mono">{row.username}</td>
                  <td className="px-4 py-2">{directory.roles.find((role) => role.code === row.role)?.name ?? row.role}</td>
                  <td className="px-4 py-2">{!row.active ? "Inactive" : row.locked ? "Locked" : "Active"}</td>
                  <td className="whitespace-nowrap px-4 py-2">
                    {row.id !== session.userId && <>
                      <Button size="sm" variant="ghost" aria-label={`Edit ${row.username}`} onClick={() => setSelection({ kind: "edit", row })}>Edit</Button>
                      <Button size="sm" variant="ghost" aria-label={`Reset password for ${row.username}`} onClick={() => setSelection({ kind: "reset", row })}>Reset password</Button>
                    </>}
                    {row.locked && <Button size="sm" variant="ghost" disabled={unlock.isPending} onClick={() => unlock.mutate(row.id)}>Unlock</Button>}
                  </td>
                </tr>
              ))}</tbody>
            </table>
          </div>
        </Panel>
      )}
      {selection && <StaffDialog selection={selection} roles={staff.data?.roles ?? []} onClose={() => setSelection(null)}
        onSaved={async () => {
          setSelection(null);
          if (selection.kind === "ownPassword") await signOut();
          else await refresh();
        }} />}
      {recoveryOpen && <RecoveryCodeDialog onClose={() => setRecoveryOpen(false)} />}
    </div>
  );
}
