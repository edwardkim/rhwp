import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

test('#7977 HF/FN 입력·undo·redo는 보조 평면 문자도 scalar 위치를 유지한다', () => {
  const result = spawnSync(process.execPath, [
    '--experimental-transform-types', '--no-warnings',
    fileURLToPath(new URL('./support/hf-fn-scalar-input.runner.mjs', import.meta.url)),
  ], { encoding: 'utf8' });
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stdout + result.stderr);
  assert.match(result.stdout, /HF_FN_SCALAR_INPUT_OK/);
});
