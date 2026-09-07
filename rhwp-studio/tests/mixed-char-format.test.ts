import test from 'node:test';
import assert from 'node:assert/strict';
import { existsSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

test('#6788 실제 WASM + Studio 혼합 모양 적용/Undo/Redo', (t) => {
  if (!existsSync(fileURLToPath(new URL('../../pkg-node/rhwp.js', import.meta.url)))) {
    t.skip('fresh Node WASM 필요: scripts/wasm-pack-locked.sh --target nodejs --out-dir pkg-node --no-opt');
    return;
  }
  const result = spawnSync(process.execPath, ['--experimental-transform-types', '--no-warnings', fileURLToPath(new URL('./support/mixed-char-format.runner.mjs', import.meta.url))], { encoding: 'utf8' });
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stdout + '\n' + result.stderr);
  assert.match(result.stdout, /MIXED_CHAR_FORMAT_OK/);
});
