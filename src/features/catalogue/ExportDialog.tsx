import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { Button } from "@/components/ui/Button";
import { Modal } from "@/components/ui/Modal";
import { SelectField } from "@/components/ui/Field";
import { catalogueApi, isDesktop, toApiError } from "@/lib/api";
import type { CatalogueExportFormat } from "@/lib/api/types";
import { downloadBase64File } from "@/lib/files";

export function ExportDialog({ ids, onClose }: { ids: number[]; onClose: () => void }) {
  const [format, setFormat] = useState<CatalogueExportFormat>("json");
  const output = useMutation({ mutationFn: () => catalogueApi.exportRecords(ids, format) });
  const save = useMutation({ mutationFn: async () => {
    const file = output.data;
    if (!file) throw new Error("Prepare the export first.");
    if (!isDesktop()) {
      downloadBase64File(file.dataBase64, file.fileName, file.mimeType);
      return { native: false, path: null };
    }
    const saved = await catalogueApi.saveExport(file.fileName, file.dataBase64);
    if (!saved.native) downloadBase64File(file.dataBase64, file.fileName, file.mimeType);
    return saved;
  } });
  const busy = output.isPending || save.isPending;
  return <Modal title="Export catalogue records" onClose={() => { if (!busy) onClose(); }} footer={<>
    <Button disabled={busy} onClick={onClose}>Close</Button>
    {output.data ? <Button variant="primary" disabled={save.isPending} onClick={() => save.mutate()}>
      {save.isPending ? "Saving…" : "Save file"}</Button> : <Button variant="primary" disabled={output.isPending || ids.length === 0} onClick={() => output.mutate()}>
      {output.isPending ? "Preparing…" : "Prepare export"}
    </Button>}
  </>}>
    <p>{ids.length} selected title{ids.length === 1 ? "" : "s"}, including their copies. Selection can include other results pages.</p>
    <SelectField label="File format" value={format} disabled={busy} onChange={(e) => { setFormat(e.target.value as CatalogueExportFormat); output.reset(); save.reset(); }}>
      <option value="json">JSON</option><option value="csv">CSV</option><option value="xlsx">Excel XLSX</option>
      <option value="marc">MARC (ISO 2709)</option><option value="marcxml">MARCXML</option>
    </SelectField>
    <p className="text-xs text-muted">Catalogue exchange includes bibliographic records and copy details. It does not replace a database backup or migrate loans, members, covers or history. JSON preserves the widest range of catalogue fields. MARC holdings use Koha field 952.</p>
    {output.data && <div>
      <p role="status">Ready: {output.data.records} titles and {output.data.copies} copies.</p>
      {output.data.warnings.length > 0 && <ul className="list-disc pl-5 text-xs text-muted">{output.data.warnings.map((warning) => <li key={warning}>{warning}</li>)}</ul>}
    </div>}
    {save.isSuccess && save.data.path && <p role="status" className="break-all">Saved to {save.data.path}</p>}
    {save.isSuccess && save.data.native && !save.data.path && <p role="status">Save cancelled. Your export is ready to save again.</p>}
    {save.isError && <p role="alert" className="text-danger">{toApiError(save.error).message}</p>}
    {output.isError && <p role="alert" className="text-danger">{toApiError(output.error).message}</p>}
  </Modal>;
}
