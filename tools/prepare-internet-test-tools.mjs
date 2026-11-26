// Download official portable tools for the optional internet-access test.
import { mkdtemp, writeFile, readFile, mkdir } from 'node:fs/promises';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { createHash } from 'node:crypto';
import { tmpdir, homedir } from 'node:os';
import { join } from 'node:path';

const directory = await mkdtemp(join(tmpdir(), 'pusthakam-public-tools-'));
async function download(repository, select) {
  const response = await fetch(`https://api.github.com/repos/${repository}/releases/latest`, { signal: AbortSignal.timeout(30_000) });
  if (!response.ok) throw Error(`${repository} release lookup: ${response.status}`);
  const release = await response.json();
  const asset = release.assets.find(select);
  if (!asset || !/^sha256:[a-f0-9]{64}$/.test(asset.digest || '')) throw Error(`No official SHA256 digest for ${repository}`);
  console.log(`Checking ${repository} ${release.tag_name}`);
  let bytes;
  try { bytes = await readFile(join(homedir(), 'Downloads', asset.name)); }
  catch (error) { if (error.code !== 'ENOENT') throw error; }
  if (!bytes) {
    console.log(`Downloading ${asset.name} from its official release`);
    const content = await fetch(asset.browser_download_url, { signal: AbortSignal.timeout(120_000) });
    if (!content.ok) throw Error(`${asset.name} download: ${content.status}`);
    bytes = Buffer.from(await content.arrayBuffer());
  }
  const digest = `sha256:${createHash('sha256').update(bytes).digest('hex')}`;
  if (digest !== asset.digest || bytes.length !== asset.size) throw Error(`${asset.name} integrity check failed`);
  const path = join(directory, asset.name);
  await writeFile(path, bytes, { flag: 'wx' });
  return { repository, version: release.tag_name, path, url: asset.browser_download_url, bytes: bytes.length, digest };
}
const results = await Promise.allSettled([
  download('cloudflare/cloudflared', asset => asset.name === 'cloudflared-windows-amd64.exe'),
  download('caddyserver/caddy', asset => /_windows_amd64\.zip$/.test(asset.name)),
]);
const failures = results.filter(result => result.status === 'rejected');
if (failures.length) throw Error(failures.map(result => result.reason.message).join('; '));
const tools = results.map(result => result.value);
const caddyDirectory = join(directory, 'caddy');
await mkdir(caddyDirectory);
await promisify(execFile)('tar.exe', ['-xf', tools[1].path, '-C', caddyDirectory], { windowsHide: true });
const report = { at: new Date().toISOString(), directory, tools, note: 'Portable test tools only; no service, PATH, DNS or firewall changes.' };
await writeFile('docs/INTERNET_TEST_TOOLING.json', JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report));
