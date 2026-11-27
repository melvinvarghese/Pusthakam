// SYNTHETIC sample data for website screenshots only. shoot.cjs appends this to the app's dev preview
// module (src/features/dev/preview.ts) inside the browser at screenshot time; it never reaches the app.
const ALL_PERMS = ["admin.backup","admin.settings","admin.staff","audit.read","catalogue.read","catalogue.write","circulation.issue","circulation.manage","classification.write","members.read","members.write","reports.read","serials.write","acquisitions.write"];
const SHOT_SESSION = { userId: 1, username: "librarian", displayName: "Librarian", role: "administrator", permissions: ALL_PERMS };
const BOOKS = [
  ["Balyakalasakhi", "Vaikom Muhammad Basheer", "DC Books", 1944, "894.8123", 3, 2],
  ["The God of Small Things", "Arundhati Roy", "IndiaInk", 1997, "823.914", 2, 0],
  ["Wings of Fire", "A. P. J. Abdul Kalam", "Universities Press", 1999, "629.1092", 4, 3],
  ["A Brief History of Time", "Stephen Hawking", "Bantam", 1988, "523.1", 2, 1],
  ["The Discovery of India", "Jawaharlal Nehru", "Penguin", 1946, "954", 3, 3],
  ["Randamoozham", "M. T. Vasudevan Nair", "Current Books", 1984, "894.8123", 2, 1],
  ["Gitanjali", "Rabindranath Tagore", "Macmillan", 1912, "891.441", 1, 1],
  ["The Story of Philosophy", "Will Durant", "Simon & Schuster", 1926, "109", 1, 0],
  ["An Introduction to Language", "Victoria Fromkin", "Cengage", 2013, "410", 2, 2],
  ["Indian Economy", "Ramesh Singh", "McGraw Hill", 2024, "330.954", 5, 4],
];
const ROWS = BOOKS.map(([title, authors, publisher, publicationYear, classificationNumber, totalCopies, availableCopies], i) => ({
  id: i + 1, title, subtitle: null, authors, publisher, publicationYear, isbn13: null, classificationNumber, totalCopies, availableCopies, archived: false }));
function days(from, to) { const out = []; for (let d = new Date(from); d <= new Date(to); d.setDate(d.getDate() + 1)) out.push(d.toISOString().slice(0, 10)); return out; }
function wave(i, base, amp) { const dow = i % 7; return dow === 6 ? 0 : Math.round(base + amp * Math.sin(i * 1.7) + (dow === 0 ? amp : 0)); }
export async function previewCall(command, mode, args) {
  if (globalThis.__LOG) console.log("CMD " + command + " " + JSON.stringify(args || {}).slice(0, 300));
  switch (command) {
    case "current_session": return mode === "login" ? null : SHOT_SESSION;
    case "catalogue_search": return { total: 3184, page: 1, pageSize: 25, rows: ROWS };
    case "dashboard_overview":
      return { catalogue: { titles: 3184, copies: 4920, available: 4651 }, activeMembers: 412,
        circulation: { onLoan: 241, dueToday: 9, overdue: 14, readyHolds: 3, outstandingFines: 2350 } };
    case "reports_circulation_activity":
      return days(args.from, args.to).map((date, i) => ({ date, issued: wave(i, 34, 10), returned: wave(i + 2, 30, 9), lost: 0, renewed: wave(i, 6, 3), reservations: wave(i, 2, 1) }));
    case "reports_catalogue_growth":
      return days(args.from, args.to).map((date, i) => ({ date, titlesAdded: wave(i, 5, 4), copiesAdded: wave(i, 8, 6) }));
    case "reports_popular_titles":
      return ROWS.slice(0, 6).map((r, i) => ({ recordId: r.id, title: r.title, archived: false, issues: 31 - i * 4 }));
    case "circulation_prepare_issue":
      return { memberId: 1, itemId: 1, memberName: "Demo Reader", title: "Balyakalasakhi", dueOn: "2026-12-11" };
    default: return basePreviewCall(command, mode, args);
  }
}
