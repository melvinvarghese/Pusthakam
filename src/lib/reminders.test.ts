import { describe, expect, it } from "vitest";
import { buildMailtoUrl, buildOverdueReminderMessage, buildWhatsAppUrl } from "./reminders";
import type { OverdueLoan } from "./api/types";

const sampleLoan: OverdueLoan = {
  loanId: 12,
  memberName: "Ananya Nair",
  memberNumber: "M-101",
  memberEmail: "ananya@example.com",
  memberPhone: "+91 98765 43210",
  title: "Balyakalasakhi",
  barcode: "BK-001",
  dueOn: "2026-11-20",
  daysOverdue: 7,
};

describe("overdue reminders", () => {
  it("formats a polite overdue reminder message", () => {
    const msg = buildOverdueReminderMessage(sampleLoan, "Town Library");
    expect(msg).toContain("Ananya Nair");
    expect(msg).toContain("Town Library");
    expect(msg).toContain("Balyakalasakhi");
    expect(msg).toContain("BK-001");
    expect(msg).toContain("2026-11-20");
    expect(msg).toContain("7 days overdue");
  });

  it("builds mailto and WhatsApp links when valid contact info exists", () => {
    const mail = buildMailtoUrl(sampleLoan, "Town Library");
    expect(mail).toMatch(/^mailto:ananya@example\.com\?subject=/);

    const wa = buildWhatsAppUrl(sampleLoan, "Town Library");
    expect(wa).toMatch(/^https:\/\/wa\.me\/919876543210\?text=/);
  });

  it("returns null when email or phone is missing/invalid", () => {
    expect(buildMailtoUrl({ ...sampleLoan, memberEmail: null })).toBeNull();
    expect(buildWhatsAppUrl({ ...sampleLoan, memberPhone: "123" })).toBeNull();
  });
});
