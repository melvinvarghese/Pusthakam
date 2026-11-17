import { useState, type FormEvent } from "react";
import { Button } from "@/components/ui/Button";
import { TextField } from "@/components/ui/Field";
import { Modal } from "@/components/ui/Modal";
import { staffApi, toApiError } from "@/lib/api";
import { downloadTextFile } from "@/lib/files";

export function RecoveryCodeDialog({ onClose }: { onClose: () => void }) {
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");
  const [code, setCode] = useState("");

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (saving) return;
    const fields = new FormData(event.currentTarget);
    setSaving(true);
    setError("");
    try {
      setCode(await staffApi.generateRecoveryCode(String(fields.get("password") ?? "")));
    } catch (error) {
      setError(toApiError(error).message);
    } finally {
      setSaving(false);
    }
  }

  return (
    <Modal title="Administrator recovery code" onClose={() => { if (!saving) onClose(); }}>
      {code ? <>
        <p className="text-sm">Save this code securely. It appears once and can be used once to reset your password.</p>
        <code className="block select-all break-all rounded bg-band p-3">{code}</code>
        <p className="text-sm">The exported text file contains the recovery code. Keep it somewhere only you or an authorized recovery contact can access.</p>
        <Button onClick={() => downloadTextFile(code + "\n", "pusthakam-recovery-code.txt")}>Download recovery code (.txt)</Button>
        <Button onClick={onClose}>Done</Button>
      </> : (
        <form onSubmit={(event) => void submit(event)} className="space-y-4">
          <p className="text-sm">Generating a code replaces any previous code for your account.</p>
          <TextField label="Current password" name="password" type="password" autoComplete="current-password" required disabled={saving} />
          {error && <p role="alert" className="text-sm text-danger">{error}</p>}
          <div className="flex justify-end gap-2">
            <Button disabled={saving} onClick={onClose}>Cancel</Button>
            <Button type="submit" variant="primary" disabled={saving}>{saving ? "Generating…" : "Generate code"}</Button>
          </div>
        </form>
      )}
    </Modal>
  );
}
