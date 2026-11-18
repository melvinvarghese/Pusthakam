import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Button } from "@/components/ui/Button";
import { TextField } from "@/components/ui/Field";
import { Modal } from "@/components/ui/Modal";
import { ApiError, circulationApi, toApiError } from "@/lib/api";
import { fileToBase64 } from "@/lib/files";

export default function RenewalImport({ onClose }: { onClose: () => void }) {
  const client=useQueryClient();
  const [file,setFile]=useState<File | null>(null);
  const [source,setSource]=useState("");
  const load=useMutation({mutationFn:async ({file,source}:{file:File;source:string}) => {
    if(file.size>10*1024*1024) throw new ApiError({code:"VALIDATION",message:"Renewal CSV must be 10 MB or smaller."});
    const dataBase64=await fileToBase64(file);
    return {dataBase64,source,preview:await circulationApi.previewRenewalImport(dataBase64,source)};
  }});
  const commit=useMutation({mutationFn:async () => {
    if(!load.data) throw new Error("Preview the CSV first.");
    return circulationApi.commitRenewalImport(load.data.dataBase64,load.data.source,load.data.preview.fingerprint);
  },onSuccess:()=>{ for(const key of ["member-loans","circulation","reports"]) void client.invalidateQueries({queryKey:[key]}); }});
  const busy=load.isPending||commit.isPending;
  const p=load.data?.preview;
  return <Modal title="Import renewal history" wide onClose={()=>{if(!busy)onClose();}}
    footer={<Button disabled={busy} onClick={onClose}>Close</Button>}>
    {commit.isSuccess ? <p role="status">Imported {commit.data} renewal events. Current loan dates and stock were unchanged.</p> : <>
      <p>Import loans first, then their complete renewal chains using the same source name. This imports historical events and retains the loan's final due date. It does not perform a new renewal.</p>
      <details className="text-sm"><summary>Required CSV format</summary>
        <p className="font-mono break-words">source_id,loan_source_id,renewed_at,previous_due_on,due_on,source_staff</p>
        <p>UTF-8; maximum 10 MB and 25,000 events. UTC timestamps: YYYY-MM-DDTHH:MM:SSZ. Dates: YYYY-MM-DD. Use one complete chain per loan ending at its imported due date. Existing renewal events block importing that loan's history.</p>
      </details>
      <TextField label="Source system or library" value={source} maxLength={128} disabled={busy}
        onChange={(e)=>{setSource(e.target.value);load.reset();commit.reset();}} />
      <TextField label="Renewal history CSV" type="file" accept=".csv,text/csv" disabled={busy}
        onChange={(e)=>{setFile(e.target.files?.[0]??null);load.reset();commit.reset();}} />
      <Button disabled={busy||!file||!source.trim()} onClick={()=>{if(file){commit.reset();load.reset();load.mutate({file,source:source.trim()});}}}>
        {load.isPending?"Checking CSV…":"Preview renewals"}
      </Button>
      {load.error&&<p role="alert" className="text-danger">{toApiError(load.error).message}</p>}
      {p&&<>
        <p role="status">{p.total} renewal events for {p.loans} loans; {p.invalid} invalid.</p>
        {p.total===0&&<p>The file contains no renewal events.</p>}
        {p.problems.length>0&&<ul className="list-disc pl-5 text-sm" aria-label="Renewal problems">{p.problems.map((v)=><li key={v}>{v}</li>)}</ul>}
        <p className="text-sm text-muted">All events are saved together. Original source staff labels are preserved; your account records the import. Imported events count toward future renewal limits.</p>
        {commit.error&&<p role="alert" className="text-danger">{toApiError(commit.error).message} Nothing was imported.</p>}
        <Button variant="primary" disabled={busy||p.total===0||p.invalid>0} onClick={()=>commit.mutate()}>
          {commit.isPending?"Importing renewals…":"Confirm import renewals"}
        </Button>
      </>}
    </>}
  </Modal>;
}
