import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Button } from "@/components/ui/Button";
import { TextField } from "@/components/ui/Field";
import { Modal } from "@/components/ui/Modal";
import { ApiError, circulationApi, toApiError } from "@/lib/api";
import { fileToBase64 } from "@/lib/files";
import { formatMinor } from "@/lib/money";
import type { HistorySummary } from "@/lib/api/types";

function Summary({ summary, label }: {summary:HistorySummary;label:string}) {
  return <section aria-label={label} className="space-y-3"><h3 className="font-semibold">{label}</h3>
    <dl className="grid grid-cols-2 gap-2 text-sm">{Object.entries(summary.counts).map(([name,count])=><div key={name}><dt className="capitalize text-muted">{name}</dt><dd>{count}</dd></div>)}</dl>
    {summary.balances.length>0 ? <div className="overflow-x-auto"><table className="w-full text-left text-sm" aria-label={`${label} balances`}>
      <thead><tr>{["Currency","Assessed","Paid","Waived","Outstanding"].map((v)=><th className="p-2" key={v}>{v}</th>)}</tr></thead>
      <tbody>{summary.balances.map((b)=><tr key={b.currency} className="border-t border-line"><th className="p-2">{b.currency}</th>
        {[b.assessedMinor,b.paidMinor,b.waivedMinor,b.outstandingMinor].map((v,i)=><td className="p-2 font-mono" key={i}>{formatMinor(v,b.currency)}</td>)}
      </tr>)}</tbody></table></div> : <p className="text-sm text-muted">No assessed financial records in this summary.</p>}
  </section>;
}
export default function HistoryImport({onClose}:{onClose:()=>void}) {
  const client=useQueryClient();const [file,setFile]=useState<File|null>(null);
  const recent=useQuery({queryKey:["history-import-batches"],queryFn:circulationApi.historyImports,retry:false});
  const load=useMutation({mutationFn:async(file:File)=>{
    if(file.size>10*1024*1024)throw new ApiError({code:"VALIDATION",message:"History JSON must be 10 MB or smaller."});
    const dataBase64=await fileToBase64(file);return {dataBase64,preview:await circulationApi.previewHistory(dataBase64)};
  }});
  const commit=useMutation({mutationFn:async()=>{
    if(!load.data)throw new Error("Preview history first.");
    return circulationApi.commitHistory(load.data.dataBase64,load.data.preview.fingerprint);
  },onSuccess:()=>{for(const key of ["history-import-batches","circulation","member-fines","member-loans","catalogue","dashboard","reports"])void client.invalidateQueries({queryKey:[key]});}});
  const busy=load.isPending||commit.isPending;const p=load.data?.preview;
  return <Modal title="Import reservations and financial history" wide onClose={()=>{if(!busy)onClose();}}
    footer={<Button disabled={busy} onClick={onClose}>{commit.isSuccess?"Done":"Close"}</Button>}>
    {commit.isSuccess ? <><p role="status">History imported and reconciled. All five parts were saved together.</p><Summary summary={commit.data} label="Saved reconciliation"/></> : <>
      <p>Preview reservations, assessed fines, payments and waivers together. Source counts and currency totals must match before the entire bundle can be saved.</p>
      <p className="text-sm text-muted">Use the neutral history JSON format, after importing members, copies and loans. This is not a raw Koha export. Only one assessed fine per closed imported loan is supported; other charges require explicit mapping.</p>
      <details className="text-sm"><summary>Format and reservation rules</summary><p>Version 1 requires sourceSystem, reservations, fines, payments, waivers and expected counts/balances. Use integer minor units and UTC timestamps. Active queues require explicit copy barcodes and contiguous positions from 1; existing local queues cannot be merged. Ready reservations need an available copy and future expiry. Entirely pending queues need an on-loan copy. Unsupported records are rejected, never skipped.</p></details>
      <TextField label="History bundle JSON" type="file" accept=".json,application/json" disabled={busy} onChange={(e)=>{setFile(e.target.files?.[0]??null);load.reset();commit.reset();}}/>
      <Button disabled={!file||busy} onClick={()=>{if(file){load.reset();commit.reset();load.mutate(file);}}}>{load.isPending?"Checking history…":"Preview history"}</Button>
      {load.error&&<p role="alert" className="text-danger">{toApiError(load.error).message}</p>}
      {p&&<>
        <p className="text-sm">Source: {p.sourceSystem}</p>
        <Summary summary={p.summary} label="Calculated destination"/><Summary summary={p.expected} label="Expected source"/>
        <p className="text-sm text-muted">Balances cover all affected fines, including earlier settlements, after this batch. Currencies are never combined. Original staff labels are retained; your account records the import. Closed reservation history does not change stock.</p>
        {p.problems.length>0&&<ul role="alert" className="list-disc pl-5 text-danger">{p.problems.map((v)=><li key={v}>{v}</li>)}</ul>}
        {p.canCommit&&<p role="status">Source counts and balances match. Confirm to import all records together.</p>}
        {commit.error&&<p role="alert" className="text-danger">{toApiError(commit.error).message} Nothing was imported.</p>}
        <Button variant="primary" disabled={busy||!p.canCommit} onClick={()=>commit.mutate()}>{commit.isPending?"Importing history…":"Confirm import history"}</Button>
      </>}
    </>}
    {recent.isError&&<p role="alert">{toApiError(recent.error).message}</p>}
    {Boolean(recent.data?.length)&&<details><summary>Recent saved reconciliations (latest 20)</summary><div className="space-y-4 pt-3">
      {recent.data?.map((batch)=><div key={batch.id}><p className="text-sm text-muted">{batch.sourceSystem} · {new Date(batch.importedAt).toLocaleString()}</p><Summary summary={batch.summary} label={`Saved batch #${batch.id}`}/></div>)}
    </div></details>}
  </Modal>;
}
