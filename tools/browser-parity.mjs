import { readFile, writeFile } from 'node:fs/promises';

const root = new URL('../', import.meta.url);
const source = await readFile(new URL('src-tauri/src/lib.rs', root), 'utf8');
const handlers = [...source.matchAll(/\$crate::commands::(\w+)::(\w+)/g)];
const modules = new Map();
for (const [, module] of handlers) {
  if (!modules.has(module)) modules.set(module, await readFile(new URL(`src-tauri/src/commands/${module}.rs`, root), 'utf8'));
}
const native = new Set(['catalogue_save_export', 'reports_save_export', 'backup_copy_to_folder']);
const bootstrap = new Set(['complete_setup', 'setup_status', 'login', 'logout', 'current_session', 'app_info']);
const commands = handlers.map(([, module, command]) => {
  const file = modules.get(module);
  const declaration = new RegExp(`pub (?:async )?fn ${command}\\b`).exec(file);
  if (!declaration) throw Error(`Registered command missing: ${module}::${command}`);
  return {
    command, module, source: `src-tauri/src/commands/${module}.rs`,
    browserBackend: 'not-implemented',
    sharedHandlerReady: file.lastIndexOf('crate::shared_command! {', declaration.index) > file.lastIndexOf('#[tauri::command', declaration.index),
    adaptation: native.has(command) ? 'browser-file-adapter' : bootstrap.has(command) ? 'independent-browser-bootstrap/session' : 'shared-permission-checked-handler',
  };
});
const inventory = {
  schemaVersion: 1,
  scope: 'V3 full staff browser parity; local network and internet; library PC default, optional always-on host',
  source: 'src-tauri/src/lib.rs app_handlers!',
  commandCount: commands.length,
  legacyStaffPortal: 'Pairing/sign-in and issue/return only; does not establish full browser parity',
  commands,
};
const path = new URL('docs/STAFF_BROWSER_COMMAND_INVENTORY.json', root);
const rendered = JSON.stringify(inventory, null, 2) + '\n';
if (process.argv.includes('--check')) {
  if (await readFile(path, 'utf8') !== rendered) throw Error('Staff browser inventory is stale. Run node tools/browser-parity.mjs.');
} else { await writeFile(path, rendered); }
console.log(`${commands.length} registered commands across ${modules.size} modules; browser backend status recorded without completion credit.`);
