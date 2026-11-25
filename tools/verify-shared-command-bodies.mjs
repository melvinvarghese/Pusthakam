import { execFileSync } from 'node:child_process';
import { readFile, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';

const base = process.argv.find(arg => arg.startsWith('--base='))?.slice(7);
if (!base || !/^[a-f0-9]{7,40}$/.test(base)) throw Error('Pass the pre-refactor Git commit as --base=<commit>.');
const root = new URL('../', import.meta.url);
const handlers = [...(await readFile(new URL('src-tauri/src/lib.rs', root), 'utf8')).matchAll(/\$crate::commands::(\w+)::(\w+)/g)];

function body(source, name) {
  const declaration = new RegExp(`pub (?:async )?fn ${name}\\b`).exec(source);
  if (!declaration) throw Error(`Missing declaration: ${name}`);
  const start = source.indexOf('{', declaration.index + declaration[0].length);
  let depth = 0;
  for (let i = start; i < source.length; i++) {
    if (source.startsWith('//', i)) { const end = source.indexOf('\n', i + 2); i = end < 0 ? source.length : end; continue; }
    if (source.startsWith('/*', i)) {
      let nested = 1; i += 2;
      while (i < source.length && nested) {
        if (source.startsWith('/*', i)) { nested++; i += 2; }
        else if (source.startsWith('*/', i)) { nested--; i += 2; }
        else i++;
      }
      i--; continue;
    }
    const raw = /^r(#{0,16})"/.exec(source.slice(i));
    if (raw) { const end = source.indexOf('"' + raw[1], i + raw[0].length); if (end < 0) throw Error('Unclosed raw string'); i = end + raw[1].length; continue; }
    if (source[i] === '"') {
      i++;
      while (i < source.length) { if (source[i] === '\\') { i += 2; continue; } if (source[i] === '"') break; i++; }
      continue;
    }
    if (source[i] === "'") { const character = /^'(?:\\u\{[^}]*\}|\\.|[^'\\])'/.exec(source.slice(i)); if (character) { i += character[0].length - 1; continue; } }
    if (source[i] === '{') depth++;
    if (source[i] === '}' && --depth === 0) return source.slice(start, i + 1).replaceAll('\r\n', '\n');
  }
  throw Error(`Unclosed body: ${name}`);
}
const files = new Map();
const results = [];
for (const [, module, command] of handlers) {
  const path = `src-tauri/src/commands/${module}.rs`;
  if (!files.has(path)) files.set(path, {
    before: execFileSync('git', ['show', `${base}:${path}`], { encoding: 'utf8', cwd: root }),
    after: await readFile(new URL(path, root), 'utf8'),
  });
  const { before, after } = files.get(path);
  const original = body(before, command), current = body(after, command);
  const referenceAdapter = module === 'lan' && ['lan_get_config', 'lan_update_config'].includes(command);
  const expected = referenceAdapter ? original.replaceAll('lan_runtime::config(&state)', 'lan_runtime::config(state)') : original;
  if (expected !== current) throw Error(`Command body changed: ${module}::${command}`);
  results.push({ command, source: path, originalBodySha256: createHash('sha256').update(original).digest('hex'), currentBodySha256: createHash('sha256').update(current).digest('hex'), adaptation: referenceAdapter ? 'Remove redundant borrow after State<AppState> becomes &AppState' : 'none' });
}
const report = { at: new Date().toISOString(), baseCommit: base, commands: results.length, exactBodies: results.filter(item => item.adaptation === 'none').length, referenceAdapters: results.filter(item => item.adaptation !== 'none').length, comparison: 'Exact body text after CRLF normalization; two explicit reference adapters for the new borrowed context', results };
await writeFile(new URL('docs/SHARED_COMMAND_PRESERVATION.json', root), JSON.stringify(report, null, 2) + '\n');
console.log(`${report.commands} command bodies checked against ${base}: ${report.exactBodies} exact, ${report.referenceAdapters} explicit reference adapters.`);
