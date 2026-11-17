import type { CSSProperties } from "react";
import { parseCallNumber } from "@/lib/callNumber";

interface Props {
  /** Full call number, e.g. "894.8123 BAS". */
  callNumber: string;
  size?: "md" | "lg";
  schemeCode?: string | null;
}

/** The product's signature element: a stacked call-number label with a Dewey class colour band. */
export function CallNumberLabel({ callNumber, size = "md", schemeCode }: Props) {
  const { lines, deweyClass } = parseCallNumber(callNumber, schemeCode == null || schemeCode === "ddc");
  const style = (deweyClass === null ? undefined : { "--cls": `var(--cls-${deweyClass})` }) as CSSProperties | undefined;
  return (
    <span className="call-label" data-size={size} style={style} role="img" aria-label={`Call number ${callNumber}`}>
      {lines.map((line, i) => (
        <span key={i} aria-hidden="true">{line}</span>
      ))}
    </span>
  );
}
