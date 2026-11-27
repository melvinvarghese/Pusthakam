// Review-only export. Does not install drafts into the application.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import ts from 'typescript';

const pairs = JSON.parse(fs.readFileSync('docs/translations/additional-draft-pairs.json', 'utf8'));
const existing = new Set();
for (const file of ['src/lib/core-translations.ts', 'src/lib/setup-translations.ts', 'src/lib/translations.ts']) {
  const tree = ts.createSourceFile(file, fs.readFileSync(file, 'utf8'), ts.ScriptTarget.Latest, true);
  function visit(node) {
    if (ts.isPropertyAssignment(node) && ts.isStringLiteral(node.name)) existing.add(node.name.text);
    ts.forEachChild(node, visit);
  }
  visit(tree);
}
for (const key of Object.keys(JSON.parse(fs.readFileSync('docs/translations/approved-glossary.json', 'utf8')).terms)) existing.add(key);

function walk(dir) {
  return fs.readdirSync(dir, { withFileTypes: true }).flatMap(entry => entry.isDirectory() ? walk(path.join(dir, entry.name)) : [path.join(dir, entry.name)]);
}
const references = new Map();
function remember(phrase, file, line) {
  phrase = phrase.replace(/\s+/g, ' ').trim();
  const refs = references.get(phrase) ?? [];
  refs.push(`${file.replaceAll('\\', '/')}:${line}`);
  references.set(phrase, refs);
}
for (const file of [...walk('src/features'), 'src-tauri/src/services/opac/catalogue.js'].filter(file => /\.(tsx|js)$/.test(file) && !/\.test\.|[\\/]dev[\\/]/.test(file))) {
  const tree = ts.createSourceFile(file, fs.readFileSync(file, 'utf8'), ts.ScriptTarget.Latest, true, file.endsWith('.tsx') ? ts.ScriptKind.TSX : ts.ScriptKind.JS);
  function visit(node) {
    if (ts.isStringLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node) || ts.isJsxText(node)) {
      remember(node.text, file, tree.getLineAndCharacterOfPosition(node.getStart(tree)).line + 1);
    }
    ts.forEachChild(node, visit);
  }
  visit(tree);
}
const htmlFile = 'src-tauri/src/services/opac/index.html';
const html = fs.readFileSync(htmlFile, 'utf8');
for (const match of html.matchAll(/>([^<>]+)</g)) remember(match[1], htmlFile, html.slice(0, match.index).split('\n').length);
for (const match of html.matchAll(/(?:placeholder|aria-label|alt)="([^"]+)"/g)) remember(match[1], htmlFile, html.slice(0, match.index).split('\n').length);

const placeholders = text => [...text.matchAll(/\{(\w+)\}/g)].map(match => match[1]).sort().join('|');
const rows = [];
for (const [area, entries] of Object.entries(pairs)) for (const [english, draft] of Object.entries(entries)) {
  if (existing.has(english)) throw Error(`Already translated: ${english}`);
  if (!references.has(english)) throw Error(`Not found as current source phrase: ${english}`);
  if (placeholders(english) !== placeholders(draft)) throw Error(`Placeholder mismatch: ${english}`);
  if (!/[\u0d00-\u0d7f]/.test(draft)) throw Error(`Missing Malayalam script: ${english}`);
  rows.push({ id: 'TXT-' + crypto.createHash('sha256').update(english).digest('hex').slice(0, 12), english, draft, area,
    source: [...new Set(references.get(english))].join('\n'), batch: 'New 200', status: 'draft-not-integrated' });
}
if (rows.length !== 200 || new Set(rows.map(row => row.id)).size !== 200) throw Error(`Expected 200 unique additions; got ${rows.length}`);
fs.writeFileSync('docs/translations/additional-draft-review.json', JSON.stringify({ at: new Date().toISOString(), scope: '200 new source-verified Malayalam review drafts; not integrated into application', rows }, null, 2) + '\n');
console.log('200 unique new drafts verified against current source; Malayalam script and placeholders checked.');
