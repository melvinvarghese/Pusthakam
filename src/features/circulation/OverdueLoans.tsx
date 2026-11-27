import { useTranslation } from "@/lib/translations";
import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Pager } from "@/components/Pager";
import { Button } from "@/components/ui/Button";
import { Panel } from "@/components/ui/Panel";
import { circulationApi, isDesktop, openExternal, toApiError } from "@/lib/api";
import { openUrl } from "@tauri-apps/plugin-opener";
import { buildMailtoUrl, buildOverdueReminderMessage, buildWhatsAppUrl } from "@/lib/reminders";
import type { ConfigOverview, OverdueLoan } from "@/lib/api/types";

export function OverdueLoans({ onReturn, busy }: { onReturn: (loan: OverdueLoan) => void; busy: boolean }) {
  const { t } = useTranslation();
  const [page, setPage] = useState(1);
  const [notice, setNotice] = useState<string | null>(null);
  const [noticeError, setNoticeError] = useState<string | null>(null);
  const qc = useQueryClient();
  const libraryName = qc.getQueryData<ConfigOverview>(["config"])?.library?.name;
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

  const sendReminder = async (url: string, channel: string, memberName: string) => {
    setNotice(null); setNoticeError(null);
    try {
      if (url.startsWith("mailto:")) {
        if (isDesktop()) await openUrl(url);
        else window.open(url, "_self");
      } else {
        await openExternal(url);
      }
      setNotice(t("Opened {channel} reminder for {member}.", { channel, member: memberName }));
    } catch (e) {
      setNoticeError(toApiError(e).message);
    }
  };
  const copyReminder = async (loan: OverdueLoan) => {
    setNotice(null); setNoticeError(null);
    try {
      await navigator.clipboard.writeText(buildOverdueReminderMessage(loan, libraryName));
      setNotice(t("Copied reminder message for {member}.", { member: loan.memberName }));
    } catch {
      setNoticeError(t("Could not copy the reminder to clipboard."));
    }
  };

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
                  <div className="flex flex-wrap items-center gap-1.5">
                    <Button size="sm" variant="ghost" disabled={busy} aria-label={t("Return copy {barcode}", { barcode: loan.barcode })} onClick={() => onReturn(loan)}>
                      {t("Return")}</Button>
                    {(() => {
                      const mailUrl = buildMailtoUrl(loan, libraryName);
                      return mailUrl ? (
                        <Button size="sm" variant="ghost" disabled={busy} aria-label={t("Email reminder to {member}", { member: loan.memberName })} onClick={() => void sendReminder(mailUrl, "email", loan.memberName)}>
                          {t("Email")}</Button>
                      ) : null;
                    })()}
                    {(() => {
                      const waUrl = buildWhatsAppUrl(loan, libraryName);
                      return waUrl ? (
                        <Button size="sm" variant="ghost" disabled={busy} aria-label={t("WhatsApp reminder to {member}", { member: loan.memberName })} onClick={() => void sendReminder(waUrl, "WhatsApp", loan.memberName)}>
                          {t("WhatsApp")}</Button>
                      ) : null;
                    })()}
                    <Button size="sm" variant="ghost" disabled={busy} aria-label={t("Copy reminder for {member}", { member: loan.memberName })} onClick={() => void copyReminder(loan)}>
                      {t("Copy notice")}</Button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {notice && <p role="status" className="px-4 py-2 text-xs text-ok">{notice}</p>}
      {noticeError && <p role="alert" className="px-4 py-2 text-xs text-danger">{noticeError}</p>}
      <Pager total={total} page={currentPage} pageSize={pageSize} onPage={setPage} />
    </Panel>
  );
}
