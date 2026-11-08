import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

test('#7983 교차 구역 선택 변경은 문서·선택을 건드리기 전에 거절한다', () => {
  const result = spawnSync(process.execPath, ['--experimental-transform-types', '--no-warnings',
    fileURLToPath(new URL('./support/cross-section-selection.runner.mjs', import.meta.url))], { encoding: 'utf8' });
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stdout + result.stderr);
  assert.match(result.stdout, /CROSS_SECTION_SELECTION_OK/);
});
