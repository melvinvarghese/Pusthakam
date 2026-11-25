// Runs the production staff app against a disposable real server and library.
import { spawn } from 'node:child_process';
import { mkdtemp, rm, writeFile, readFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { tmpdir } from 'node:os';
import { join, resolve, relative, isAbsolute } from 'node:path';
import { createServer } from 'node:net';
import { browserReview } from '../design/opac/browser.mjs';
import { staffMobileChecks, publicMobileChecks } from './staff-mobile-checks.mjs';
const data = await mkdtemp(join(tmpdir(), 'pusthakam-staff-test-'));
const within = relative(resolve(tmpdir()), resolve(data));
if (!within || within.startsWith('..') || isAbsolute(within)) throw Error('Test directory outside temporary directory.');
const reservation = createServer();
await new Promise(resolve => reservation.listen(0, '127.0.0.1', resolve));
const port = reservation.address().port;
await new Promise(resolve => reservation.close(resolve));
const binary = process.env.PUSTHAKAM_SERVER_BINARY || 'src-tauri/target/debug/pusthakam-server.exe';
const binarySha256 = createHash('sha256').update(await readFile(resolve(binary))).digest('hex');
const server = spawn(resolve(binary), ['--data-dir', data, '--port', String(port)], { windowsHide: true, stdio: ['pipe', 'pipe', 'pipe'] });
let output = '';
server.stdout.on('data', b => { output += b; });
server.stderr.on('data', b => { output += b; });
const pause = ms => new Promise(resolve => setTimeout(resolve, ms));
const checks = [];
try {
  for (let i = 0; i < 100 && !output.includes('server listening'); i++) { if (server.exitCode !== null) throw Error(output); await pause(100); }
  if (!output.includes('server listening')) throw Error(`Server startup timed out: ${output}`);
  await browserReview(async ({ send, evaluate, waitFor: browserWaitFor, assert, exceptions }) => {
    const waitFor = async expression => {
      try { await browserWaitFor(expression, 30_000); }
      catch (error) { throw Error(`${error.message}\nPage: ${await evaluate('document.body.innerText')}\nErrors: ${JSON.stringify(await evaluate('window.apiErrors'))}`); }
    };
    await send('Emulation.setDeviceMetricsOverride', { width: 1366, height: 950, deviceScaleFactor: 1, mobile: false });
    await send('Page.navigate', { url: `http://127.0.0.1:${port}/staff` });
    await waitFor('!!document.querySelector("meta[name=pusthakam-staff-csrf]")');
    // Finish the app's bootstrap query before the fixture creates its first account.
    await waitFor('document.body.innerText.includes("Set up your library") && !!document.querySelector("input")');
    const setup = { input: { library: { name: 'Browser Test Library', profile: 'school', address: null, contact: null, currency: 'INR', locale: 'en' }, admin: { username: 'owner', displayName: 'Test Owner', password: 'test-password' }, applyProfileDefaults: true, finePerDayMinor: 100 } };
    await evaluate(`window.command=async(command,args={})=>{const r=await fetch('/api/staff/commands',{method:'POST',headers:{'Content-Type':'application/json','X-CSRF-Token':document.querySelector('meta[name=pusthakam-staff-csrf]').content},body:JSON.stringify({command,args})});const data=await r.json();if(!r.ok)throw Error(JSON.stringify(data));return data.result;};true`);
    await evaluate(`command('complete_setup',${JSON.stringify(setup)})`);
    await send('Page.reload');
    await waitFor('!!document.querySelector("nav") && document.body.innerText.includes("Test Owner")');
    checks.push('fresh server setup and authenticated production app');
    await evaluate(`window.apiErrors=[];const originalFetch=window.fetch;window.fetch=async(...args)=>{const response=await originalFetch(...args);if(!response.ok)window.apiErrors.push([String(args[0]),response.status,await response.clone().text()]);return response;};true`);
    if (process.argv.includes('--mobile')) await staffMobileChecks({ send, evaluate, waitFor, assert, pause, checks });
    for (const route of ['catalogue', 'accession', 'members', 'circulation', 'classification', 'inventory', 'acquisitions', 'periodicals', 'reports', 'printing', 'settings', 'backup', 'audit', 'staff']) {
      await evaluate(`location.hash='/${route}'`);
      await pause(400);
      await waitFor('!document.body.innerText.includes("Loading ")');
      await assert('!document.body.innerText.includes("Open Pusthakam as an app")', `desktop-only error at ${route}`);
      const errors = await evaluate('window.apiErrors');
      if (errors.length) throw Error(`${route}: ${JSON.stringify(errors)}`);
      checks.push(`production ${route} route with real API`);
    }
    await send('Emulation.setDeviceMetricsOverride', { width: 390, height: 844, deviceScaleFactor: 1, mobile: true });
    await evaluate("location.hash='/catalogue'"); await pause(200);
    await assert('document.documentElement.scrollWidth<=innerWidth', 'mobile page overflow');
    checks.push('mobile catalogue layout');
    await evaluate("location.hash='/backup'");
    await waitFor('Array.from(document.querySelectorAll("button")).some(b=>b.textContent==="Create backup")');
    await evaluate('Array.from(document.querySelectorAll("button")).find(b=>b.textContent==="Create backup").click()');
    await waitFor('document.body.innerText.includes("Backup saved and checked.")');
    await waitFor('!!document.querySelector("a[download]")');
    await assert(`(async()=>{const response=await fetch(document.querySelector('a[download]').href);const bytes=new Uint8Array(await response.arrayBuffer());return response.ok && new TextDecoder().decode(bytes.slice(0,15))==='SQLite format 3' && response.headers.get('Content-Disposition').includes('attachment');})()`, 'backup download is a real SQLite snapshot');
    await evaluate('Array.from(document.querySelectorAll("button")).find(b=>b.textContent==="Verify").click()');
    await waitFor('document.body.innerText.includes("Integrity check passed.")');
    checks.push('browser create, authenticated download and verify backup');
    await evaluate(`document.querySelector('button[aria-label^="Restore "]').click()`);
    await waitFor('Array.from(document.querySelectorAll("button")).some(b=>b.textContent==="Queue restore")');
    await evaluate('Array.from(document.querySelectorAll("button")).find(b=>b.textContent==="Queue restore").click()');
    await waitFor('document.body.innerText.includes("Restore queued for")');
    await assert('document.body.innerText.includes("Closing this browser does not restart the host.")', 'browser restore explains host restart');
    await evaluate('Array.from(document.querySelectorAll("button")).find(b=>b.textContent==="Cancel queued restore").click()');
    await waitFor('!document.body.innerText.includes("Restore queued for")');
    checks.push('browser restore queue, host restart instructions and cancellation');
    if (process.argv.includes('--mobile')) await publicMobileChecks({ send, evaluate, waitFor, assert, checks });
    if (exceptions.length) throw Error(`Page exceptions: ${exceptions}`);
    console.log(JSON.stringify({ passedChecks: checks.length, exceptions }));
    await writeFile('docs/STAFF_BROWSER_REVIEW.json', JSON.stringify({ at: new Date().toISOString(), binary, binarySha256, source: 'production embedded assets, isolated headless server, temporary SQLite library', mobileReview: process.argv.includes('--mobile'), checks, exceptions, limits: 'Edge touch emulation verifies route navigation/layout, selected dialogs, sign-in, populated catalogue/member records and embedded OPAC. Physical phones, Safari, real soft keyboards, complete workflows, remote HTTPS and printer/scanner acceptance remain separate.' }, null, 2) + '\n');
  });
} finally {
  server.stdin.end('quit\n');
  for (let i = 0; i < 100 && server.exitCode === null; i++) await pause(100);
  if (server.exitCode === null) { server.kill(); await new Promise(resolve => server.once('exit', resolve)); }
  await rm(data, { recursive: true, force: true, maxRetries: 5, retryDelay: 200 });
}
