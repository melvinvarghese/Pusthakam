// Optional, isolated public-access demo. No owner database or DNS is modified.
import { spawn } from 'node:child_process';
import { mkdtemp, readFile, writeFile, access } from 'node:fs/promises';
import { createHash, randomBytes } from 'node:crypto';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { createServer } from 'node:net';
import { request } from 'node:http';

const pause = ms => new Promise(resolve => setTimeout(resolve, ms));
const localOnly = process.argv.includes('--local-check');
const manifest = JSON.parse(await readFile('docs/INTERNET_TEST_TOOLING.json', 'utf8'));
const cloudflared = manifest.tools.find(tool => tool.repository === 'cloudflare/cloudflared');
const caddyZip = manifest.tools.find(tool => tool.repository === 'caddyserver/caddy');
const caddy = join(manifest.directory, 'caddy', 'caddy.exe');
for (const tool of [cloudflared, caddyZip]) {
  if (!tool || `sha256:${createHash('sha256').update(await readFile(tool.path)).digest('hex')}` !== tool.digest) throw Error('Portable tool integrity mismatch');
}
await access(caddy);
const binary = resolve('src-tauri/target/release/bundle/server/pusthakam-server.exe');
const binarySha256 = createHash('sha256').update(await readFile(binary)).digest('hex');
const directory = await mkdtemp(join(tmpdir(), 'pusthakam-internet-demo-'));
const checks = [];
const children = [];
let server, fatal = false;
async function port() {
  const socket = createServer();
  await new Promise(resolve => socket.listen(0, '127.0.0.1', resolve));
  const value = socket.address().port;
  await new Promise(resolve => socket.close(resolve));
  return value;
}
const backendPort = await port(), proxyPort = await port();
const local = `http://127.0.0.1:${backendPort}`;
function launch(executable, args, name) {
  const child = spawn(executable, args, { windowsHide: true, cwd: directory, stdio: ['pipe', 'pipe', 'pipe'] });
  const entry = { child, name, output: '' };
  child.stdout.on('data', data => { entry.output = (entry.output + data).slice(-100_000); });
  child.stderr.on('data', data => { entry.output = (entry.output + data).slice(-100_000); });
  child.on('error', error => { entry.output += error.message; });
  children.push(entry);
  return entry;
}
async function until(entry, predicate, timeout = 30_000) {
  const started = Date.now();
  while (!predicate(entry.output)) {
    if (entry.child.exitCode !== null || Date.now() - started > timeout) throw Error(`${entry.name}: ${entry.output}`);
    await pause(100);
  }
}
async function start(publicUrl) {
  server = launch(binary, ['--data-dir', directory, '--port', String(backendPort), ...(publicUrl ? ['--public-url', publicUrl] : [])], 'Pusthakam');
  await until(server, output => output.includes('server listening'));
}
async function stopServer() {
  server.child.stdin.end('quit\n');
  await until(server, () => server.child.exitCode !== null);
}
async function cleanup() {
  for (const entry of [...children].reverse()) {
    if (entry.child.exitCode !== null) continue;
    if (entry === server) {
      entry.child.stdin.end('quit\n');
      for (let i = 0; i < 100 && entry.child.exitCode === null; i++) await pause(100);
    }
    if (entry.child.exitCode === null) entry.child.kill();
  }
}
const username = 'internet-test', password = randomBytes(18).toString('base64url');
try {
  console.log('Creating a separate Pusthakam test library...');
  await start();
  const page = await fetch(`${local}/staff`);
  let cookie = page.headers.get('set-cookie').split(';')[0];
  const csrf = (await page.text()).match(/name="pusthakam-staff-csrf" content="([a-f0-9]+)"/)[1];
  async function command(command, args = {}) {
    const response = await fetch(`${local}/api/staff/commands`, { method: 'POST', headers: { Cookie: cookie, Origin: local, 'Content-Type': 'application/json', 'X-CSRF-Token': csrf }, body: JSON.stringify({ command, args }) });
    if (response.headers.get('set-cookie')) cookie = response.headers.get('set-cookie').split(';')[0];
    const result = await response.json();
    if (!response.ok) throw Error(`${command}: ${JSON.stringify(result)}`);
    return result.result;
  }
  await command('complete_setup', { input: { library: { name: 'Pusthakam Internet Test Library', profile: 'school', address: null, contact: null, currency: 'INR', locale: 'en' }, admin: { username, displayName: 'Internet Test Administrator', password }, applyProfileDefaults: true, finePerDayMinor: 100 } });
  await command('catalogue_save_record', { input: { title: 'Pusthakam public access test book', publisher: 'Test library', contributors: [], subjects: [] } });
  checks.push('isolated library setup and sample catalogue through real API');
  await stopServer();
  const config = join(directory, 'Caddyfile');
  await writeFile(config, `{
    admin off
    auto_https off
}
http://:${proxyPort} {
    bind 127.0.0.1
    request_body {
        max_size 140MiB
    }
    @app path / /opac /staff /staff/ /assets/* /api/opac/* /api/staff/commands /api/staff/backups/*
    handle @app {
        reverse_proxy 127.0.0.1:${backendPort} {
            request_buffers 140MiB
            header_up Host {hostport}
            header_up X-Forwarded-Proto https
            header_up X-Forwarded-For {remote_host}
        }
    }
    handle {
        respond "Not found" 404
    }
}
`);
  const proxy = launch(caddy, ['run', '--config', config, '--adapter', 'caddyfile'], 'Caddy');
  await until(proxy, output => output.includes('serving initial configuration'));
  if (!localOnly) console.log('Connecting the free temporary tunnel; this can take a minute...');
  const tunnel = localOnly ? null : launch(cloudflared.path, ['tunnel', '--no-autoupdate', '--protocol', 'http2', '--url', `http://127.0.0.1:${proxyPort}`], 'Cloudflare tunnel');
  if (tunnel) await until(tunnel, output => /https:\/\/[a-z0-9-]+\.trycloudflare\.com/.test(output), 60_000);
  const publicUrl = tunnel ? tunnel.output.match(/https:\/\/[a-z0-9-]+\.trycloudflare\.com/)[0] : 'https://pusthakam-test.invalid';
  await start(publicUrl);
  const proxyRequest = (route, method = 'GET', headers = {}, body = '') => {
    // Node fetch discards an overridden Host; use HTTP request for this proxy check.
    return new Promise((resolve, reject) => {
      const pending = request(`http://127.0.0.1:${proxyPort}${route}`, { method, headers: { ...headers, Host: new URL(publicUrl).host, ...(body ? { 'Content-Length': Buffer.byteLength(body) } : {}) } }, response => {
        let text = '';
        response.setEncoding('utf8');
        response.on('data', chunk => { text += chunk; });
        response.on('end', () => resolve({ status: response.statusCode, cookies: response.headers['set-cookie'] || [], text }));
      });
      pending.setTimeout(10_000, () => pending.destroy(Error('Local proxy request timed out')));
      pending.on('error', reject);
      pending.end(body);
    });
  };
  let staffPage;
  for (const route of ['/opac', '/staff']) {
    const response = await proxyRequest(route);
    if (response.status !== 200) throw Error(`Proxy ${route}: ${response.status}`);
    if (route === '/staff' && !response.cookies.some(cookie => cookie.includes('; Secure'))) throw Error('Public staff cookie is not Secure');
    if (route === '/staff') staffPage = response;
    checks.push(`local proxy ${route} and public-origin headers`);
  }
  const publicCsrf = staffPage.text.match(/name="pusthakam-staff-csrf" content="([a-f0-9]+)"/)[1];
  const login = await proxyRequest('/api/staff/commands', 'POST', { Cookie: staffPage.cookies[0].split(';')[0], Origin: publicUrl, 'X-CSRF-Token': publicCsrf, 'Content-Type': 'application/json' }, JSON.stringify({ command: 'login', args: { username, password } }));
  if (login.status !== 200 || !login.cookies.some(cookie => cookie.includes('; Secure'))) throw Error(`HTTPS-origin proxy login failed: ${login.status}`);
  checks.push('real staff login through proxy with CSRF and Secure session rotation');
  for (const route of ['/api/lan/config', '/api/staff/login', '/api/staff/issue']) {
    const response = await fetch(`http://127.0.0.1:${proxyPort}${route}`);
    if (response.status !== 404) throw Error(`Legacy route exposed: ${route}`);
    checks.push(`proxy blocks ${route}`);
  }
  if (tunnel) await until(tunnel, output => output.includes('Registered tunnel connection'), 60_000);
  if (!localOnly) await writeFile(join(directory, 'test-login.json'), JSON.stringify({ username, password, publicUrl }, null, 2));
  const report = { at: new Date().toISOString(), publicUrl: localOnly ? null : publicUrl, localOnly, binarySha256, checks, directory, processes: localOnly ? [] : children.filter(entry => entry.child.exitCode === null).map(entry => ({ name: entry.name, pid: entry.child.pid })), note: localOnly ? 'Local proxy test with synthetic HTTPS origin; no tunnel or public access.' : 'Temporary isolated demo only. Credentials stored outside repository. Awaiting actual mobile-data acceptance.' };
  await writeFile(localOnly ? 'docs/INTERNET_PROXY_REVIEW.json' : 'docs/INTERNET_ACCESS_TEST.json', JSON.stringify(report, null, 2) + '\n');
  if (localOnly) {
    await cleanup();
    console.log(JSON.stringify(report));
    process.exit(0);
  }
  console.log(JSON.stringify({ ready: true, publicUrl, loginFile: join(directory, 'test-login.json'), checks }));
  console.log(`OPAC: ${publicUrl}/opac`);
  console.log(`Staff: ${publicUrl}/staff`);
  console.log(`Staff login details: ${join(directory, 'test-login.json')}`);
  console.log('Keep this process running for the temporary test. Type quit to stop.');
  process.stdin.setEncoding('utf8');
  process.stdin.on('data', async data => {
    if (data.trim() === 'quit') { await cleanup(); process.exit(0); }
  });
  const timer = setInterval(() => {
    const stopped = [server, proxy, tunnel].find(entry => entry.child.exitCode !== null);
    if (stopped && !fatal) {
      fatal = true;
      clearInterval(timer);
      console.error(`${stopped.name} stopped; temporary test is unavailable.`);
      cleanup().finally(() => process.exit(1));
    }
  }, 2000);
} catch (error) {
  await cleanup();
  await writeFile(join(directory, 'failure.txt'), error.message);
  console.error(error.message);
  console.error(`Test files retained at ${directory}; owner data was not used.`);
  process.exitCode = 1;
}
