// Real process restart/restore proof using an isolated library.
import { spawn } from 'node:child_process';
import { mkdtemp, rm, writeFile, readFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { tmpdir } from 'node:os';
import { join, relative, resolve, isAbsolute } from 'node:path';
import { createServer, connect } from 'node:net';
const pause = ms => new Promise(resolve => setTimeout(resolve, ms));
const data = await mkdtemp(join(tmpdir(), 'pusthakam-restore-test-'));
const within = relative(resolve(tmpdir()), resolve(data));
if (!within || within.startsWith('..') || isAbsolute(within)) throw Error('Unsafe test directory.');
const reservation = createServer();
await new Promise(resolve => reservation.listen(0, '127.0.0.1', resolve));
const port = reservation.address().port;
await new Promise(resolve => reservation.close(resolve));
const origin = `http://127.0.0.1:${port}`;
const binary = process.env.PUSTHAKAM_SERVER_BINARY || 'src-tauri/target/debug/pusthakam-server.exe';
const binarySha256 = createHash('sha256').update(await readFile(resolve(binary))).digest('hex');
let child;
let output = '';
let cookie, csrf;
let stage = 'startup';
async function start() {
  stage = 'starting server';
  output = '';
  child = spawn(resolve(binary), ['--data-dir', data, '--port', String(port)], { windowsHide: true, stdio: ['pipe', 'pipe', 'pipe'] });
  child.stdout.on('data', bytes => { output += bytes; });
  child.stderr.on('data', bytes => { output += bytes; });
  child.on('error', error => { output += error.message; });
  for (let i = 0; i < 300 && !output.includes('server listening'); i++) {
    if (child.exitCode !== null) throw Error(output);
    await pause(100);
  }
  if (!output.includes('server listening')) throw Error(`Startup timeout: ${output}`);
  await delayedRequest(true);
  stage = 'loading staff page';
  const page = await fetch(`${origin}/staff`);
  cookie = page.headers.get('set-cookie').split(';')[0];
  csrf = (await page.text()).match(/name="pusthakam-staff-csrf" content="([a-f0-9]+)"/)[1];
}
async function stop() {
  if (!child || child.exitCode !== null) return;
  child.stdin.end('quit\n');
  for (let i = 0; i < 300 && child.exitCode === null; i++) await pause(100);
  if (child.exitCode !== 0) throw Error(`Graceful shutdown failed: ${output}`);
}
async function command(command, args = {}) {
  stage = command;
  const response = await fetch(`${origin}/api/staff/commands`, { method: 'POST', headers: { Cookie: cookie, Origin: origin, 'Content-Type': 'application/json', 'X-CSRF-Token': csrf }, body: JSON.stringify({ command, args }) });
  if (response.headers.get('set-cookie')) cookie = response.headers.get('set-cookie').split(';')[0];
  const body = await response.json();
  if (!response.ok) throw Error(`${command}: ${JSON.stringify(body)}`);
  return body.result;
}
async function delayedRequest(page = false) {
  stage = 'delayed request headers/body';
  const body = JSON.stringify({ command: 'current_session', args: {} });
  const response = await new Promise((resolve, reject) => {
    const socket = connect(port, '127.0.0.1');
    let result = '';
    socket.setTimeout(5000, () => socket.destroy(new Error('Delayed request timed out.')));
    socket.on('error', reject);
    socket.on('data', data => { result += data; });
    socket.on('end', () => resolve(result));
    socket.on('connect', () => setTimeout(() => {
      if (page) { socket.write(`GET /staff HTTP/1.1\r\nHost: 127.0.0.1:${port}\r\nConnection: close\r\n\r\n`); return; }
      socket.write(`POST /api/staff/commands HTTP/1.1\r\nHost: 127.0.0.1:${port}\r\nCookie: ${cookie}\r\nX-CSRF-Token: ${csrf}\r\nContent-Type: application/json\r\nContent-Length: ${Buffer.byteLength(body)}\r\nConnection: close\r\n\r\n`);
      setTimeout(() => socket.write(body), 100);
    }, 100));
  });
  if (!response.startsWith('HTTP/1.1 200')) throw Error(`Split request rejected: ${response}`);
}
try {
  await start();
  await delayedRequest();
  await command('complete_setup', { input: { library: { name: 'Restore Test Library', profile: 'school', address: null, contact: null, currency: 'INR', locale: 'en' }, admin: { username: 'owner', displayName: 'Test Owner', password: 'test-password' }, applyProfileDefaults: true, finePerDayMinor: 100 } });
  const backup = await command('backup_create');
  await command('set_setting', { key: 'restore.test', value: 'after-backup' });
  await command('backup_request_restore', { name: backup.name });
  await stop();
  await start();
  await command('login', { username: 'owner', password: 'test-password' });
  if (await command('get_setting', { key: 'restore.test' }) !== null) throw Error('Restored database retained post-snapshot mutation.');
  const list = await command('backup_list');
  if (!list.lastRestore || list.pendingRestore !== null || list.files.length < 2) throw Error(`Missing restore/recovery evidence: ${JSON.stringify(list)}`);
  await stop();
  const result = { at: new Date().toISOString(), binary, binarySha256, source: 'real console host with isolated temporary library', checks: ['delayed initial request headers', 'separately arriving request body', 'fresh setup through HTTP', 'create checked snapshot', 'mutate after snapshot', 'queue restore', 'graceful stop and process restart', 'restored staff login', 'post-snapshot mutation absent', 'pre-restore recovery snapshot retained', 'restore request consumed'], ownerDataUsed: false };
  await writeFile('docs/HEADLESS_HOST_REVIEW.json', JSON.stringify(result, null, 2) + '\n');
  console.log(JSON.stringify(result));
} catch (error) {
  throw new Error(`${stage}: ${error.message}; server exit=${child?.exitCode}; output=${output}`, { cause: error });
} finally {
  if (child && child.exitCode === null) { child.kill(); await new Promise(resolve => child.once('exit', resolve)); }
  await rm(data, { recursive: true, force: true, maxRetries: 5, retryDelay: 200 });
}
