import test from 'node:test';
import assert from 'node:assert/strict';
import { PassThrough } from 'node:stream';
import { closeBrowser } from '../e2e/helpers.mjs';

// 관계형 검사: 로컬 검증 브라우저의 종료가 확인된 뒤에만 그 자식 stdio를 닫는다.
// 원격 사용자 브라우저와 종료 실패 상태의 스트림 소유권은 가져오지 않는다.
test('종료한 로컬 브라우저가 상속시킨 파이프를 검증 프로세스에 남기지 않는다', async () => {
  const child = { stdout: new PassThrough(), stderr: new PassThrough(), exitCode: null as number | null, signalCode: null };
  const browser = {
    process: () => child,
    async close() {
      assert.equal(child.stdout.destroyed, false);
      assert.equal(child.stderr.destroyed, false);
      child.exitCode = 0;
    },
  };
  await closeBrowser(browser);
  assert.equal(child.stdout.destroyed, true);
  assert.equal(child.stderr.destroyed, true);
});

test('원격 사용자 브라우저는 검증 탭과 CDP 연결만 정리한다', async () => {
  const calls: string[] = [];
  const browser = { _isRemote: true, _testPages: [{ async close() { calls.push('page'); } }],
    disconnect() { calls.push('disconnect'); }, process() { throw new Error('remote process ownership'); } };
  await closeBrowser(browser);
  assert.deepEqual(calls, ['page', 'disconnect']);
  assert.deepEqual(browser._testPages, []);
});

test('브라우저 종료 실패를 스트림 정리로 성공 처리하지 않는다', async () => {
  const child = { stdout: new PassThrough(), stderr: new PassThrough(), exitCode: null, signalCode: null };
  try {
    await assert.rejects(closeBrowser({ process: () => child,
      async close() { throw new Error('browser close failed'); } }), /browser close failed/);
    assert.equal(child.stdout.destroyed, false);
    assert.equal(child.stderr.destroyed, false);
  } finally { child.stdout.destroy(); child.stderr.destroy(); }
});

test('종료가 확인되지 않은 로컬 자식 스트림은 닫지 않는다', async () => {
  const child = { stdout: new PassThrough(), stderr: new PassThrough(), exitCode: null, signalCode: null };
  try {
    await closeBrowser({ process: () => child, async close() {} });
    assert.equal(child.stdout.destroyed, false);
    assert.equal(child.stderr.destroyed, false);
  } finally { child.stdout.destroy(); child.stderr.destroy(); }
});
