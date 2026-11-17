import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Button } from "@/components/ui/Button";
import { SelectField, TextField } from "@/components/ui/Field";
import { Modal } from "@/components/ui/Modal";
import { ApiError, membersApi, toApiError } from "@/lib/api";
import type { Named } from "@/lib/api/types";
import { fileToBase64 } from "@/lib/files";

export default function KohaMemberImport({ categories, onClose }: { categories: Named[]; onClose: () => void }) {
  const client = useQueryClient();
  const [file, setFile] = useState<File | null>(null);
  const [mappings, setMappings] = useState<Record<string, number>>({});
  const load = useMutation({
    mutationFn: async (source: File) => {
      if (source.size > 10 * 1024 * 1024) {
        throw new ApiError({ code: "VALIDATION", message: "The patron CSV must be 10 MB or smaller." });
      }
      const dataBase64 = await fileToBase64(source);
      return { dataBase64, preview: await membersApi.previewKoha(dataBase64) };
    },
  });
  const commit = useMutation({
    mutationFn: ({ dataBase64, ready }: { dataBase64: string; ready: number }) =>
      membersApi.importKoha(dataBase64, mappings, ready),
    onSuccess: () => { void client.invalidateQueries({ queryKey: ["members"] }); },
  });
  const preview = load.data?.preview;
  const codes = Object.entries(preview?.categories ?? {});
  const active = categories.filter((category) => !category.archived);
  const mapped = codes.every(([code]) => Object.hasOwn(mappings, code) && active.some((category) => category.id === mappings[code]));
  const busy = load.isPending || commit.isPending;

  return (
    <Modal title="Import Koha patrons" onClose={() => { if (!busy) onClose(); }} wide
      footer={<Button disabled={busy} onClick={onClose}>{commit.isSuccess ? "Done" : "Close"}</Button>}>
      {commit.isSuccess ? <p role="status">Imported {commit.data} {commit.data === 1 ? "member" : "members"}.</p> : (
        <>
          <TextField label="Koha patron CSV" type="file" accept=".csv,text/csv" disabled={busy}
            hint="Use a UTF-8 CSV with cardnumber, surname, and categorycode columns."
            onChange={(event) => {
              setFile(event.target.files?.[0] ?? null);
              load.reset();
              commit.reset();
              setMappings({});
            }} />
          <Button disabled={!file || busy} onClick={() => {
            if (!file) return;
            setMappings({});
            commit.reset();
            load.mutate(file);
          }}>{load.isPending ? "Reading CSV…" : "Preview CSV"}</Button>
          {load.error && <p role="alert" className="text-danger">{toApiError(load.error).message}</p>}
          {preview && (
            <>
              <p role="status">{preview.ready} ready to import · {preview.skipped} skipped</p>
              {preview.problems.length > 0 && (
                <details className="text-sm">
                  <summary>Skipped rows{preview.skipped > preview.problems.length ? " (first 50)" : ""}</summary>
                  <ul className="list-disc space-y-1 pl-5 pt-2">{preview.problems.map((problem) => <li key={problem}>{problem}</li>)}</ul>
                </details>
              )}
              {preview.sample.length > 0 && (
                <ul aria-label="Sample patrons" className="space-y-1 rounded-md border border-line p-3 text-sm">
                  {preview.sample.map((patron) => (
                    <li key={patron.memberNumber} className="break-words">
                      <span className="font-mono">{patron.memberNumber}</span> · {patron.name} · {patron.categoryCode}
                      {patron.expiresOn && <> · Expires {patron.expiresOn}</>}
                    </li>
                  ))}
                </ul>
              )}
              {codes.length > 0 && (
                <fieldset disabled={busy} className="grid gap-3 sm:grid-cols-2">
                  <legend className="mb-2 text-sm font-medium">Map Koha categories</legend>
                  {codes.map(([code, count]) => (
                    <SelectField key={code} label={`${code} (${count})`}
                      value={Object.hasOwn(mappings, code) ? mappings[code] : ""}
                      onChange={(event) => setMappings({ ...mappings, [code]: Number(event.target.value) })}>
                      <option value="" disabled>Choose member category</option>
                      {active.map((category) => <option key={category.id} value={category.id}>{category.name}</option>)}
                    </SelectField>
                  ))}
                </fieldset>
              )}
              {commit.error && <p role="alert" className="text-danger">{toApiError(commit.error).message}</p>}
              <Button variant="primary" disabled={busy || !preview.ready || !mapped} onClick={() => {
                if (load.data) commit.mutate({ dataBase64: load.data.dataBase64, ready: preview.ready });
              }}>{commit.isPending ? "Importing…" : "Import members"}</Button>
            </>
          )}
        </>
      )}
    </Modal>
  );
}
