/**
 * Puts the Node runtime the desk ships into src-tauri/resources/node/, where a
 * build bundles it from (decision
 * `a-local-ahpd-is-found-or-installed-with-the-bundled-node`). The desk runs npm from it to install ahpd, and ahpd on it.
 *
 * The official build for the OS and CPU this runs on, checked against the
 * release's SHASUMS256.txt, and kept in .cache/node/ so a second run is offline.
 * Run before `tauri dev`, `tauri build`, `pnpm test` and `pnpm lint`; `pnpm stage:all` does.
 */

import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import {
  existsSync,
  lstatSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  readlinkSync,
  renameSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

/** The Node the desk ships: 22, as ahpd's `engines` asks, at its newest patch when this was set. */
const VERSION = 'v22.23.3';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const to = join(root, 'src-tauri/resources/node');
const cache = join(root, '.cache/node');

const ARCHES = { x64: 'x64', arm64: 'arm64' };
const arch = ARCHES[process.arch];
const os = { win32: 'win', linux: 'linux' }[process.platform];
if (arch === undefined || os === undefined) {
  console.error(`stage-node: no Node build is shipped for ${process.platform}-${process.arch}`);
  process.exit(1);
}
const name = `node-${VERSION}-${os}-${arch}`;
const file = `${name}.${os === 'win' ? 'zip' : 'tar.gz'}`;
const stamp = join(to, '.staged');
// Bumped when the staged layout changes, so a runtime staged the old way is staged again.
const staged = `${name}+bin-scripts`;

if (existsSync(stamp) && readFileSync(stamp, 'utf8').trim() === staged) {
  console.log(`stage-node: ${name} is staged at ${to}`);
  process.exit(0);
}

async function download(url) {
  const answer = await fetch(url);
  if (!answer.ok) throw new Error(`${url} answered ${answer.status}`);
  return Buffer.from(await answer.arrayBuffer());
}

mkdirSync(cache, { recursive: true });
const archive = join(cache, file);
const base = `https://nodejs.org/dist/${VERSION}`;
const sums = (await download(`${base}/SHASUMS256.txt`)).toString('utf8');
const expected = sums
  .split('\n')
  .map((line) => line.trim().split(/\s+/))
  .find(([, listed]) => listed === file)?.[0];
if (expected === undefined) throw new Error(`SHASUMS256.txt of ${VERSION} does not list ${file}`);

const sha256 = (bytes) => createHash('sha256').update(bytes).digest('hex');
if (!existsSync(archive) || sha256(readFileSync(archive)) !== expected) {
  console.log(`stage-node: downloading ${base}/${file}`);
  const bytes = await download(`${base}/${file}`);
  const got = sha256(bytes);
  if (got !== expected) throw new Error(`${file} has sha256 ${got}; SHASUMS256.txt says ${expected}`);
  writeFileSync(archive, bytes);
}

// Unpacked beside the target and renamed into place, so a failed run never
// leaves half a runtime that the stamp would vouch for.
const unpack = join(cache, 'unpack');
rmSync(unpack, { recursive: true, force: true });
mkdirSync(unpack, { recursive: true });
// Windows' own tar (bsdtar) reads a zip; another tar on PATH, such as Git's, may not.
const tar = process.platform === 'win32' ? join(process.env.SystemRoot ?? 'C:\\Windows', 'System32', 'tar.exe') : 'tar';
execFileSync(tar, ['-xf', archive, '-C', unpack], { stdio: 'inherit' });
const [top, ...more] = readdirSync(unpack);
if (top !== name || more.length > 0) throw new Error(`${file} did not unpack to ${name}/ alone`);

// The Linux build's bin/npm, bin/npx and bin/corepack are symlinks into lib/.
// Tauri copies a resource through its symlink, so the installed desk would hold
// copies of the scripts whose `require('../lib/cli.js')` then looks beside bin/
// and fails; ahpd runs `npm` by name from this bin/ to install its backends.
// Each link becomes a script that runs its target with this Node, and needs no link.
if (os === 'linux') {
  const bin = join(unpack, top, 'bin');
  for (const tool of readdirSync(bin)) {
    const at = join(bin, tool);
    if (!lstatSync(at).isSymbolicLink()) continue;
    const target = readlinkSync(at);
    rmSync(at);
    writeFileSync(at, `#!/bin/sh\nhere=$(dirname "$(readlink -f "$0")")\nexec "$here/node" "$here/${target}" "$@"\n`, {
      mode: 0o755,
    });
  }
  execFileSync(join(bin, 'npm'), ['--version'], { stdio: 'inherit' });
}

rmSync(to, { recursive: true, force: true });
mkdirSync(dirname(to), { recursive: true });
renameSync(join(unpack, top), to);
rmSync(unpack, { recursive: true, force: true });
writeFileSync(stamp, `${staged}\n`);
console.log(`stage-node: staged ${name} at ${to}`);
