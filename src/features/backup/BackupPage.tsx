import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { PageHeader } from "@/components/PageHeader";
import { Button } from "@/components/ui/Button";
import { TextField } from "@/components/ui/Field";
import { Modal } from "@/components/ui/Modal";
import { Panel } from "@/components/ui/Panel";
import { backupApi, toApiError } from "@/lib/api";
import { can, useSession } from "@/lib/session";

export function BackupPage() {
  const { session } = useSession();
  const allowed = can(session, "admin.backup");
  const client = useQueryClient();
  const [restoreTarget, setRestoreTarget] = useState<string | null>(null);
  const [copyTarget, setCopyTarget] = useState<string | null>(null);
  const list = useQuery({ queryKey: ["database-backups"], queryFn: backupApi.list, enabled: allowed, refetchInterval: 60_000 });
  const create = useMutation({
    mutationFn: backupApi.create,
    onSuccess: () => client.invalidateQueries({ queryKey: ["database-backups"] }),
  });
  const schedule = useMutation({
    mutationFn: backupApi.setDaily,
    onSuccess: () => client.invalidateQueries({ queryKey: ["database-backups"] }),
  });
  const verify = useMutation({ mutationFn: backupApi.verify });
  const copy = useMutation({
    mutationFn: ({ name, folder }: { name: string; folder: string }) => backupApi.copyToFolder(name, folder),
    onSuccess: () => setCopyTarget(null),
  });
  const restore = useMutation({
    mutationFn: backupApi.requestRestore,
    onSuccess: async () => {
      await client.invalidateQueries({ queryKey: ["database-backups"] });
      setRestoreTarget(null);
    },
  });
  const cancelRestore = useMutation({
    mutationFn: backupApi.cancelRestore,
    onSuccess: () => client.invalidateQueries({ queryKey: ["database-backups"] }),
  });
  if (!allowed) return <p className="p-6" role="alert">You do not have permission to manage backups.</p>;

  return (
    <div className="mx-auto max-w-[1040px] space-y-5 p-6">
      <PageHeader title="Database Backups" description="Save a checked snapshot of the library database."
        actions={<Button variant="primary" disabled={create.isPending} onClick={() => create.mutate()}>
          {create.isPending ? "Creating backup…" : "Create backup"}
        </Button>} />
      {create.isError && <p role="alert" className="text-danger">{toApiError(create.error).message}</p>}
      {create.data && <p role="status">Backup saved and checked. <span className="font-mono">{create.data.name}</span></p>}
      {copy.data && <p role="status">Backup copy saved and checked. <span className="font-mono">{copy.data.name}</span></p>}
      {list.isPending && <p role="status" className="text-muted">Loading backups.</p>}
      {list.isError && <p role="alert" className="text-danger">{toApiError(list.error).message}</p>}
      {list.data && (
        <Panel className="p-4">
          {list.data.lastRestore && <p className="mb-4" role="status">{list.data.lastRestore}</p>}
          {list.data.pendingRestore && (
            <div className="mb-4 space-y-2 rounded border border-line p-3">
              <p role="status">Restore queued for <span className="font-mono">{list.data.pendingRestore}</span>.</p>
              <p className="text-sm">Close every copy of Pusthakam, then reopen it to restore. The current database will be backed up first.</p>
              <Button size="sm" disabled={cancelRestore.isPending} onClick={() => cancelRestore.mutate()}>Cancel queued restore</Button>
              {cancelRestore.isError && <p role="alert" className="text-danger">{toApiError(cancelRestore.error).message}</p>}
            </div>
          )}
          <label className="mb-4 flex items-center gap-2 text-sm">
            <input type="checkbox" checked={list.data.dailyEnabled} disabled={schedule.isPending}
              onChange={(event) => schedule.mutate(event.target.checked)} />
            Back up on first launch each day or when the date changes while Pusthakam is open
          </label>
          <p className="mb-4 text-sm text-muted">A checked daily backup replaces older daily copies. Manual backups and backups queued for restore are kept.</p>
          {schedule.isError && <p role="alert" className="mb-4 text-danger">{toApiError(schedule.error).message}</p>}
          <p className="text-sm font-medium">Backup folder</p>
          <p className="break-all font-mono text-xs text-muted">{list.data.folder}</p>
          <p className="mb-4 text-sm text-muted">Keep a copy on a separate drive or another computer so you can recover if this computer is lost.</p>
          {list.data.files.length === 0 ? <p className="text-muted">No database backups yet.</p> : (
            <ul aria-label="Saved backups" className="divide-y divide-line">
              {list.data.files.map((file) => (
                <li key={file.name} className="flex flex-wrap items-baseline justify-between gap-2 py-3 text-sm">
                  <span className="break-all font-mono">{file.name}</span>
                  <div className="flex flex-wrap items-center gap-2">
                    <span className="text-muted">{file.sizeBytes.toLocaleString()} bytes</span>
                    <Button size="sm" disabled={verify.isPending} onClick={() => verify.mutate(file.name)} aria-label={`Verify ${file.name}`}>
                      {verify.isPending && verify.variables === file.name ? "Checking…" : "Verify"}
                    </Button>
                    <Button size="sm" onClick={() => { copy.reset(); setCopyTarget(file.name); }}
                      aria-label={`Copy ${file.name} to folder`}>Copy to folder</Button>
                    <Button size="sm" disabled={list.data.pendingRestore !== null} aria-label={`Restore ${file.name}`}
                      onClick={() => { restore.reset(); setRestoreTarget(file.name); }}>Restore…</Button>
                  </div>
                  {verify.variables === file.name && verify.isSuccess && (
                    <p className="w-full" role="status">Integrity check passed. Database schema version {verify.data}.</p>
                  )}
                  {verify.variables === file.name && verify.isError && (
                    <p className="w-full text-danger" role="alert">Could not verify this backup. {toApiError(verify.error).message}</p>
                  )}
                </li>
              ))}
            </ul>
          )}
        </Panel>
      )}
      {copyTarget && (
        <Modal title="Copy backup to folder" onClose={() => { if (!copy.isPending) setCopyTarget(null); }}>
          <form className="space-y-3" onSubmit={(event) => {
            event.preventDefault();
            const folder = String(new FormData(event.currentTarget).get("folder") ?? "");
            copy.mutate({ name: copyTarget, folder });
          }}>
            <p className="break-all font-mono">{copyTarget}</p>
            <TextField label="Destination folder" name="folder" required disabled={copy.isPending} />
            {copy.isError && <p role="alert" className="text-danger">{toApiError(copy.error).message}</p>}
            <Button type="submit" variant="primary" disabled={copy.isPending}>{copy.isPending ? "Copying…" : "Save copy"}</Button>
          </form>
        </Modal>
      )}
      {restoreTarget && (
        <Modal title="Restore database on next launch" onClose={() => { if (!restore.isPending) setRestoreTarget(null); }}
          footer={<>
            <Button disabled={restore.isPending} onClick={() => setRestoreTarget(null)}>Cancel</Button>
            <Button variant="primary" disabled={restore.isPending} onClick={() => restore.mutate(restoreTarget)}>
              {restore.isPending ? "Checking backup…" : "Queue restore"}
            </Button>
          </>}>
          <p className="break-all font-mono">{restoreTarget}</p>
          <p className="text-sm">Restoring replaces the library database, including staff accounts and changes made since this backup. Pusthakam saves a recovery copy before replacing it.</p>
          {restore.isError && <p role="alert" className="text-danger">{toApiError(restore.error).message}</p>}
        </Modal>
      )}
    </div>
  );
}
