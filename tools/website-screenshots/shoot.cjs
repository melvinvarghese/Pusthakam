// Takes website screenshots from the app's dev preview with synthetic data. Run with `npm run dev` going.
const { chromium } = require('playwright');
const fs = require('fs'); const path = require('path');
const mock = fs.readFileSync(path.join(__dirname, 'mock.js'), 'utf8');
async function page(b, opts = {}) {
  const p = await b.newPage({ viewport: { width: 1440, height: 900 }, ...opts });
  await p.route('**/src/features/dev/preview.ts*', async (route) => {
    const resp = await route.fetch(); let body = await resp.text();
    body = body.replace('export async function previewCall(', 'async function basePreviewCall(') + '\n' + mock;
    route.fulfill({ response: resp, body });
  });
  p.on('console', m => { if (process.env.LOG && m.text().startsWith('CMD')) console.log(m.text()); });
  p.on('pageerror', e => console.log('ERR', e.message));
  return p;
}
module.exports = { page };
if (require.main === module) (async () => {
  const b = await chromium.launch();
  const p = await page(b);
  await p.goto(`http://127.0.0.1:1420/?preview#/`, { waitUntil: 'networkidle' });
  await p.waitForTimeout(4000);
  for (const r of process.argv.slice(2)) { console.log('== ' + r); await p.evaluate(r => location.hash = '#/' + r, r); await p.waitForTimeout(1500); await p.screenshot({ path: `/tmp/shots/${r.replace(/\W/g,'_')||'dashboard'}.png` }); }
  await b.close();
})();
