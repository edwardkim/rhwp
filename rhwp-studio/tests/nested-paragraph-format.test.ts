import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

test('중첩 표 문단 정렬: 실제 WASM, 셀 선택, Undo/Redo, 저장 왕복', () => {
  const result = spawnSync(process.execPath, [
    '--no-warnings',
    fileURLToPath(new URL('./support/nested-paragraph-format.runner.mjs', import.meta.url)),
  ], { encoding: 'utf8' });
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stdout + '\n' + result.stderr);
  assert.match(result.stdout, /NESTED_PARAGRAPH_FORMAT_OK/);
});
