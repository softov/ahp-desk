/**
 * Copies the build's UI into src-tauri/resources/ui/<ui>/, where a release
 * build bundles it from. Run before `tauri build`; tauri.conf.json does.
 *
 * - AHP_DESK_UI: ahpw (default) or ahpapp. ahpapp comes in plan p5.
 * - AHP_DESK_AHPW: npm (default), the published @ahpd/web, or local, a built checkout.
 * - AHP_DESK_AHPW_DIR: the checkout for local, default ../ahpw from this workspace.
 */

import { cpSync, existsSync, rmSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');

function setting(name, fallback, allowed) {
  const value = process.env[name] || fallback;
  if (!allowed.includes(value)) {
    console.error(`${name} is "${value}"; use one of ${allowed.join(', ')}`);
    process.exit(1);
  }
  return value;
}

const ui = setting('AHP_DESK_UI', 'ahpw', ['ahpw', 'ahpapp']);
if (ui === 'ahpapp') {
  console.error('AHP_DESK_UI=ahpapp: ahpapp support comes in plan p5');
  process.exit(1);
}
const source = setting('AHP_DESK_AHPW', 'npm', ['npm', 'local']);
const from =
  source === 'local'
    ? join(resolve(root, process.env.AHP_DESK_AHPW_DIR || '../ahpw'), 'dist/app')
    : join(root, 'node_modules/@ahpd/web/dist/app');

if (!existsSync(join(from, 'index.html'))) {
  console.error(
    source === 'local'
      ? `No built ahpw at ${from}: run \`pnpm build\` in the ahpw checkout, or set AHP_DESK_AHPW_DIR.`
      : `No @ahpd/web at ${from}: run \`pnpm install\`.`,
  );
  process.exit(1);
}

const to = join(root, 'src-tauri/resources/ui', ui);
rmSync(to, { recursive: true, force: true });
cpSync(from, to, { recursive: true });
console.log(`staged ${ui} from ${source}: ${from} -> ${to}`);
