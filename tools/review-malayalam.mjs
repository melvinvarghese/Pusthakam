// Current React source with synthetic development data; never owner library data.
import { createServer } from 'vite';
import { mkdir, writeFile } from 'node:fs/promises';
import { browserReview } from '../design/opac/browser.mjs';

const server = await createServer({ server: { host: '127.0.0.1', port: 0, open: false } });
await server.listen();
const address = server.httpServer.address();
const origin = `http://127.0.0.1:${address.port}`;
const output = new URL('../design/malayalam/previews/', import.meta.url);
await mkdir(output, { recursive: true });
const checks = [];
try {
  await browserReview(async ({ send, evaluate, waitFor, assert, exceptions, pause }) => {
    const text = english => JSON.stringify(english);
    const choose = async language => {
      await evaluate(`(()=>{const s=document.querySelector('select');s.value=${text(language)};s.dispatchEvent(new Event('change',{bubbles:true}));})()`);
      await pause(100);
    };
    await send('Page.navigate', { url: `${origin}/?preview=login` });
    await waitFor(`!!document.querySelector('input[autocomplete="username"]')`, 15000);
    await assert('document.documentElement.lang==="en"', 'Fresh profile must default to English');
    await evaluate(`(()=>{for(const [name,value] of [['username','review-user'],['current-password','review-password']]){const input=document.querySelector('[autocomplete="'+name+'"]');Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(input,value);input.dispatchEvent(new Event('input',{bubbles:true}));}})()`);
    await choose('ml');
    await assert(`document.querySelector('h1').textContent===${text('ലോഗിൻ ചെയ്യുക')}`, 'Malayalam login heading');
    await assert(`document.querySelector('input[autocomplete="username"]').value==='review-user'`, 'Username retained');
    checks.push('English default; Malayalam switch; credentials retained');
    for (const width of [320, 390, 768, 1366]) {
      await send('Emulation.setDeviceMetricsOverride', { width, height: 900, deviceScaleFactor: 1, mobile: width < 800 });
      await pause(100);
      await assert('document.documentElement.scrollWidth<=innerWidth', `Login overflows at ${width}`);
      const screenshot = await send('Page.captureScreenshot', { format: 'png' });
      await writeFile(new URL(`login-${width}.png`, output), Buffer.from(screenshot.data, 'base64'));
      checks.push(`Malayalam login layout at ${width}px`);
    }
    await send('Page.navigate', { url: `${origin}/?preview` });
    await waitFor(`!!document.querySelector('.staff-shell')`, 15000);
    await assert(`document.documentElement.lang==='ml'`, 'Language retained across reload');
    for (const width of [320, 390, 768, 1366]) {
      await send('Emulation.setDeviceMetricsOverride', { width, height: 900, deviceScaleFactor: 1, mobile: width < 800 });
      await pause(100);
      await assert('document.documentElement.scrollWidth<=innerWidth', `Staff page overflows at ${width}`);
      await assert(`(()=>{const h=document.querySelector('.staff-topbar');return h.scrollWidth<=h.clientWidth;})()`, `Staff header overflows at ${width}`);
      const screenshot = await send('Page.captureScreenshot', { format: 'png' });
      await writeFile(new URL(`staff-${width}.png`, output), Buffer.from(screenshot.data, 'base64'));
      checks.push(`Malayalam staff header/page layout at ${width}px`);
    }
    await assert(`Array.from(document.querySelectorAll('nav a')).some(a=>a.textContent===${text('പുസ്തകപ്പട്ടിക')})`, 'Translated catalogue nav');
    await evaluate(`location.hash='/settings'`);
    await waitFor(`!!document.querySelector('select option[value="ml"]')`);
    await choose('en');
    await assert(`document.documentElement.lang==='en'`, 'Settings switches back to English');
    checks.push('Persisted preference; translated navigation; settings switch back to English');
    if (exceptions.length) throw Error(JSON.stringify(exceptions));
    checks.push('No browser runtime exceptions');
  });
  const report = { at: new Date().toISOString(), scope: 'Current frontend source, synthetic dev preview; no native backend/installer acceptance', checks };
  await writeFile(new URL('../docs/translations/SHARED_UI_REVIEW.json', import.meta.url), JSON.stringify(report, null, 2) + '\n');
  console.log(JSON.stringify(report));
} finally {
  await server.close();
}
