import test from 'node:test';
import assert from 'node:assert/strict';

import { singleFlight } from '../src/command/single-flight.ts';

/** 외부에서 완료 시점을 정하는 promise. */
function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((res, rej) => { resolve = res; reject = rej; });
  return { promise, resolve, reject };
}

test('진행 중이면 두 번째 요청은 작업을 다시 실행하지 않는다', async () => {
  const gate = deferred<string>();
  let calls = 0;
  const guarded = singleFlight(async () => { calls += 1; return gate.promise; });

  const first = guarded();
  const second = guarded();

  // 앞선 요청이 끝나기 전이므로 두 번째는 건너뛴다 — picker 중복 호출이 나가지 않는다.
  assert.equal(await second, undefined);
  assert.equal(calls, 1);

  gate.resolve('done');
  assert.equal(await first, 'done');
  assert.equal(calls, 1);
});

test('앞선 요청이 끝나면 다음 요청은 정상 실행된다', async () => {
  let calls = 0;
  const guarded = singleFlight(async () => { calls += 1; return calls; });

  assert.equal(await guarded(), 1);
  assert.equal(await guarded(), 2);
  assert.equal(calls, 2);
});

test('작업이 예외를 던져도 가드가 풀린다', async () => {
  let calls = 0;
  const guarded = singleFlight(async () => {
    calls += 1;
    if (calls === 1) throw new Error('첫 시도 실패');
    return 'ok';
  });

  await assert.rejects(() => guarded(), /첫 시도 실패/);
  // 한 번 실패했다고 이후 열기가 영구히 막히면 안 된다.
  assert.equal(await guarded(), 'ok');
  assert.equal(calls, 2);
});

test('인자를 그대로 전달한다', async () => {
  const seen: unknown[][] = [];
  const guarded = singleFlight(async (...args: unknown[]) => { seen.push(args); return args.length; });

  assert.equal(await guarded('services', { id: 'x' }), 2);
  assert.deepEqual(seen, [['services', { id: 'x' }]]);
});

test('건너뛴 요청은 앞선 요청의 결과를 가져오지 않는다', async () => {
  const gate = deferred<string>();
  const guarded = singleFlight(async () => gate.promise);

  const first = guarded();
  const skipped = await guarded();
  gate.resolve('첫 결과');

  // 건너뛴 호출은 undefined — 호출자가 "실행되지 않았음"을 구분할 수 있어야 한다.
  assert.equal(skipped, undefined);
  assert.equal(await first, '첫 결과');
});
