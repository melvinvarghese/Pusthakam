import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const permitted = new Map([
  ['/design/opac/', ['design/opac/index.html', 'text/html; charset=utf-8']],
  ['/design/opac/styles.css', ['design/opac/styles.css', 'text/css; charset=utf-8']],
  ['/design/opac/prototype.js', ['design/opac/prototype.js', 'text/javascript; charset=utf-8']],
  ['/branding/pusthakam-logo.png', ['branding/pusthakam-logo.png', 'image/png']],
  ['/node_modules/@fontsource-variable/hanken-grotesk/files/hanken-grotesk-latin-wght-normal.woff2', ['node_modules/@fontsource-variable/hanken-grotesk/files/hanken-grotesk-latin-wght-normal.woff2', 'font/woff2']],
  ['/node_modules/@fontsource/courier-prime/files/courier-prime-latin-400-normal.woff2', ['node_modules/@fontsource/courier-prime/files/courier-prime-latin-400-normal.woff2', 'font/woff2']],
]);
export const server = createServer(async (request, response) => {
  const path = new URL(request.url, 'http://127.0.0.1').pathname;
  if (request.method !== 'GET') { response.writeHead(405); response.end(); return; }
  if (path === '/') { response.writeHead(302, { Location: '/design/opac/' }); response.end(); return; }
  const entry = permitted.get(path);
  if (!entry) { response.writeHead(404); response.end('Not found'); return; }
  try {
    const bytes = await readFile(resolve(root, entry[0]));
    response.writeHead(200, { 'Content-Type': entry[1], 'Cache-Control': 'no-store', 'X-Content-Type-Options': 'nosniff' });
    response.end(bytes);
  } catch { response.writeHead(404); response.end('File unavailable'); }
});
server.listen(5193, '127.0.0.1', () => console.log('OPAC prototypes: http://127.0.0.1:5193/design/opac/'));
