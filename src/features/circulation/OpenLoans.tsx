import { useTranslation } from "@/lib/translations";
import { useEffect, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Pager } from "@/components/Pager";
import { Button } from "@/components/ui/Button";
import { TextField } from "@/components/ui/Field";
import { Panel } from "@/components/ui/Panel";
import { circulationApi, toApiError } from "@/lib/api";

export function OpenLoans({ action, busy, onSelect }: { action: "return" | "renew"; busy: boolean; onSelect: (barcode: string) => void }) {
  const { t } = useTranslation();
  const [text, setText] = useState("");
  const [search, setSearch] = useState("");
  const [page, setPage] = useState(1);
  useEffect(() => { const timer = setTimeout(() => setSearch(text.trim()), 200); return () => clearTimeout(timer); }, [text]);
  const result = useQuery({ queryKey: ["circulation", "open-loans", search, page], queryFn: () => circulationApi.listOpenLoans(search, page), retry: false, refetchInterval: 60_000 });
  const current = search === text.trim();
  return <Panel className="space-y-3 p-4">
    <h2 className="font-semibold">{t("Currently issued books")}</h2>
    <TextField label={t("Search current loans")} value={text} onChange={(event) => { setText(event.target.value); setPage(1); }} hint={t("Search by book, member, barcode or accession number.")} />
    {(!current || result.isPending) && <p role="status">{t("Loading current loans.")}</p>}
    {current && result.isError && <div role="alert">{toApiError(result.error).message} <Button onClick={() => void result.refetch()}>{t("Retry loans")}</Button></div>}
    {current && result.data && <>
      {result.data.total === 0 ? <p className="text-muted">{search ? t("No matching current loans.") : t("No currently issued books.")}</p> : <div className="overflow-x-auto"><table className="w-full text-left text-sm">
        <caption className="sr-only">{t("Current loans, earliest due first")}</caption>
        <thead><tr>{["Book / Copy", "Member", "Due date", "Action"].map(label => <th key={label} scope="col" className="px-3 py-2">{t(label)}</th>)}</tr></thead>
        <tbody>{result.data.rows.map(loan => <tr key={loan.loanId} className="border-t border-line">
          <td className="px-3 py-2">{loan.title}<div className="font-mono text-xs text-muted">{loan.accessionNo} / {loan.barcode}</div></td>
          <td className="px-3 py-2">{loan.memberName}<div className="text-xs text-muted">{loan.memberNumber}</div></td>
          <td className="whitespace-nowrap px-3 py-2">{loan.dueOn}</td>
          <td className="px-3 py-2"><Button size="sm" disabled={busy || result.isFetching} aria-label={t(action === "return" ? "Review return for copy {barcode}" : "Review renew for copy {barcode}", { barcode: loan.barcode })} onClick={() => onSelect(loan.barcode)}>{action === "return" ? t("Review return") : t("Review renewal")}</Button></td>
        </tr>)}</tbody>
      </table></div>}
      <Pager page={result.data.page} pageSize={result.data.pageSize} total={result.data.total} onPage={setPage} />
    </>}
  </Panel>;
}
