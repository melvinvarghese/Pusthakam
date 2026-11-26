import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import ts from 'typescript';

// Stable IDs identify wording, independently of worksheet order.
const rows = new Map();
function add(text, file, line, context, draft = '') {
  text = text.replace(/\s+/g, ' ').trim();
  if (!/[A-Za-z]/.test(text) || text.length < 2 || text.length > 1800) return;
  if (/^(https?:|\/|@|data:|#[\da-f]{3,8}$)/i.test(text)) return;
  if (/^(?:input:|\(min-width:|PO-\{|\{new Date\(|\{r\.classificationNumber\}|\{r\.isbn13\})/.test(text)) return;
  const id = 'TXT-' + crypto.createHash('sha256').update(text).digest('hex').slice(0, 12);
  const row = rows.get(text) ?? { id, english: text, malayalam: '', draft, contexts: new Set(), sources: new Set() };
  row.contexts.add(context);
  row.sources.add(`${file.replaceAll('\\', '/')}:${line}`);
  if (draft) row.draft = draft;
  rows.set(text, row);
}
function walk(dir) {
  return fs.readdirSync(dir, { withFileTypes: true }).flatMap(e => e.isDirectory() ? walk(path.join(dir, e.name)) : [path.join(dir, e.name)]);
}
const visibleProps = /^(label|title|description|hint|placeholder|summary|message|text|heading|emptyText|confirmLabel|cancelLabel|name)$/;
for (const file of walk('src').filter(f => /\.[tj]sx?$/.test(f) && !/\.test\.|[\\/]dev[\\/]|[\\/]test[\\/]|setup-translations/.test(f))) {
  const source = fs.readFileSync(file, 'utf8');
  const tree = ts.createSourceFile(file, source, ts.ScriptTarget.Latest, true, file.endsWith('tsx') ? ts.ScriptKind.TSX : ts.ScriptKind.TS);
  function visit(node) {
    const line = tree.getLineAndCharacterOfPosition(node.getStart(tree)).line + 1;
    const area = file.replaceAll('\\', '/').split('/').slice(1, -1).join(' / ');
    if (ts.isJsxText(node)) add(node.text, file, line, `${area}: screen text`);
    if (ts.isStringLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node) || ts.isTemplateExpression(node)) {
      const p = node.parent;
      const attr = ts.isJsxAttribute(p) ? p.name.getText(tree) : '';
      const prop = ts.isPropertyAssignment(p) ? p.name.getText(tree).replace(/["']/g, '') : '';
      const technical = attr && !/^(label|title|hint|placeholder|alt|aria-label|description|emptyText)$/.test(attr);
      const text = ts.isTemplateExpression(node)
        ? node.head.text + node.templateSpans.map(s => `{${s.expression.getText(tree)}}${s.literal.text}`).join('')
        : node.text;
      const near = p.getText(tree);
      const visible = attr || visibleProps.test(prop) || ts.isJsxExpression(p) ||
        (/\s/.test(text) && !/^(import|export)\b/.test(near) && !ts.isImportDeclaration(p));
      if (visible && !technical && (!/^[a-z\d_.:/@-]+$/.test(text) || visibleProps.test(prop) || !!attr) &&
          !/(?:\b(?:flex|grid|text-|bg-|px-|py-|rounded-|items-|justify-)|SELECT |INSERT |UPDATE |DELETE |CREATE TABLE|\{\s*"|<svg)/.test(text)) {
        add(text, file, line, `${area}: ${attr || prop || 'message / option'}`);
      }
    }
    ts.forEachChild(node, visit);
  }
  visit(tree);
}
// Include reviewed setup wording even when its English source is passed to t().
const draftFile = 'src/lib/setup-translations.ts';
const draftTree = ts.createSourceFile(draftFile, fs.readFileSync(draftFile, 'utf8'), ts.ScriptTarget.Latest, true);
function drafts(n) {
  if (ts.isPropertyAssignment(n) && ts.isStringLiteral(n.name) && ts.isStringLiteral(n.initializer)) {
    add(n.name.text, draftFile, draftTree.getLineAndCharacterOfPosition(n.getStart(draftTree)).line + 1, 'Setup: existing translation to review', n.initializer.text);
  }
  ts.forEachChild(n, drafts);
}
drafts(draftTree);
const htmlFile = 'src-tauri/src/services/opac/index.html';
const html = fs.readFileSync(htmlFile, 'utf8');
for (const m of html.matchAll(/>([^<>]+)</g)) {
  if (!/^\s*\{\{/.test(m[1])) add(m[1], htmlFile, html.slice(0, m.index).split('\n').length, 'Public OPAC: screen text');
}
for (const m of html.matchAll(/(?:placeholder|aria-label|alt)="([^"]+)"/g)) add(m[1], htmlFile, html.slice(0, m.index).split('\n').length, 'Public OPAC: accessible label / placeholder');
const jsFile = 'src-tauri/src/services/opac/catalogue.js';
const js = fs.readFileSync(jsFile, 'utf8');
const jsTree = ts.createSourceFile(jsFile, js, ts.ScriptTarget.Latest, true, ts.ScriptKind.JS);
function opac(n) {
  if (ts.isStringLiteral(n) || ts.isNoSubstitutionTemplateLiteral(n) || ts.isTemplateExpression(n)) {
    const text = ts.isTemplateExpression(n) ? n.head.text + n.templateSpans.map(s => `{${s.expression.getText(jsTree)}}${s.literal.text}`).join('') : n.text;
    if (/\s/.test(text) || /^[A-Z][a-z]+$/.test(text)) add(text, jsFile, jsTree.getLineAndCharacterOfPosition(n.getStart(jsTree)).line + 1, 'Public OPAC: dynamic message / option');
  }
  ts.forEachChild(n, opac);
}
opac(jsTree);
const output = [...rows.values()].map(r => ({...r, contexts: [...r.contexts].sort().join('; '), sources: [...r.sources].sort().join('\n')})).sort((a,b) => a.contexts.localeCompare(b.contexts) || a.english.localeCompare(b.english));
fs.mkdirSync('docs/translations', { recursive: true });
fs.writeFileSync('docs/translations/english-review.json', JSON.stringify({ generatedAt: new Date().toISOString(), scope: 'Source-derived UI review inventory; not a guarantee of complete runtime/backend coverage.', rows: output }, null, 2) + '\n');
console.log(`Exported ${output.length} unique candidate phrases (${output.filter(r => r.draft).length} setup drafts).`);
