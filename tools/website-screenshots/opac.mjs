// Renders the real public catalogue page (src-tauri/src/services/opac) on a phone with synthetic search results.
import fs from 'node:fs'; import path from 'node:path'; import { fileURLToPath } from 'node:url';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..'); const dir = path.join(root, 'src-tauri/src/services/opac');
const b64 = (f) => fs.readFileSync(f).toString('base64');
function html(name) {
  const nm = path.join(root, 'node_modules');
  const css = fs.readFileSync(path.join(dir, 'styles.css'), 'utf8')
    .replace('{{SANS_FONT}}', b64(path.join(nm, '@fontsource-variable/hanken-grotesk/files/hanken-grotesk-latin-wght-normal.woff2')))
    .replace('{{MONO_FONT}}', b64(path.join(nm, '@fontsource/courier-prime/files/courier-prime-latin-400-normal.woff2')))
    .replace('{{MALAYALAM_FONT}}', b64(path.join(nm, '@fontsource-variable/noto-sans-malayalam/files/noto-sans-malayalam-malayalam-wght-normal.woff2')));
  return fs.readFileSync(path.join(dir, 'index.html'), 'utf8').replace('{{STYLES}}', css)
    .replace('{{SCRIPT}}', fs.readFileSync(path.join(dir, 'catalogue.js'), 'utf8'))
    .replaceAll('{{LOGO}}', b64(path.join(root, 'src/assets/pusthakam-logo.png'))).replaceAll('{{LIBRARY_NAME}}', name);
}
const items = [
  { id: 1, title: 'Randamoozham', subtitle: null, authors: ['M. T. Vasudevan Nair'], publisher: 'Current Books', year: 1984, isbn: null, classification: '894.8123 VAS', subjects: ['Malayalam fiction'], locations: ['Main Library, shelf A1'], totalCopies: 2, availableCopies: 1 },
  { id: 2, title: 'Balyakalasakhi', subtitle: null, authors: ['Vaikom Muhammad Basheer'], publisher: 'DC Books', year: 1944, isbn: null, classification: '894.8123 BAS', subjects: ['Malayalam fiction'], locations: ['Main Library, shelf A1'], totalCopies: 3, availableCopies: 0 },
];
export default async function opacShot(browser, devices, out) {
  const ctx = await browser.newContext({ ...devices['iPhone 13'] });
  const p = await ctx.newPage();
  await p.route('http://library.local/api/opac/search*', r => r.fulfill({ json: { items, total: 2, page: 1, pageSize: 20 } }));
  await p.route('http://library.local/api/opac/cover/*', r => r.fulfill({ status: 404 }));
  await p.route('http://library.local/opac', r => r.fulfill({ contentType: 'text/html', body: html('Demo School Library') }));
  await p.goto('http://library.local/opac');
  await p.fill('#q', 'malayalam fiction'); await p.keyboard.press('Enter'); await p.waitForTimeout(800); await p.locator('#q').blur();
  await p.screenshot({ path: out }); await ctx.close();
}
