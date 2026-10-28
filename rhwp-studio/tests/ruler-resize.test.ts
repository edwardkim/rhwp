import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import test from 'node:test';

test('실제 Ruler의 bitmap 크기 변경·paint 갱신 계약', () => {
  const { NODE_TEST_CONTEXT: _parentContext, ...rulerTestEnv } = process.env;
  const result = spawnSync(process.execPath, [
    '--experimental-transform-types', '--no-warnings', '--test', '--test-reporter=tap',
    fileURLToPath(new URL('./support/ruler-resize.cases.mjs', import.meta.url)),
  ], { encoding: 'utf8', timeout: 30_000, env: rulerTestEnv });
  assert.ifError(result.error);
  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
  assert.match(result.stdout, /# fail 0\b/);
  assert.match(result.stdout, /# skipped 0\b/);
});
