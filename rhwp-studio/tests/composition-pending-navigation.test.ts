import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

test('IME 종료 후 예약 탐색은 본문·머리말/꼬리말·각주의 키보드 소유자를 따른다', (t) => {
  if (!process.allowedNodeEnvironmentFlags.has('--experimental-transform-types')) {
    t.skip('이 행위 러너는 --experimental-transform-types를 지원하는 Node 24에서 실행한다');
    return;
  }
  const runner = fileURLToPath(new URL('./support/composition-pending-navigation.runner.mjs', import.meta.url));
  const result = spawnSync(process.execPath, ['--experimental-transform-types', '--no-warnings', runner], {
    encoding: 'utf8',
  });
  assert.equal(result.status, 0, `IME 예약 탐색 행위 검증 실패:\n${result.stdout}\n${result.stderr}`);
  assert.match(result.stdout, /COMPOSITION_PENDING_NAVIGATION_OK/);
});
