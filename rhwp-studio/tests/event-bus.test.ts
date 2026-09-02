import test from 'node:test';
import assert from 'node:assert/strict';

import { EventBus } from '../src/core/event-bus.ts';

/** `console.error` 를 가로채 한 번의 배달 동안 보고된 줄을 모은다. */
function captureConsoleErrors(run: () => void): { reported: unknown[][]; thrown: unknown } {
  const reported: unknown[][] = [];
  const original = console.error;
  console.error = (...args: unknown[]) => {
    reported.push(args);
  };
  let thrown: unknown;
  try {
    run();
  } catch (error) {
    thrown = error;
  } finally {
    console.error = original;
  }
  return { reported, thrown };
}

test('a throwing subscriber does not stop delivery to the rest', () => {
  const bus = new EventBus();
  const called: string[] = [];

  bus.on('document-changed', () => {
    called.push('first');
  });
  bus.on('document-changed', () => {
    called.push('second');
    throw new Error('구독자 고장');
  });
  bus.on('document-changed', () => {
    called.push('third');
  });

  const { thrown } = captureConsoleErrors(() => bus.emit('document-changed'));

  assert.deepEqual(
    called,
    ['first', 'second', 'third'],
    '앞선 구독자가 던져도 남은 구독자는 모두 불려야 한다',
  );
  assert.match(String(thrown), /구독자 고장/, '실패는 삼키지 않고 발행자에게 그대로 전달한다');
});

test('the delivery failure names the event, which the rethrown error cannot', () => {
  const bus = new EventBus();
  bus.on('document-view-changed', () => {
    throw new Error('재도색 실패');
  });

  const { reported, thrown } = captureConsoleErrors(() => bus.emit('document-view-changed'));

  assert.equal(reported.length, 1);
  assert.match(
    String(reported[0][0]),
    /document-view-changed/,
    `어느 이벤트의 구독자였는지 보고에 있어야 한다: ${String(reported[0][0])}`,
  );
  assert.equal(reported[0][1], thrown, '보고에는 원래 오류 객체가 그대로 실린다');
});

test('every failure is reported and the first one is the one that propagates', () => {
  const bus = new EventBus();
  const first = new Error('첫 실패');
  const second = new Error('둘째 실패');
  bus.on('document-changed', () => {
    throw first;
  });
  bus.on('document-changed', () => {
    throw second;
  });

  const { reported, thrown } = captureConsoleErrors(() => bus.emit('document-changed'));

  assert.equal(reported.length, 2, '두 실패 모두 보고돼야 한다 — 되풀림은 하나만 나른다');
  assert.equal(thrown, first);
});

test('emit with no failing subscriber neither reports nor throws', () => {
  const bus = new EventBus();
  let calls = 0;
  const unsubscribe = bus.on('zoom-changed', () => {
    calls += 1;
  });

  const { reported, thrown } = captureConsoleErrors(() => bus.emit('zoom-changed', 1.5));

  assert.equal(calls, 1);
  assert.deepEqual(reported, []);
  assert.equal(thrown, undefined);

  unsubscribe();
  bus.emit('zoom-changed', 2);
  assert.equal(calls, 1, '해지한 구독자는 다시 불리지 않는다');
});
