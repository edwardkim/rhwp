import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import test from 'node:test';

test('실제 Ruler의 pointer 입력·취소·commit 계약', () => {
  // Ruler 생성자의 TS parameter property를 실행하는 기존 저장소의 행위 러너 방식.
  // 부모 runner의 IPC reporter 문맥을 물려주지 않고 자식의 TAP 결과를 직접 판정한다.
  const { NODE_TEST_CONTEXT: _parentContext, ...rulerTestEnv } = process.env;
  const result = spawnSync(process.execPath, [
    '--experimental-transform-types', '--no-warnings', '--test', '--test-reporter=tap',
    fileURLToPath(new URL('./support/ruler-input.cases.mjs', import.meta.url)),
  ], { encoding: 'utf8', timeout: 30_000, env: rulerTestEnv });
  assert.ifError(result.error);
  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
  assert.match(result.stdout, /# fail 0\b/);
  assert.match(result.stdout, /# skipped 0\b/);
});
