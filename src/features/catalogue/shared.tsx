import { useEffect, useRef, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import JsBarcode from "jsbarcode";
import { Chip } from "@/components/ui/Chip";
import { configApi } from "@/lib/api";
import type { CopyStatus } from "@/lib/api/types";

/** The configuration lists (categories, collections, locations, shelves), shared by every catalogue screen. */
export function useConfig() {
  return useQuery({ queryKey: ["config"], queryFn: configApi.overview, retry: false });
}

/** Returns `value` after it has stopped changing for `ms`. Used so searching runs as the user pauses typing. */
export function useDebounced<T>(value: T, ms: number): T {
  const [v, setV] = useState(value);
  useEffect(() => {
    const t = setTimeout(() => setV(value), ms);
    return () => clearTimeout(t);
  }, [value, ms]);
  return v;
}

export const STATUS_LABELS: Record<CopyStatus, string> = {
  available: "Available", on_loan: "On loan", reserved_hold: "Reserved", in_repair: "In repair",
  lost: "Lost", damaged: "Damaged", withdrawn: "Withdrawn", in_transit: "In transit",
};

/** Statuses staff may choose by hand. On loan and Reserved are set by issuing and returning. */
export const MANUAL_STATUSES: CopyStatus[] = ["available", "in_repair", "damaged", "lost", "withdrawn", "in_transit"];

export function StatusChip({ status }: { status: CopyStatus }) {
  return <Chip>{STATUS_LABELS[status]}</Chip>;
}

export const SOURCE_LABELS = { purchase: "Purchase", donation: "Donation", transfer: "Transfer", exchange: "Exchange", other: "Other" } as const;

export const ROLE_LABELS = {
  author: "Author", additional_author: "Additional author", editor: "Editor", translator: "Translator",
  illustrator: "Illustrator", other: "Other",
} as const;

/** Code 128 barcode drawn locally (no network). */
export function BarcodeSvg({ value, height = 44, quietZone = 0, displayValue = true }: {
  value: string; height?: number; quietZone?: number; displayValue?: boolean;
}) {
  const ref = useRef<SVGSVGElement>(null);
  useEffect(() => {
    if (!ref.current) return;
    try {
      JsBarcode(ref.current, value, {
        format: "CODE128", displayValue, height, margin: 0, marginLeft: quietZone, marginRight: quietZone,
        fontSize: 12, font: "Courier Prime",
      });
    } catch {
      /* a value Code 128 cannot encode simply shows no bars */
    }
  }, [value, height, quietZone, displayValue]);
  return <svg ref={ref} role="img" aria-label={`Barcode ${value}`} />;
}
