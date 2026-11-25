import { writeFile, mkdir } from 'node:fs/promises';
import { server } from './server.mjs';
import { browserReview } from './browser.mjs';
const output = new URL('./previews/', import.meta.url);
await mkdir(output, { recursive: true });
try {
  await browserReview(async ({ send, evaluate, waitFor, assert, pause, exceptions, requests }) => {
    const results = [];
    for (const view of ['reading-room', 'swiss-stacks', 'sovereign-folio']) {
      await send('Emulation.setDeviceMetricsOverride', { width: 1366, height: 950, deviceScaleFactor: 1, mobile: false });
      await send('Page.navigate', { url: `http://127.0.0.1:5193/design/opac/?view=${view}` });
      await waitFor('document.querySelectorAll(".cloth-book-card, .swiss-row, .folio-card").length === 10');
      await evaluate('document.fonts.ready.then(()=>true)');
      for (const [width, height] of [[1366,950],[1024,768],[768,1024],[390,844],[320,800]]) {
        await send('Emulation.setDeviceMetricsOverride', { width, height, deviceScaleFactor: 1, mobile: width < 641 }); await pause(80);
        const dimensions = await evaluate('({viewport:innerWidth,scroll:document.documentElement.scrollWidth})');
        await assert('document.documentElement.scrollWidth <= innerWidth', `${view}: horizontal overflow at ${width}: ${JSON.stringify(dimensions)}`);
        await assert('document.querySelector("#search-input").getBoundingClientRect().width > 100', `${view}: search at ${width}`);
        results.push({ view, width, height, horizontalOverflow: false });
        if (width === 1366 || width === 390) {
          const shot = await send('Page.captureScreenshot', { format: 'png' });
          await writeFile(new URL(`opac-${view}-${width === 1366 ? 'desktop' : 'mobile'}.png`, output), Buffer.from(shot.data, 'base64'));
        }
      }
      await evaluate('(()=>{const i=document.querySelector("#search-input");i.value="Basheer";i.dispatchEvent(new Event("input",{bubbles:true}));})()');
      await assert('document.querySelectorAll(".cloth-book-card, .swiss-row, .folio-card").length===1 && document.querySelector(".cloth-book-card, .swiss-row, .folio-card").textContent.includes("Balyakalasakhi")', `${view}: author search`);
      await evaluate('document.querySelector("#search-clear").click(); document.querySelector("#avail-check").click()');
      await assert('document.querySelectorAll(".cloth-book-card, .swiss-row, .folio-card").length===8', `${view}: availability`);
      await evaluate('document.querySelector("#avail-check").click(); document.querySelector("[data-subject=History]").click()');
      await assert('document.querySelectorAll(".cloth-book-card, .swiss-row, .folio-card").length===1', `${view}: subject`);
      await evaluate('document.querySelector("[data-subject=All]").click();document.querySelector(".cloth-book-card, .swiss-row, .folio-card").focus();');
      await send('Input.dispatchKeyEvent', { type: 'keyDown', key: 'Enter', code: 'Enter', windowsVirtualKeyCode: 13 });
      await send('Input.dispatchKeyEvent', { type: 'keyUp', key: 'Enter', code: 'Enter', windowsVirtualKeyCode: 13 });
      await assert('document.querySelector("#details-dialog").open', `${view}: keyboard details`);
      await send('Input.dispatchKeyEvent', { type: 'keyDown', key: 'Escape', code: 'Escape', windowsVirtualKeyCode: 27 });
      await send('Input.dispatchKeyEvent', { type: 'keyUp', key: 'Escape', code: 'Escape', windowsVirtualKeyCode: 27 });
      await assert('!document.querySelector("#details-dialog").open', `${view}: Escape`);
      await evaluate('(()=>{const i=document.querySelector("#search-input");i.value="not-a-book-anywhere";i.dispatchEvent(new Event("input",{bubbles:true}));})()');
      await assert('document.querySelectorAll(".cloth-book-card, .swiss-row, .folio-card").length===0 && !!document.querySelector("#reset-filters")', `${view}: no matches`);
      await evaluate('document.querySelector("#reset-filters").click()');
      await assert('document.querySelectorAll(".cloth-book-card, .swiss-row, .folio-card").length===10', `${view}: reset`);
      results.push({ view, interactions: ['author search','availability','subject','keyboard details','Escape','empty state','reset'] });
    }
    const external = requests.filter(url => /^https?:/.test(url) && new URL(url).hostname !== '127.0.0.1');
    if (exceptions.length || external.length) throw Error(JSON.stringify({ exceptions, external }));
    await writeFile(new URL('review-results.json', output), JSON.stringify({ at: new Date().toISOString(), results, pageExceptions: exceptions, externalPageRequests: external }, null, 2)+'\n');
    console.log(JSON.stringify({ layouts: 15, interactions: 21, exceptions: exceptions.length, externalRequests: external.length }));
  });
} finally { server.closeAllConnections(); server.close(); }
