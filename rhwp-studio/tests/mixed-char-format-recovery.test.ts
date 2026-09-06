import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

test('#6814 command/history/UI 오류 회귀 — WASM 산출물 없이 필수 실행', () => {
  const result = spawnSync(process.execPath, ['--experimental-transform-types', '--no-warnings',
    fileURLToPath(new URL('./support/mixed-char-format-recovery.runner.mjs', import.meta.url))], { encoding: 'utf8' });
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stdout + '\n' + result.stderr);
  assert.match(result.stdout, /CHAR_FORMAT_RECOVERY_OK/);
});
