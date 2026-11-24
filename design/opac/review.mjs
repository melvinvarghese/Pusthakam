import { writeFile, mkdir } from 'node:fs/promises';
import { server } from './server.mjs';

const output = new URL('./previews/', import.meta.url);
await mkdir(output, { recursive: true });
let targets;
for (let attempt=0; attempt<80; attempt++) {
  try { targets = await fetch('http://127.0.0.1:9361/json/list', {signal:AbortSignal.timeout(1000)}).then(r=>r.json()); break; }
  catch { await new Promise(resolve=>setTimeout(resolve,200)); }
}
if (!targets) { server.close(); throw Error('Preview browser did not start'); }
const target = targets.find(t => t.type === 'page' && t.url === 'about:blank');
if (!target) throw Error('No isolated preview tab found');
const ws = new WebSocket(target.webSocketDebuggerUrl);
await new Promise((resolve, reject) => { ws.onopen = resolve; ws.onerror = reject; });
let next = 0;
const pending = new Map();
const failures = [];
const requests = [];
ws.onmessage = event => {
  const message = JSON.parse(event.data);
  if (message.method === 'Runtime.exceptionThrown') failures.push(message.params.exceptionDetails.text);
  if (message.method === 'Network.requestWillBeSent') requests.push(message.params.request.url);
  if (!message.id) return;
  const item = pending.get(message.id);
  if (!item) return;
  clearTimeout(item.timer); pending.delete(message.id);
  if (message.error) item.reject(Error(JSON.stringify(message.error))); else item.resolve(message.result);
};
function send(method, params = {}) {
  return new Promise((resolve, reject) => {
    const id = ++next;
    const timer = setTimeout(() => { pending.delete(id); reject(Error(`${method} timeout`)); }, 20000);
    pending.set(id, { resolve, reject, timer });
    ws.send(JSON.stringify({ id, method, params }));
  });
}
const pause = ms => new Promise(resolve => setTimeout(resolve, ms));
async function evaluate(expression) {
  const r = await send('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true });
  if (r.exceptionDetails) throw Error(JSON.stringify(r.exceptionDetails));
  return r.result.value;
}
async function ready() {
  for (let i = 0; i < 80; i++) {
    if (await evaluate('document.querySelectorAll(".book").length === 8')) {
      await evaluate('document.fonts.ready.then(()=>true)'); return;
    }
    await pause(100);
  }
  throw Error('Prototype failed to load');
}
async function assert(expression, label) {
  if (!await evaluate(expression)) throw Error(label);
}
const results = [];
try {
  await send('Page.enable'); await send('Runtime.enable'); await send('Network.enable');
  for (const view of ['shelf', 'index', 'table']) {
    await send('Emulation.setDeviceMetricsOverride', { width: 1366, height: 1000, deviceScaleFactor: 1, mobile: false });
    await send('Page.navigate', { url: `http://127.0.0.1:5193/design/opac/?view=${view}` }); await ready();
    for (const [width, height] of [[1366,1000],[768,1024],[390,960],[320,800]]) {
      await send('Emulation.setDeviceMetricsOverride', { width, height, deviceScaleFactor: 1, mobile: width < 641 }); await pause(100);
      await assert('document.documentElement.scrollWidth <= innerWidth', `${view}: overflow at ${width}`);
      await assert('document.querySelector("#search").getBoundingClientRect().width > 100', `${view}: unusable search at ${width}`);
      results.push({ view, width, height, horizontalOverflow: false });
      if (width === 1366 || width === 390) {
        const screenshot = await send('Page.captureScreenshot', { format: 'png' });
        await writeFile(new URL(`${view}-${width === 1366 ? 'desktop' : 'mobile'}.png`, output), Buffer.from(screenshot.data, 'base64'));
      }
    }
    await evaluate('(()=>{const i=document.querySelector("#search");i.value="Basheer";i.dispatchEvent(new Event("input",{bubbles:true}));})()');
    await assert('document.querySelectorAll(".book").length===1 && document.querySelector(".book h3").textContent==="Balyakalasakhi"', `${view}: author search`);
    await evaluate('document.querySelector("#clear").click(); document.querySelector("#available").click()');
    await assert('document.querySelectorAll(".book").length === 6', `${view}: availability filter`);
    await evaluate('document.querySelector("#available").click(); document.querySelector("[data-subject=History]").click()');
    await assert('document.querySelectorAll(".book").length===1', `${view}: subject filter`);
    await evaluate('(()=>{document.querySelectorAll("[data-subject]")[0].click(); const s=document.querySelector("#sort");s.value="title";s.dispatchEvent(new Event("change"));})()');
    await assert('document.querySelector(".book h3").textContent === "A Brief History of Time"', `${view}: title sorting`);
    await evaluate('document.querySelector(".details-link").focus(); document.querySelector(".details-link").click()');
    await assert('document.querySelector("#details").open && document.querySelector("#detail-title").textContent === "A Brief History of Time"', `${view}: book details`);
    await send('Input.dispatchKeyEvent', { type: 'keyDown', key: 'Escape', code: 'Escape', windowsVirtualKeyCode: 27 });
    await send('Input.dispatchKeyEvent', { type: 'keyUp', key: 'Escape', code: 'Escape', windowsVirtualKeyCode: 27 });
    await assert('!document.querySelector("#details").open', `${view}: Escape dismissal`);
    await evaluate('(()=>{const i=document.querySelector("#search");i.value="not-a-book-anywhere";i.dispatchEvent(new Event("input",{bubbles:true}));})()');
    await assert('document.querySelectorAll(".book").length===0 && !document.querySelector("#empty").hidden', `${view}: empty state`);
    await evaluate('document.querySelector("#reset").click()');
    await assert('document.querySelectorAll(".book").length===8', `${view}: reset`);
    results.push({ view, interactions: ['author search','availability','subject','sorting','details','Escape','empty state','reset'] });
  }
  const external = requests.filter(url => /^https?:/.test(url) && new URL(url).hostname !== '127.0.0.1');
  if (failures.length || external.length) throw Error(JSON.stringify({ failures, external }));
  await writeFile(new URL('review-results.json', output), JSON.stringify({ results, pageExceptions: failures, externalPageRequests: external }, null, 2));
  console.log(JSON.stringify({ layouts: 12, interactionChecks: 24, pageExceptions: failures.length, externalPageRequests: external.length, screenshots: 6 }));
} finally {
  try { await send('Browser.close'); } catch { /* Browser closes its debugger connection. */ }
  ws.close();
  server.closeAllConnections();
  server.close();
}
