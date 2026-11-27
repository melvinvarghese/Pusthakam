// Usage: run `npx vite --port 1420` in the repo root, then `node tools/website-screenshots/run.mjs`.
// Writes PNGs to website/assets/screens/ (convert to WebP before committing). All data shown is synthetic.
// Needs Playwright with Chromium, which is not a project dependency: `npm install --global playwright`.
import { createRequire } from "node:module";
import { mkdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { page } from "./page.mjs";
import opacShot from "./opac.mjs";

const { chromium, devices } = createRequire(import.meta.url)("playwright");

const out = fileURLToPath(new URL("../../website/assets/screens", import.meta.url));
mkdirSync(out, { recursive: true });
const browser = await chromium.launch();
for (const theme of ["light", "dark"]) {
  const p = await page(browser, { viewport: { width: 1120, height: 700 }, deviceScaleFactor: 2, colorScheme: theme });
  await p.goto("http://127.0.0.1:1420/?preview#/", { waitUntil: "networkidle" });
  await p.waitForTimeout(4000);
  const go = async (route) => { await p.evaluate((r) => { globalThis.location.hash = "#/" + r; }, route); await p.waitForTimeout(1500); };
  await go(""); await p.screenshot({ path: `${out}/dashboard-${theme}.png` });
  if (theme === "dark") { await p.close(); continue; }
  await go("catalogue"); await p.screenshot({ path: `${out}/catalogue.png` });
  await go("circulation");
  await p.getByLabel("Member number").fill("DEMO-001");
  await p.getByLabel("Book barcode or accession number").fill("004217");
  await p.getByRole("button", { name: "Review loan", exact: true }).click();
  await p.waitForTimeout(800);
  await p.screenshot({ path: `${out}/circulation.png` });
  await go("reports"); await p.screenshot({ path: `${out}/reports.png` });
  await p.close();
}
await opacShot(browser, devices, `${out}/opac-phone.png`);
await browser.close();
