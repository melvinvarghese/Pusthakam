// Usage: start `npm run dev` in the repo root, then `node tools/website-screenshots/run.cjs`.
// Writes PNGs to website/assets/screens/. All data shown is synthetic (see mock.js and opac.cjs).
const { chromium, devices } = require('playwright');
const path = require('path'); const { page } = require('./shoot.cjs'); const opacShot = require('./opac.cjs');
const out = path.resolve(__dirname, '../../website/assets/screens'); require('fs').mkdirSync(out, { recursive: true });
(async () => {
  const b = await chromium.launch();
  for (const theme of ['light', 'dark']) {
    const p = await page(b, { viewport: { width: 1120, height: 700 }, deviceScaleFactor: 2, colorScheme: theme });
    await p.goto('http://127.0.0.1:1420/?preview#/', { waitUntil: 'networkidle' }); await p.waitForTimeout(4000);
    const go = async (r) => { await p.evaluate(r => location.hash = '#/' + r, r); await p.waitForTimeout(1500); };
    await go(''); await p.screenshot({ path: `${out}/dashboard-${theme}.png` });
    if (theme === 'dark') continue;
    await go('catalogue'); await p.screenshot({ path: `${out}/catalogue.png` });
    await go('circulation');
    await p.getByLabel('Member number').fill('DEMO-001'); await p.getByLabel('Book barcode or accession number').fill('004217');
    await p.getByRole('button', { name: 'Review loan', exact: true }).click(); await p.waitForTimeout(800);
    await p.screenshot({ path: `${out}/circulation.png` });
    await go('reports'); await p.screenshot({ path: `${out}/reports.png` });
    await p.close();
  }
  await opacShot(b, devices, `${out}/opac-phone.png`);
  await b.close();
})();
