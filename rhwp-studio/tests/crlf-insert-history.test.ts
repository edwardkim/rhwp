import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

// #8001: 코어가 정규화한 입력과 명령의 커서·Undo 길이가 같아야 한다.
test('CRLF 입력의 본문·셀·중첩 셀 커서와 Undo/Redo는 뒤 글자를 보존한다', () => {
  const runner = fileURLToPath(new URL('./support/crlf-insert-history.runner.mjs', import.meta.url));
  const result = spawnSync(process.execPath,
    ['--experimental-transform-types', '--no-warnings', runner], { encoding: 'utf8' });
  assert.ifError(result.error);
  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
  assert.match(result.stdout, /CRLF_INSERT_HISTORY_OK/);
});
