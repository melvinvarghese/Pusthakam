# Member fine accounts

In Members, choose **Fines** beside a member. The read-only account is available for active and archived members to staff with member-read permission.

The account lists assessed fines, including fully settled fines. Each row shows the title, copy barcode, assessment time, assessed amount, payments, waivers and outstanding balance. Totals cover every page and are separated by currency; different currencies are never added together. An account with no assessed fines has an explicit empty state.

Choose **View** in a fine's History column to see its payments and waivers, methods, recording times and staff names. Waiver reasons are returned only to staff with circulation-management permission. The history command verifies that the requested fine belongs to the selected member. Use **Back to fines** to return to the account without closing the dialog.

Payments and waivers remain in Circulation with their existing permission and balance-conflict checks. The member view adds no financial write actions. New assessments and settlements invalidate the member account queries so reopened views refresh from the ledger.

Only assessed fines are included. Current overdue estimates may become assessed fines when copies are returned. This view does not change fine arithmetic, issue-time terms, due dates or settlement history. No schema migration is required.

The backend uses the same balance calculation as the circulation ledger and reads account rows, counts and totals from one database snapshot. Amounts stay in integer minor units, with safe-range checks before they reach the UI.

## Windows acceptance

1. Quit the older test build from its tray menu before opening the refreshed executable. Clicking X only hides a tray-enabled build.
2. Open Members and choose **Fines** for a member. Check either their account or the explicit no-fines message.
3. For a member with an assessed fine, compare the assessed, paid, waived and outstanding values against Circulation → Fines. A partially paid fine must show its remaining balance.
4. Open the settlement history and check payment/waiver entries. Use Back to fines. Settled fines should remain in the account.
5. Check an archived member and a read-only staff account. Reading is allowed with member-read permission; recording payments or waivers is not offered in this dialog.

Automated reconciliation includes partial payment, waiver, final settlement, multiple currencies, pagination, another member's fine exclusion, archived-member access, reason redaction and denied read/write actions.

Owner Windows result (18 November 2026): account opens and displays the expected no-assessed-fines message. No overdue copy is available. This confirms the empty state only; populated balances/history and archived/read-only native checks remain open.
