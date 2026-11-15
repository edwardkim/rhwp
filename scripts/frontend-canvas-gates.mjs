// #9097: browser-only checks share the frontend job's optimized WASM.
import { createHash } from 'node:crypto';
import { copyFileSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));

export function verifyCanvasWasm(manifestPath, repositoryRoot = root) {
  const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));
  if (manifest.success !== true || manifest.profile !== 'release'
      || manifest.build_configuration?.build_mode !== 'standard'
      || !Array.isArray(manifest.tool_commands?.['wasm-opt'])
      || manifest.tool_commands['wasm-opt'].length === 0) {
    throw new Error('Frontend Canvas gates require a successful optimized release WASM manifest');
  }
  for (const name of ['rhwp.js', 'rhwp_bg.wasm']) {
    const expected = manifest.artifacts?.[name]?.sha256;
    const actual = createHash('sha256').update(readFileSync(path.join(repositoryRoot, 'pkg', name))).digest('hex');
    if (!/^[a-f0-9]{64}$/.test(expected || '') || actual !== expected) {
      throw new Error(`Frontend Canvas WASM artifact mismatch: ${name}`);
    }
  }
  return manifest;
}

function run(command, args, cwd, env) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, { cwd, env, stdio: 'inherit' });
    child.once('error', reject);
    child.once('exit', (code, signal) => code === 0 && !signal
      ? resolve() : reject(new Error(`${command} failed: ${signal || code}`)));
  });
}

export async function runFrontendCanvasGates() {
  const manifestPath = process.env.RHWP_WASM_BUILD_MANIFEST;
  if (!manifestPath) throw new Error('RHWP_WASM_BUILD_MANIFEST is required');
  const manifest = verifyCanvasWasm(manifestPath);
  const output = path.join(root, 'output', 'frontend-canvas-gates');
  mkdirSync(output, { recursive: true });
  // No build/download here: use the exact package already checked above.
  for (const name of ['rhwp.js', 'rhwp_bg.wasm', 'rhwp.d.ts', 'rhwp_bg.wasm.d.ts']) {
    copyFileSync(path.join(root, 'pkg', name), path.join(root, 'rhwp-studio', 'public', name));
  }
  const { startViteDevServer } = await import('../rhwp-studio/e2e/vite-server.mjs');
  const server = await startViteDevServer();
  const checks = [];
  try {
    const suites = [
      ['canvas-font-alias-issue9077.test.mjs'],
      ['canvas-render-diff.test.mjs'],
      ['renderer-baseline.mjs', `--manifest=${path.join(root, 'scripts', 'renderer_baseline_manifest.json')}`,
        '--profiles=screen', '--readiness-only', `--output=${path.join(output, 'canvaskit-readiness')}`],
    ];
    for (const [name, ...args] of suites) {
      await run(process.execPath, [path.join(root, 'rhwp-studio', 'e2e', name), '--mode=headless', ...args],
        path.join(root, 'rhwp-studio'),
        { ...process.env, VITE_URL: server.url });
      checks.push(name);
    }
  } finally {
    await server.stop();
  }
  writeFileSync(path.join(output, 'result.json'), JSON.stringify({
    success: true, source_sha: manifest.source_sha, artifacts: manifest.artifacts, checks,
  }, null, 2) + '\n');
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  await runFrontendCanvasGates();
}
