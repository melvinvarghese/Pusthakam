import { useTranslation } from "@/lib/translations";
import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Pager } from "@/components/Pager";
import { Button } from "@/components/ui/Button";
import { Panel } from "@/components/ui/Panel";
import { circulationApi, toApiError } from "@/lib/api";
import type { OverdueLoan } from "@/lib/api/types";

export function OverdueLoans({ onReturn, busy }: { onReturn: (loan: OverdueLoan) => void; busy: boolean }) {
  const { t } = useTranslation();
  const [page, setPage] = useState(1);
  const result = useQuery({
    queryKey: ["circulation", "overdue", page],
    queryFn: () => circulationApi.listOverdue(page),
    retry: false,
    refetchInterval: 60_000,
  });

  if (result.isPending) return <p role="status" className="text-muted">{t("Loading overdue loans.")}</p>;
  if (result.isError) return <p role="alert" className="text-danger">{toApiError(result.error).message}</p>;

  const { rows, total, page: currentPage, pageSize } = result.data;
  if (total === 0) return <Panel className="p-6 text-muted">{t("No overdue loans.")}</Panel>;

  return (
    <Panel>
      <div className="overflow-x-auto">
        <table className="w-full border-collapse text-left text-sm">
          <caption className="sr-only">{t("Overdue loans, oldest due first")}</caption>
          <thead className="border-b border-line text-xs text-muted">
            <tr>{["Title", "Borrower", "Due date", "Days overdue", "Action"].map((label) => (
              <th key={label} scope="col" className="px-4 py-2">{t(label)}</th>
            ))}</tr>
          </thead>
          <tbody>
            {rows.map((loan) => (
              <tr key={loan.loanId} className="border-b border-line last:border-b-0">
                <td className="px-4 py-2">
                  <div>{loan.title}</div>
                  <div className="font-mono text-xs text-muted">{loan.barcode}</div>
                </td>
                <td className="px-4 py-2">
                  <div>{loan.memberName}</div>
                  <div className="font-mono text-xs text-muted">{loan.memberNumber}</div>
                </td>
                <td className="whitespace-nowrap px-4 py-2">{loan.dueOn}</td>
                <td className="px-4 py-2 font-medium text-danger">{loan.daysOverdue}</td>
                <td className="px-4 py-2">
                  <Button size="sm" variant="ghost" disabled={busy} aria-label={t("Return copy {barcode}", { barcode: loan.barcode })} onClick={() => onReturn(loan)}>
                    {t("Return")}</Button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <Pager total={total} page={currentPage} pageSize={pageSize} onPage={setPage} />
    </Panel>
  );
}
