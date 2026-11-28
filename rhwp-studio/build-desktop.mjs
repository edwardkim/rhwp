import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import {
  cpSync, existsSync, lstatSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, renameSync, rmSync, writeFileSync,
} from 'node:fs';
import { dirname, join, resolve } from 'node:path';

const studio = import.meta.dirname;
const root = resolve(studio, '..');
const output = resolve(process.argv[2] ?? join(studio, 'dist-desktop'));
const wasm = join(root, 'pkg', 'rhwp_bg.wasm');
const vite = join(studio, 'node_modules', 'vite', 'bin', 'vite.js');
if (!existsSync(wasm) || !existsSync(join(root, 'pkg', 'rhwp.js'))) {
  throw new Error('Build the locked Rust WASM package into pkg/ before building desktop Studio.');
}
if (!existsSync(vite)) {
  throw new Error('Install Studio dependencies with npm ci --prefix rhwp-studio.');
}
if (existsSync(output)) {
  throw new Error(`Studio bundle output already exists: ${output}`);
}

const hash = (path) => createHash('sha256').update(readFileSync(path)).digest('hex');
const coreSha256 = hash(wasm);
const fonts = join(root, 'assets', 'fonts');
mkdirSync(dirname(output), { recursive: true });
const staging = mkdtempSync(join(dirname(output), '.studio-desktop-'));
try {
  const publicDir = join(staging, 'public');
  const bundle = join(staging, 'bundle');
  const dist = join(bundle, 'studio');
  const sourcePublic = join(studio, 'public');
  // Windows checkouts can represent public/fonts' Git symlink as a plain text file.
  // Vite bundles the current core via pkg/, so skip the legacy public bindings as well.
  const replacedPublicPaths = new Set(
    ['fonts', 'rhwp.js', 'rhwp.d.ts', 'rhwp_bg.wasm', 'rhwp_bg.wasm.d.ts']
      .map((name) => join(sourcePublic, name)),
  );
  cpSync(sourcePublic, publicDir, {
    recursive: true,
    filter: (path) => !replacedPublicPaths.has(path) && !lstatSync(path).isSymbolicLink(),
  });
  cpSync(fonts, join(publicDir, 'fonts'), { recursive: true });

  execFileSync(process.execPath, [vite, 'build', '--base', './', '--outDir', dist], {
    cwd: studio,
    stdio: 'inherit',
    env: { ...process.env, RHWP_SUBSECOND: '0', RHWP_DESKTOP_PUBLIC_DIR: publicDir },
  });

  for (const path of ['index.html', 'theme-init.js', 'locale-init.js', 'print.html', 'fonts/licenses']) {
    if (!existsSync(join(dist, path))) {
      throw new Error(`Desktop Studio asset missing: ${path}`);
    }
  }
  const wasmAssets = readdirSync(join(dist, 'assets'))
    .filter((name) => /^rhwp_bg-.*\.wasm$/.test(name));
  if (wasmAssets.length !== 1 || hash(join(dist, 'assets', wasmAssets[0])) !== coreSha256) {
    throw new Error('Desktop Studio did not package the WASM from pkg/rhwp_bg.wasm.');
  }

  for (const license of ['LICENSE', 'THIRD_PARTY_LICENSES.md']) {
    cpSync(join(root, license), join(bundle, license));
  }
  writeFileSync(join(bundle, 'bundle.json'), JSON.stringify({
    entry: 'studio/index.html',
    wasm: `studio/assets/${wasmAssets[0]}`,
    wasmSha256: coreSha256,
  }, null, 2) + '\n');

  renameSync(bundle, output);
  console.log(`Offline Studio bundle: ${output}`);
} finally {
  rmSync(staging, { recursive: true, force: true });
}
