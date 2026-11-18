import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Button } from "@/components/ui/Button";
import { TextField } from "@/components/ui/Field";
import { Modal } from "@/components/ui/Modal";
import { ApiError, circulationApi, toApiError } from "@/lib/api";
import { fileToBase64 } from "@/lib/files";

export default function LoanImport({ onClose }: { onClose: () => void }) {
  const client = useQueryClient();
  const [sourceSystem, setSourceSystem] = useState("");
  const [useCurrentTerms, setUseCurrentTerms] = useState(false);
  const [file, setFile] = useState<File | null>(null);
  const load = useMutation({
    mutationFn: async (source: File) => {
      if (source.size > 10 * 1024 * 1024) throw new ApiError({ code: "VALIDATION", message: "Loan CSV must be 10 MB or smaller." });
      const dataBase64 = await fileToBase64(source);
      return { dataBase64, preview: await circulationApi.previewImport(dataBase64) };
    },
  });
  const commit = useMutation({
    mutationFn: async () => {
      if (!load.data) throw new Error("Preview the file first.");
      return circulationApi.commitImport(load.data.dataBase64, sourceSystem.trim(), load.data.preview.fingerprint, useCurrentTerms);
    },
    onSuccess: () => {
      for (const key of ["catalogue", "member-loans", "circulation", "dashboard", "reports"]) void client.invalidateQueries({ queryKey: [key] });
    },
  });
  const busy = load.isPending || commit.isPending;
  const preview = load.data?.preview;
  return <Modal title="Preview loan import" onClose={() => { if (!busy) onClose(); }} wide
    footer={<Button disabled={busy} onClick={onClose}>Close</Button>}>
    {commit.isSuccess ? <p role="status">Imported {commit.data} loans. Original source labels and dates were preserved.</p> : <>
    <p>Review this loan-history CSV before confirming. All records are imported together; an error rolls back the entire batch.</p>
    <p className="text-sm text-muted">Use the Pusthakam loan CSV format, not a raw Koha export. Import members and catalogue copies first.</p>
    <details className="text-sm"><summary>Required CSV format</summary>
      <p className="break-words font-mono">source_id,member_number,barcode,issued_at,due_on,returned_at,source_staff</p>
      <p>Use UTF-8 with these columns in order. Timestamps: YYYY-MM-DDTHH:MM:SSZ (UTC). Due dates: YYYY-MM-DD. Leave returned_at empty for an open loan. Keep member numbers and barcodes as text, including leading zeros. Maximum 10 MB and 25,000 records.</p>
    </details>
    <TextField label="Loan history CSV" type="file" accept=".csv,text/csv" disabled={busy}
      onChange={(event) => { setFile(event.target.files?.[0] ?? null); load.reset(); commit.reset(); setUseCurrentTerms(false); }} />
    <Button disabled={!file || busy} onClick={() => { if (file) { load.reset(); commit.reset(); setUseCurrentTerms(false); load.mutate(file); } }}>
      {load.isPending ? "Checking CSV…" : "Preview CSV"}
    </Button>
    {load.error && <p role="alert" className="text-danger">{toApiError(load.error).message}</p>}
    {preview && <>
      <p role="status">{preview.total} records: {preview.open} valid open loans, {preview.closed} valid closed loans, {preview.invalid} invalid.</p>
      {preview.total === 0 && <p>The file contains no loan records.</p>}
      {preview.problems.length > 0 && <div><p>Problems{preview.invalid > preview.problems.length ? " (first 100)" : ""}</p>
        <ul className="list-disc pl-5 text-sm">{preview.problems.map((problem) => <li key={problem}>{problem}</li>)}</ul>
      </div>}
      <TextField label="Source system or library" maxLength={128} value={sourceSystem} disabled={busy}
        hint="Use the same name for later batches from this source; source loan IDs cannot be reused."
        onChange={(event) => { setSourceSystem(event.target.value); commit.reset(); }} />
      <p className="text-sm text-muted">Original source staff labels are preserved separately. Your account is recorded as the importing operator. This file imports loans only; renewals, reservations, fines and settlements are not included. Unknown historical return conditions remain unspecified.</p>
      {preview.open > 0 && <label className="flex items-start gap-2 text-sm"><input type="checkbox" checked={useCurrentTerms} disabled={busy}
        onChange={(event) => setUseCurrentTerms(event.target.checked)} />
        Use current fine policies for these open loans. I understand that future returns may assess fines using these policies, including days overdue before import.
      </label>}
      <p className="text-sm">Confirming imports {preview.open} open and {preview.closed} closed loans. Closed history does not change current copy status or assess past fines.</p>
      {commit.error && <p role="alert" className="text-danger">{toApiError(commit.error).message} Nothing was imported. Preview again if library data changed.</p>}
      <Button variant="primary" disabled={busy || !preview.total || preview.invalid > 0 || !sourceSystem.trim() || (preview.open > 0 && !useCurrentTerms)}
        onClick={() => commit.mutate()}>{commit.isPending ? "Importing loans…" : "Confirm import loans"}</Button>
    </>}
    </>}
  </Modal>;
}
