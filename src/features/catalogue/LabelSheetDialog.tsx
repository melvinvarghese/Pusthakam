import { useEffect, useRef, useState } from "react";
import { SelectField } from "@/components/ui/Field";
import { Button } from "@/components/ui/Button";
import { Modal } from "@/components/ui/Modal";
import type { CopyRow } from "@/lib/api/types";
import { BarcodeSvg } from "./shared";
import "./LabelSheet.css";
import { useLabelHeading } from "./LabelHeading";

export function LabelSheetDialog({ copies, onClose }: { copies: CopyRow[]; onClose: () => void }) {
  const [style, setStyle] = useState("barcode");
  const { heading, field } = useLabelHeading();
  const sheet = useRef<HTMLDivElement>(null);
  const printCleanup = useRef<(() => void) | null>(null);
  useEffect(() => () => printCleanup.current?.(), []);

  function printSheet() {
    if (!sheet.current) return;
    printCleanup.current?.();
    const printable = sheet.current.cloneNode(true) as HTMLDivElement;
    printable.classList.add("label-sheet-print");
    const cleanup = () => {
      printable.remove();
      window.removeEventListener("afterprint", cleanup);
      printCleanup.current = null;
    };
    printCleanup.current = cleanup;
    document.body.append(printable);
    window.addEventListener("afterprint", cleanup, { once: true });
    window.print();
  }

  return (
    <Modal title="Copy label sheet" onClose={onClose} wide footer={<>
      <Button onClick={onClose}>Close</Button>
      <Button variant="primary" onClick={printSheet}>Print labels</Button>
    </>}>
      <p className="text-sm text-muted">{copies.length} {copies.length === 1 ? "label" : "labels"}. Use plain paper or full-sheet label stock.</p>
      <SelectField label="Label style" value={style} onChange={(e) => setStyle(e.target.value)}><option value="barcode">Barcode and call number</option><option value="spine">Spine label</option></SelectField>
      {field}
      <div className="max-h-[50vh] overflow-auto">
        <div ref={sheet} className="label-sheet">
          {copies.map((copy) => (
            <div key={copy.id} className="copy-label">
              {heading && <div className="break-words font-semibold">{heading}</div>}
              {style === "spine" ? <><div className="font-mono text-lg whitespace-pre-line">{copy.callNumber || "No call number"}</div><div className="font-mono">{copy.accessionNo}</div></> : <>
              <div className="font-semibold">{copy.title}</div>
              {copy.callNumber && <div className="font-mono">{copy.callNumber}</div>}
              <div className="font-mono">Accession {copy.accessionNo}</div>
              {/^[\x20-\x7E]+$/.test(copy.barcode)
                ? <>
                  <BarcodeSvg value={copy.barcode} height={32} quietZone={20} displayValue={false} />
                  {copy.barcode !== copy.accessionNo && <div className="font-mono">{copy.barcode}</div>}
                </>
                : <div className="font-mono">Barcode {copy.barcode} (text only)</div>}
              </>}
            </div>
          ))}
        </div>
      </div>
    </Modal>
  );
}
