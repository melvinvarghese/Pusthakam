import { useTranslation } from "@/lib/translations";
import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Pager } from "@/components/Pager";
import { Button } from "@/components/ui/Button";
import { Modal } from "@/components/ui/Modal";
import { membersApi, toApiError } from "@/lib/api";
import type { Member, MemberLoan } from "@/lib/api/types";

function status(loan: MemberLoan, t: ReturnType<typeof useTranslation>["t"]) {
  if (!loan.returnedOn) return t(loan.overdue ? "Overdue" : "On loan");
  if (loan.returnCondition === "lost") return t("Lost \u00b7 {date}", { date: loan.returnedOn });
  if (loan.returnCondition === "damaged") return t("Returned damaged \u00b7 {date}", { date: loan.returnedOn });
  return t("Returned {date}", { date: loan.returnedOn });
}

export function MemberLoanHistory({ member, onClose }: { member: Member; onClose: () => void }) {
  const { t } = useTranslation();
  const [page, setPage] = useState(1);
  const result = useQuery({
    queryKey: ["member-loans", member.id, page],
    queryFn: () => membersApi.listLoans(member.id, page),
    retry: false,
  });

  return (
    <Modal title={t("Loans for {name}", { name: member.name })} onClose={onClose} wide
      footer={<Button onClick={onClose}>{t("Close")}</Button>}>
      <p className="font-mono text-xs text-muted">{member.memberNumber}</p>
      {result.isPending && <p role="status" className="text-muted">{t("Loading loan history.")}</p>}
      {result.isError && <p role="alert" className="text-danger">{toApiError(result.error).message}</p>}
      {result.data?.total === 0 && <p className="text-muted">{t("This member has no loans.")}</p>}
      {result.data && result.data.total > 0 && (
        <>
          <div className="overflow-x-auto">
            <table className="w-full border-collapse text-left text-sm">
              <caption className="sr-only">{t("Member loan history, open loans first")}</caption>
              <thead className="border-b border-line text-xs text-muted">
                <tr>{["Title", "Issued", "Due", "Status"].map((label) => (
                  <th key={label} scope="col" className="px-2 py-2">{t(label)}</th>
                ))}</tr>
              </thead>
              <tbody>
                {result.data.rows.map((loan) => (
                  <tr key={loan.id} className="border-b border-line last:border-b-0">
                    <td className="px-2 py-2">
                      <div>{loan.title}</div>
                      <div className="font-mono text-xs text-muted">{loan.barcode}</div>
                      {loan.importSource && <div className="text-xs text-muted">{t("Imported from {source}. Source staff label: {staff}.", { source: loan.importSource, staff: loan.sourceStaff ?? "" })}</div>}
                      {Boolean(loan.renewalCount) && <div className="text-xs text-muted">{t("Renewals:")} {loan.renewalCount}</div>}
                    </td>
                    <td className="whitespace-nowrap px-2 py-2">{loan.issuedOn}</td>
                    <td className="whitespace-nowrap px-2 py-2">{loan.dueOn}</td>
                    <td className={`whitespace-nowrap px-2 py-2 ${loan.overdue ? "text-danger" : ""}`}>
                      {status(loan, t)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <Pager page={result.data.page} pageSize={result.data.pageSize} total={result.data.total} onPage={setPage} />
        </>
      )}
    </Modal>
  );
}
