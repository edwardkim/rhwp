import test from 'node:test';
import assert from 'node:assert/strict';
import { ZoomInputSettle, type ZoomInputSettleHost } from '../src/view/zoom-input-settle.ts';

function fixture(quietMs = 120) {
  let at = 0;
  let nextId = 0;
  const timers = new Map<number, { at: number; callback: () => void }>();
  const captured: (() => void)[] = [];
  const ready: number[] = [];
  const host: ZoomInputSettleHost = {
    now: () => at,
    schedule(callback, delay) {
      const id = ++nextId;
      timers.set(id, { at: at + delay, callback });
      captured.push(callback);
      return id as unknown as ReturnType<typeof setTimeout>;
    },
    cancel: id => { timers.delete(id as unknown as number); },
  };
  const settle = new ZoomInputSettle(generation => ready.push(generation), host, quietMs);
  const advance = (value: number) => {
    at = value;
    for (const [id, timer] of timers) {
      if (timer.at <= at) { timers.delete(id); timer.callback(); }
    }
  };
  return { settle, ready, timers, captured, advance };
}

for (const quiet of [80, 120, 160]) {
  test(`${quiet}ms: 작은 입력마다 수렴해도 quiet 뒤 ready는 한 번이다`, () => {
    const f = fixture(quiet);
    for (let i = 0; i < 20; i++) {
      f.advance(i * 16);
      const generation = f.settle.input();
      f.settle.converge(generation);
      assert.equal(f.settle.pending, true);
      assert.equal(f.settle.inputActive, true);
      assert.equal(f.timers.size, 1);
      assert.deepEqual(f.ready, []);
    }
    f.advance(19 * 16 + quiet - 1);
    assert.deepEqual(f.ready, []);
    f.advance(19 * 16 + quiet);
    assert.deepEqual(f.ready, [20]);
    assert.equal(f.settle.pending, false);
    assert.equal(f.timers.size, 0);
    f.settle.converge(20);
    f.advance(2000);
    assert.deepEqual(f.ready, [20]);
  });
}

test('quiet가 수렴보다 먼저면 추가 timer/rAF 없이 수렴 통지를 기다린다', () => {
  const f = fixture();
  const generation = f.settle.input();
  f.advance(120);
  assert.equal(f.settle.inputActive, false);
  assert.equal(f.settle.pending, true);
  assert.equal(f.timers.size, 0);
  assert.deepEqual(f.ready, []);
  f.settle.converge(generation);
  assert.deepEqual(f.ready, [generation]);
});

test('stale timer/수렴은 역방향 새 입력과 취소 후 세대를 완료하지 않는다', () => {
  const f = fixture();
  const old = f.settle.input();
  f.settle.converge(old);
  const oldTimer = f.captured[0];
  f.advance(16);
  const current = f.settle.input();
  oldTimer();
  f.settle.converge(old);
  f.advance(136);
  assert.deepEqual(f.ready, []);
  assert.equal(f.settle.pending, true);
  f.settle.converge(current);
  assert.deepEqual(f.ready, [current]);
  f.settle.input();
  const cancelled = f.captured.at(-1)!;
  assert.equal(f.settle.cancel(), true);
  cancelled();
  f.advance(1000);
  assert.equal(f.timers.size, 0);
  assert.deepEqual(f.ready, [current]);
});

test('충분히 떨어진 두 입력 구간은 각각 한 번 완료한다', () => {
  const f = fixture();
  f.settle.converge(f.settle.input());
  f.advance(120);
  f.advance(500);
  f.settle.converge(f.settle.input());
  f.advance(620);
  assert.equal(f.ready.length, 2);
});

test('명시적 flush는 대기를 끝내고 취소한 timer는 다시 완료하지 않는다', () => {
  const f = fixture();
  f.settle.input();
  const timer = f.captured[0];
  f.settle.flush();
  timer();
  f.settle.flush();
  assert.deepEqual(f.ready, [1]);
  assert.equal(f.timers.size, 0);
});

test('ready callback의 재진입은 새 pending을 잃지 않는다', () => {
  let settle: ZoomInputSettle;
  const timers = new Map<ReturnType<typeof setTimeout>, () => void>();
  let at = 0;
  let n = 0;
  const host: ZoomInputSettleHost = {
    now: () => at,
    schedule: callback => { const id = ++n as unknown as ReturnType<typeof setTimeout>; timers.set(id, callback); return id; },
    cancel: id => { timers.delete(id); },
  };
  settle = new ZoomInputSettle(() => { settle.input(); }, host);
  settle.converge(settle.input());
  at = 120;
  [...timers.values()][0]();
  assert.equal(settle.pending, true);
  assert.equal(settle.generation, 2);
  assert.equal(timers.size, 1);
});
