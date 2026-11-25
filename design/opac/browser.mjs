import { spawn } from 'node:child_process';
import { mkdtemp, readFile, rm, access } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve, relative, isAbsolute } from 'node:path';
const pause = ms => new Promise(resolve => setTimeout(resolve, ms));
export async function browserReview(run) {
  const candidates = [process.env.OPAC_REVIEW_BROWSER, 'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe', 'C:/Program Files/Google/Chrome/Application/chrome.exe', '/usr/bin/chromium'].filter(Boolean);
  let executable;
  for (const candidate of candidates) { try { await access(candidate); executable = candidate; break; } catch { /* Next installed browser. */ } }
  if (!executable) throw Error('Set OPAC_REVIEW_BROWSER to an installed Chromium browser.');
  const profile = await mkdtemp(join(tmpdir(), 'pusthakam-opac-review-'));
  const withinTemp = relative(resolve(tmpdir()), resolve(profile));
  if (!withinTemp || withinTemp.startsWith('..') || isAbsolute(withinTemp)) throw Error('Review profile outside temporary directory.');
  const child = spawn(executable, ['--headless=new', '--no-sandbox', '--disable-gpu', '--remote-debugging-port=0', `--user-data-dir=${profile}`, '--no-first-run', '--no-default-browser-check', '--disable-background-networking', '--disable-extensions', '--disable-component-extensions-with-background-pages', '--disable-features=msEdgeSidebarV2,msEdgeTranslate,msEdgeWallet', 'about:blank'], { windowsHide: true, stdio: 'ignore' });
  let socket;
  try {
    let port;
    for (let attempt = 0; attempt < 100; attempt++) {
      try { port = Number((await readFile(join(profile, 'DevToolsActivePort'), 'utf8')).split('\n')[0]); break; } catch { await pause(100); }
    }
    if (!port) throw Error('Isolated review browser failed to start.');
    const targets = await fetch(`http://127.0.0.1:${port}/json/list`).then(response => response.json());
    socket = new WebSocket(targets.find(item => item.type === 'page' && item.url === 'about:blank').webSocketDebuggerUrl);
    await new Promise((resolve, reject) => { socket.onopen = resolve; socket.onerror = reject; });
    const pending = new Map(); let next = 0;
    const exceptions = [], requests = [];
    socket.onmessage = event => {
      const message = JSON.parse(event.data);
      if (message.method === 'Runtime.exceptionThrown') exceptions.push(message.params.exceptionDetails.text);
      if (message.method === 'Network.requestWillBeSent') requests.push(message.params.request.url);
      const item = pending.get(message.id);
      if (!item) return;
      clearTimeout(item.timer); pending.delete(message.id);
      if (message.error) item.reject(Error(JSON.stringify(message.error))); else item.resolve(message.result);
    };
    const send = (method, params = {}) => new Promise((resolve, reject) => {
      const id = ++next;
      const timer = setTimeout(() => { pending.delete(id); reject(Error(`Timed out: ${method}`)); }, 15000);
      pending.set(id, { resolve, reject, timer }); socket.send(JSON.stringify({ id, method, params }));
    });
    const evaluate = async expression => {
      const result = await send('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true });
      if (result.exceptionDetails) throw Error(JSON.stringify(result.exceptionDetails));
      return result.result.value;
    };
    const waitFor = async (expression, timeoutMs = 5000) => {
      const deadline = Date.now() + timeoutMs;
      while (Date.now() < deadline) { if (await evaluate(expression)) return; await pause(50); }
      throw Error(`Browser condition not met: ${expression}`);
    };
    const assert = async (expression, label) => { if (!await evaluate(expression)) throw Error(label); };
    await send('Page.enable'); await send('Runtime.enable'); await send('Network.enable');
    try { return await run({ send, evaluate, waitFor, assert, pause, exceptions, requests }); }
    finally { try { await send('Browser.close'); } catch { /* Owned browser closes its debugger. */ } }
  } finally {
    socket?.close(); child.kill();
    await rm(profile, { recursive: true, force: true, maxRetries: 10, retryDelay: 200 }).catch(() => {});
  }
}
