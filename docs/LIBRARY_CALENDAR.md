# Library calendar

Library Settings → Library calendar lets administrators manage recurring closed weekdays and dated holidays. Other signed-in staff can view the calendar. Changes are validated in Rust and recorded in the audit log in the same transaction as the calendar change.

Select closed weekdays and choose **Save closed days**. At least one weekday must remain open. No weekdays are closed by default, preserving existing behavior after upgrade.

Use **Add holiday** for a real date and a short name. Existing holidays can be edited or removed; removal requires confirmation. Duplicate dates are rejected. Calendar data remains available after restarting the app.

For issue, the loan rule first adds its calendar-day loan period to today. For renewal, the renewal period is added to the current due date. If that result falls on a holiday or closed weekday, the due date advances until it reaches an open date. Consecutive holidays and weekly closures are combined. These are calendar-day loan periods, not counts of open days.

Changes apply when calculating a new issue or renewal. They do not rewrite existing loans' due dates, alter fine assessment rules or prevent staff from working on a closed date. If the calendar changes after an issue/renewal preview, confirming the stale due date fails and staff must review the updated preview.

Schema 17 adds the closed-weekday table; existing holiday records remain intact. Normal startup backup-before-upgrade rules apply.

## Windows check

Close older verification apps before launching the new build, because it upgrades the disposable library to schema 17.

1. Open Library Settings and find **Library calendar**. Select Sunday and save.
2. Add a holiday named **Test closure** on a future date. Edit its name, restart the app, and check that the Sunday setting and edited holiday remain.
3. In Circulation, preview an issue for an available copy and eligible member. Add that proposed due date as another holiday. Preview again: the date must move forward past all closed days.
4. Issue the copy. Preview renewal, add its proposed date as a holiday, and check the refreshed renewal date advances too. Existing saved due dates must stay unchanged until an issue/renewal is confirmed.
5. Remove test holidays with confirmation and restore the original weekly settings. Read-only staff should see the calendar but have no editing actions.
