# Desktop window and member lookup

## Window behavior

Pusthakam starts maximized, filling the working screen while retaining the normal title bar and window controls.

The main window's X button hides it in the notification-area/system tray while the application process keeps running. Click the Pusthakam tray icon or right-click it and choose **Open Pusthakam** to return to the same screen and signed-in session. The icon may be inside Windows' hidden-icons arrow.

Choose **Quit** from the tray menu to exit fully. A full quit, restart or computer shutdown ends the in-memory session and requires normal sign-in next time. Passwords are not saved or replayed. Use Sign out when you want to end the current staff session. Hidden sessions still use the existing backend permission/session checks.

If the tray cannot be created, ordinary window closing stays enabled so the app cannot become hidden without a way to restore it. Reopening through the tray is supported; launching a second executable process is not a single-instance restore feature.

## Find member by name

In Circulation's Issue form, choose **Find member by name**. Matching active member records appear in a dropdown as you type; no Search click is required. The same picker is now available in Reservations. Staff need member-read permission to see either lookup action. Direct member-number entry remains available.

The search trims whitespace, waits briefly while typing, requests bounded result pages and shows the member number alongside each name. Identical names remain distinguishable. Results for an old search cannot be selected after the typed name changes. Use the mouse or Down/Up arrows followed by Enter to choose a result. Escape closes the picker and restores focus; choosing a member fills their number and focuses the book barcode/accession field. More than one results page has Previous/Next controls. Search errors can be retried and no-match results are explicit.

This does not bypass membership, expiry, loan-limit or reservation rules: the existing backend confirmation and validation paths still decide eligibility.

## Windows checks

1. Open the new verification executable and confirm the window starts maximized.
2. Sign in, click X, then restore through the Pusthakam tray icon. The same session and page should remain.
3. In Circulation, type part of a member's name in the picker. Choose a match and check the populated member number. Repeat in Reservations, including keyboard selection.
4. Right-click the tray icon and choose Quit. Reopen the executable and confirm normal sign-in is required.

The minimum three-second startup loading screen remains deferred and is not implemented here.
