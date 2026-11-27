import type { OverdueLoan } from "@/lib/api/types";

export function buildOverdueReminderMessage(loan: OverdueLoan, libraryName?: string): string {
  const lib = libraryName?.trim() || "the library";
  const days = loan.daysOverdue === 1 ? "1 day" : `${loan.daysOverdue} days`;
  return `Hello ${loan.memberName}, this is a reminder from ${lib} that "${loan.title}" (Copy ${loan.barcode}) was due on ${loan.dueOn} (${days} overdue). Please return or renew it when you visit next. Thank you!`;
}

export function buildMailtoUrl(loan: OverdueLoan, libraryName?: string): string | null {
  const email = loan.memberEmail?.trim();
  if (!email || !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) return null;
  const subject = encodeURIComponent(`Library overdue reminder: ${loan.title}`);
  const body = encodeURIComponent(buildOverdueReminderMessage(loan, libraryName));
  return `mailto:${email}?subject=${subject}&body=${body}`;
}

export function buildWhatsAppUrl(loan: OverdueLoan, libraryName?: string): string | null {
  const digits = (loan.memberPhone ?? "").replace(/\D/g, "");
  if (digits.length < 7) return null;
  const text = encodeURIComponent(buildOverdueReminderMessage(loan, libraryName));
  return `https://wa.me/${digits}?text=${text}`;
}
