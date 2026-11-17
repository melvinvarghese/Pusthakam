import { useState, type FormEvent } from "react";
import { AuthLayout } from "./AuthLayout";
import { Button } from "@/components/ui/Button";
import { TextField } from "@/components/ui/Field";
import { staffApi, toApiError } from "@/lib/api";

export function RecoverAccount({ onCancel, onRecovered }: { onCancel: () => void; onRecovered: () => void }) {
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");
  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (saving) return;
    const data = new FormData(event.currentTarget);
    const password = String(data.get("password") ?? "");
    if (password !== data.get("confirmPassword")) {
      setError("The new passwords do not match.");
      return;
    }
    setSaving(true);
    setError("");
    try {
      await staffApi.recover(String(data.get("username") ?? ""), String(data.get("code") ?? ""), password);
      onRecovered();
    } catch (error) {
      setError(toApiError(error).message);
    } finally {
      setSaving(false);
    }
  }
  return (
    <AuthLayout>
      <form onSubmit={(event) => void submit(event)} className="w-[min(24rem,100%)] space-y-4 rounded-lg border border-line bg-surface p-6 text-fg">
        <h1 className="text-xl font-bold">Recover administrator account</h1>
        <fieldset disabled={saving} className="space-y-3">
          <TextField label="Administrator username" name="username" autoComplete="username" required autoFocus />
          <TextField label="Recovery code" name="code" type="password" autoComplete="off" required maxLength={64} />
          <TextField label="New password" name="password" type="password" autoComplete="new-password" required maxLength={200} />
          <TextField label="Confirm new password" name="confirmPassword" type="password" autoComplete="new-password" required maxLength={200} />
        </fieldset>
        {error && <p role="alert" className="text-sm text-danger">{error}</p>}
        <div className="flex flex-wrap justify-end gap-2">
          <Button disabled={saving} onClick={onCancel}>Back to sign in</Button>
          <Button type="submit" variant="primary" disabled={saving}>{saving ? "Resetting…" : "Reset password"}</Button>
        </div>
      </form>
    </AuthLayout>
  );
}
