// Snapshot the exact core draft resource for spreadsheet review; no app edits.
import fs from 'node:fs';
import crypto from 'node:crypto';
import ts from 'typescript';

const file = 'src/lib/core-translations.ts';
const source = fs.readFileSync(file, 'utf8');
const tree = ts.createSourceFile(file, source, ts.ScriptTarget.Latest, true);
const areas = {
  Dashboard: ['src/features/dashboard/Dashboard.tsx'],
  Catalogue: ['src/features/catalogue/CataloguePage.tsx', 'src/features/catalogue/RecordPage.tsx', 'src/features/catalogue/RecordFormDialog.tsx', 'src/features/catalogue/SavedSearches.tsx', 'src/features/catalogue/shared.tsx'],
  Circulation: ['src/features/circulation/CirculationPage.tsx', 'src/features/circulation/FindBook.tsx', 'src/features/circulation/FindMember.tsx', 'src/features/circulation/OpenLoans.tsx', 'src/features/circulation/OverdueLoans.tsx'],
  Members: ['src/features/members/MembersPage.tsx', 'src/features/members/MemberLoanHistory.tsx'],
  Shared: ['src/components/Pager.tsx'],
};
const contents = Object.fromEntries(Object.entries(areas).map(([area, files]) => [area, files.map(f => fs.readFileSync(f, 'utf8')).join('\n')]));
const rows = [];
function visit(node) {
  if (ts.isPropertyAssignment(node) && ts.isStringLiteral(node.name) && ts.isStringLiteral(node.initializer)) {
    const english = node.name.text;
    const used = Object.entries(contents).filter(([, text]) => text.includes(JSON.stringify(english))).map(([area]) => area);
    rows.push({ id: 'TXT-' + crypto.createHash('sha256').update(english).digest('hex').slice(0, 12), english,
      draft: node.initializer.text, area: used.length ? used.join(' / ') : 'Shared / usage to confirm',
      source: `${file}:${tree.getLineAndCharacterOfPosition(node.getStart(tree)).line + 1}` });
  }
  ts.forEachChild(node, visit);
}
visit(tree);
if (rows.length !== 319 || new Set(rows.map(row => row.id)).size !== 319) throw Error('Core draft snapshot must contain 319 unique phrases.');
const placeholders = text => [...text.matchAll(/\{(\w+)\}/g)].map(m => m[1]).sort().join('|');
for (const row of rows) if (placeholders(row.english) !== placeholders(row.draft)) throw Error(`Placeholder mismatch: ${row.id}`);
rows.sort((a,b) => a.area.localeCompare(b.area) || a.english.localeCompare(b.english));
fs.writeFileSync('docs/translations/core-draft-review.json', JSON.stringify({ at: new Date().toISOString(), scope: '319 core draft resource entries; owner review pending; not complete application coverage', rows }, null, 2) + '\n');
console.log('319 exact draft pairs exported; unique IDs and placeholders checked.');
