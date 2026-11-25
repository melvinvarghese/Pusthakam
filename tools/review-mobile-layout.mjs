// Geometry review only: source shell classes/mobile CSS plus the last compiled base CSS.
// React interaction and fresh-bundle checks are separate, never inferred from this fixture.
import { createServer } from 'node:http';
import { readFile, readdir, mkdir, writeFile } from 'node:fs/promises';
import { browserReview } from '../design/opac/browser.mjs';
const root = new URL('../', import.meta.url);
const source = name => readFile(new URL(name, root), 'utf8');
const shell = await source('src/app/AppShell.tsx');
const modal = await source('src/components/ui/Modal.tsx');
const mobile = await source('src/styles/mobile.css');
const index = await source('dist/index.html');
const cssPath = index.match(/href="([^"]+\.css)"/)[1];
const base = await source(`dist${cssPath}`);
const classes = (text, token) => {
  const found = text.match(new RegExp(`className="([^"]*${token}[^"]*)"`));
  if (!found) throw Error(`Source class missing: ${token}`);
  return found[1];
};
const assets = new Map();
for (const name of await readdir(new URL('dist/assets/', root))) {
  if (/\.(woff2?|png)$/.test(name)) assets.set(`/assets/${name}`, await readFile(new URL(`dist/assets/${name}`, root)));
}
const nav = '<nav aria-label="Primary">' + ['Dashboard', 'Catalogue', 'Circulation', 'Members', 'Inventory', 'Library Settings'].map(name => `<a href="#main" class="block h-8 px-3 leading-8">${name}</a>`).join('') + '</nav>';
const rail = `<aside class="staff-sidebar row-span-2 flex min-h-0 flex-col bg-ink text-rail-text"><div class="h-14 px-4">Pusthakam</div>${nav}</aside>`;
const form = '<form class="grid grid-cols-2 gap-3"><label>Book title<input class="w-full h-9 border px-3" value="A library book"></label><label>ISBN<input class="w-full h-9 border px-3" value="9780140449136"></label></form>';
const memberFixture = '<section class="staff-panel border border-line"><table role="table" class="staff-card-table staff-member-table w-full"><thead><tr><th>Member number</th><th>Name</th><th>Category</th><th>Expiry</th><th>Actions</th></tr></thead><tbody><tr><td data-mobile-label="Member number">MEM-001</td><td><div>Library Member</div><small>member@example.org</small></td><td data-mobile-label="Category">Adult reader</td><td data-mobile-label="Expiry date">2027-11-25</td><td><button class="px-3">Loans</button><button class="px-3">Fines</button><button class="px-3">Card</button><button class="px-3">Edit</button><button class="px-3">Archive</button></td></tr></tbody></table></section>';
const html = `<!doctype html><meta name="viewport" content="width=device-width,initial-scale=1"><style>${base}\n${mobile}</style><div id="root">
<div class="${classes(shell, 'staff-shell')}">${rail}
<header class="${classes(shell, 'staff-topbar')}"><button class="staff-menu-toggle" aria-label="Open navigation">☰</button><button class="${classes(shell, 'staff-search')}"><span>Search or go to a section</span><kbd>Ctrl K</kbd>⌕<span class=staff-search-short>Search</span></button><div class="staff-account ml-auto flex items-center gap-4 text-rail-text"><div class="inline-flex"><button class="h-8 px-2">☀</button><button class="h-8 px-2">▣</button><button class="h-8 px-2">☾</button></div><div class="staff-account-name">Librarian</div><button class="h-8 px-3">Sign out</button></div></header>
<main id="main" class="${classes(shell, 'staff-workspace')}"><div class="mx-auto max-w-[1100px] space-y-5 p-6"><header class="staff-page-heading flex items-end justify-between gap-3"><h1 class='text-xl font-bold leading-7'>Catalogue</h1><div class="flex flex-wrap gap-2"><button class="border px-3">Add book</button><button class="border px-3">Import records</button></div></header>
<section class="staff-panel min-w-0 rounded-lg border border-line bg-surface"><div class="flex flex-wrap gap-3 p-3"><input aria-label="Search" class="w-full min-w-0 flex-1 h-9 border px-3" placeholder="Title, author or ISBN"><button class="border px-3">Search</button></div><table role="table" class="staff-card-table w-full table-fixed"><thead><tr><th>Select</th><th>Title</th><th>Publisher</th><th>Class</th><th>Copies</th></tr></thead><tbody><tr><td><input type="checkbox" aria-label="Select book"></td><td><div class="flex items-start gap-3"><div class="shrink-0 w-10 h-14 border" aria-hidden="true"></div><div class="min-w-0"><a href="#main" class="font-semibold">A long catalogue title for a real-sized phone display</a><p>Library Author — 9780140449136</p></div></div></td><td data-mobile-label="Publisher">Library Press, 2026</td><td data-mobile-label="Class">823.9</td><td data-mobile-label="Copies">1 of 2 available</td></tr></tbody></table></section>
<section class="staff-panel border border-line"><table id="wide-table" class="w-full"><caption>Detailed report: swipe within this table</caption><thead><tr><th>Member</th><th>Title</th><th>Issued</th><th>Due</th><th>Actions</th></tr></thead><tbody><tr><td>Member number</td><td>A long title</td><td>2026-11-25</td><td>2026-12-09</td><td><button class="px-3">Return</button></td></tr></tbody></table></section>${memberFixture}${form}</div></main></div></div>
<div id="modal" hidden class="${classes(modal, 'staff-modal-backdrop')}"><div role="dialog" aria-modal="true" class="staff-dialog w-[min(30rem,100%)] bg-surface border border-line"><h2 class="border-b px-5 py-3">Edit book</h2><div class="p-5">${form}</div><div class="flex justify-end gap-2 border-t px-5 py-3"><button class="px-3">Cancel</button><button class="px-3">Save book</button></div></div></div>`;
const server = createServer((req, res) => {
  if (req.url === '/') { res.setHeader('Content-Type', 'text/html; charset=utf-8'); res.end(html); }
  else if (assets.has(req.url)) { res.end(assets.get(req.url)); }
  else { res.writeHead(404); res.end(); }
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const output = new URL('design/mobile/previews/', root);
await mkdir(output, { recursive: true });
const checks = [];
try {
  await browserReview(async ({ send, evaluate, waitFor, assert, pause, exceptions }) => {
    await send('Network.setBlockedURLs', { urls: ['https://*', 'http://*.microsoft.com/*'] });
    await send('Page.navigate', { url: `http://127.0.0.1:${server.address().port}/` });
    await waitFor('!!document.querySelector(".staff-workspace")', 30000);
    for (const [width, height] of [[320, 740], [390, 844], [768, 1024], [1024, 768], [1366, 950]]) {
      await send('Emulation.setDeviceMetricsOverride', { width, height, deviceScaleFactor: 1, mobile: width < 1024 });
      await pause(100);
      await evaluate('document.querySelector(".staff-workspace").scrollTop=0');
      await assert('document.documentElement.scrollWidth<=innerWidth', `page overflow ${width}`);
      if (width < 1024) {
        await assert('Math.abs(document.querySelector(".staff-workspace").getBoundingClientRect().width-innerWidth)<2', `full workspace ${width}`);
        await assert('getComputedStyle(document.querySelector(".staff-sidebar")).display==="none"', `rail hidden ${width}`);
        await assert('document.querySelector(".staff-menu-toggle").getBoundingClientRect().height>=44', `touch menu ${width}`);
        await assert('document.querySelector(".staff-card-table td:nth-child(2)").getBoundingClientRect().width>=150', `readable record ${width}`);
        await assert('document.querySelector("#wide-table").parentElement.scrollWidth>document.querySelector("#wide-table").parentElement.clientWidth || innerWidth>=700', `table scroll ${width}`);
        await assert('parseFloat(getComputedStyle(document.querySelector("input[aria-label=Search]")).fontSize)>=16', `input size ${width}`);
        await assert('getComputedStyle(document.querySelector(".staff-workspace form")).gridTemplateColumns.split(" ").length===1', `workspace form columns ${width}`);
        await assert('Array.from(document.querySelectorAll(".staff-member-table button")).every(b=>b.getBoundingClientRect().right<=innerWidth && b.getBoundingClientRect().height>=44)', `member actions ${width}`);
        await evaluate('document.querySelector("#modal").hidden=false');
        await assert('document.querySelector("[role=dialog]").getBoundingClientRect().right<=innerWidth && document.querySelector("[role=dialog]").getBoundingClientRect().height<=innerHeight', `dialog bounds ${width}`);
        await assert('getComputedStyle(document.querySelector("[role=dialog] form")).gridTemplateColumns.split(" ").length===1', `single-column form ${width}`);
        await evaluate('document.querySelector("#modal").hidden=true');
      } else {
        await assert('getComputedStyle(document.querySelector(".staff-sidebar")).display!=="none"', `desktop rail ${width}`);
      }
      checks.push(`source layout ${width}x${height}`);
      if ([390, 1366].includes(width)) {
        const shot = await send('Page.captureScreenshot', { format: 'png' });
        await writeFile(new URL(`staff-${width}.png`, output), Buffer.from(shot.data, 'base64'));
        if(width===390){
          await evaluate('document.querySelector(".staff-member-table").scrollIntoView()');
          const members=await send('Page.captureScreenshot',{format:'png'});
          await writeFile(new URL('staff-members-390.png',output),Buffer.from(members.data,'base64'));
        }
      }
    }
    if (exceptions.length) throw Error(JSON.stringify(exceptions));
    const result = { at: new Date().toISOString(), source: 'Source AppShell class names and mobile.css; last compiled base CSS; explicit fixture holdings, not a newly built React application', checks, exceptions, limits: 'Geometry only. React menu/route behavior tests and fresh native/browser bundle verification remain blocked by the recorded build/tool telemetry issue; actual phone and keyboard acceptance pending.' };
    await writeFile(new URL('review.json', output), JSON.stringify(result, null, 2) + '\n');
    console.log(JSON.stringify(result));
  });
} finally { server.closeAllConnections(); server.close(); }
