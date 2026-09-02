import test from 'node:test';
import assert from 'node:assert/strict';

import {
  PageRenderScheduler,
  type PageRenderIdleDeadline,
  type PageRenderSchedulerHost,
  type PageRenderWork,
} from '../src/view/page-render-scheduler.ts';

class FakeHost implements PageRenderSchedulerHost {
  time = 0;
  nextId = 0;
  frames = new Map<number, () => void>();
  idles = new Map<number, (deadline: PageRenderIdleDeadline) => void>();
  timers = new Map<number, () => void>();
  canceledFrames: number[] = [];
  canceledIdles: number[] = [];
  canceledTimers: number[] = [];

  now(): number { return this.time; }
  requestFrame(callback: () => void): number {
    const id = ++this.nextId;
    this.frames.set(id, callback);
    return id;
  }
  cancelFrame(id: number): void {
    this.frames.delete(id);
    this.canceledFrames.push(id);
  }
  requestIdle(callback: (deadline: PageRenderIdleDeadline) => void): number {
    const id = ++this.nextId;
    this.idles.set(id, callback);
    return id;
  }
  cancelIdle(id: number): void {
    this.idles.delete(id);
    this.canceledIdles.push(id);
  }
  setTimeout(callback: () => void): number {
    const id = ++this.nextId;
    this.timers.set(id, callback);
    return id;
  }
  clearTimeout(id: number): void {
    this.timers.delete(id);
    this.canceledTimers.push(id);
  }

  runFrame(): void {
    const next = this.frames.entries().next().value as [number, () => void] | undefined;
    assert.ok(next, '예약된 frame이 있어야 한다');
    this.frames.delete(next[0]);
    next[1]();
  }

  runIdle(didTimeout = false, remaining = 10): void {
    const next = this.idles.entries().next().value as ([
      number,
      (deadline: PageRenderIdleDeadline) => void,
    ] | undefined);
    assert.ok(next, '예약된 idle callback이 있어야 한다');
    this.idles.delete(next[0]);
    next[1]({ didTimeout, timeRemaining: () => remaining });
  }

  runTimer(): void {
    const next = this.timers.entries().next().value as [number, () => void] | undefined;
    assert.ok(next, '예약된 timeout이 있어야 한다');
    this.timers.delete(next[0]);
    next[1]();
  }
}

function work(
  pageIndex: number,
  priority: number,
  output: number[],
  options: {
    valid?: () => boolean;
    costMs?: number;
    host?: FakeHost;
    workClass?: 'visible' | 'retained-transition' | 'prefetch';
  } = {},
): PageRenderWork {
  return {
    pageIndex,
    priority,
    rasterKey: `page:${pageIndex}`,
    workClass: options.workClass ?? 'prefetch',
    isValid: options.valid ?? (() => true),
    run: () => {
      output.push(pageIndex);
      if (options.host) options.host.time += options.costMs ?? 0;
    },
  };
}

test('많은 visible은 우선순위대로 page 경계에서 분할하고 soft budget 뒤 다음 frame에 양보한다', () => {
  const host = new FakeHost();
  const scheduler = new PageRenderScheduler(host, {
    visibleSliceBudgetMs: 4,
    maxVisiblePagesPerSlice: 2,
  });
  const output: number[] = [];
  scheduler.setDesiredWork(1, [
    work(3, 3, output, { host, costMs: 5 }),
    work(1, 0, output, { host, costMs: 5 }),
    work(2, 1, output, { host, costMs: 5 }),
  ], [], true);

  assert.deepEqual(output, [], '3쪽 이상은 입력 callback에서 바로 raster하지 않는다');
  assert.equal(host.frames.size, 1);
  host.runFrame();
  assert.deepEqual(output, [1], '한 page가 soft budget을 넘으면 그 경계에서 양보한다');
  assert.equal(host.frames.size, 1);
  host.runFrame();
  assert.deepEqual(output, [1, 2]);
  host.runFrame();
  assert.deepEqual(output, [1, 2, 3]);
  assert.equal(scheduler.snapshot().visibleSlices, 3);
});

test('1·2 visible fast path는 동기 실행하고 prefetch는 visible보다 먼저 실행하지 않는다', () => {
  const host = new FakeHost();
  const scheduler = new PageRenderScheduler(host);
  const output: number[] = [];
  scheduler.setDesiredWork(
    1,
    [work(0, 0, output), work(1, 1, output)],
    [work(2, 0, output)],
    true,
  );
  assert.deepEqual(output, [0, 1]);
  assert.equal(host.frames.size, 0);
  assert.equal(host.idles.size, 1);
  host.runIdle();
  assert.deepEqual(output, [0, 1, 2]);
});

test('idle callback 한 번은 prefetch 한 page만 실행하고 deadline 부족이면 양보한다', () => {
  const host = new FakeHost();
  const scheduler = new PageRenderScheduler(host);
  const output: number[] = [];
  scheduler.setDesiredWork(1, [], [
    work(2, 2, output),
    work(0, 0, output),
    work(1, 1, output),
  ], false);

  host.runIdle(false, 0);
  assert.deepEqual(output, []);
  assert.equal(host.idles.size, 1);
  host.runIdle(false, 10);
  assert.deepEqual(output, [0]);
  assert.equal(host.idles.size, 1);
  host.runIdle(true, 0);
  assert.deepEqual(output, [0, 1]);
  host.runIdle(false, 10);
  assert.deepEqual(output, [0, 1, 2]);
});

test('새 generation은 예약된 frame을 재사용하고 같은 page를 중복하지 않아 연속 입력 중에도 진행한다', () => {
  const host = new FakeHost();
  const scheduler = new PageRenderScheduler(host);
  const output: number[] = [];
  scheduler.setDesiredWork(1, [
    work(0, 0, output), work(1, 1, output), work(2, 2, output),
  ], [], false);
  const firstFrameId = [...host.frames.keys()][0];

  scheduler.setDesiredWork(2, [
    work(9, 5, output),
    work(9, 0, output),
    work(8, 1, output),
    work(7, 2, output),
  ], [], false);
  assert.deepEqual([...host.frames.keys()], [firstFrameId], 'rAF를 취소·재예약하지 않는다');
  host.runFrame();
  assert.deepEqual(output, [9, 8], '같은 page는 낮은 priority 작업 하나만 남는다');
  host.runFrame();
  assert.deepEqual(output, [9, 8, 7]);
});

test('새 generation의 작은 fast path가 최신 작업을 끝내면 이전 빈 frame도 회수한다', () => {
  const host = new FakeHost();
  const scheduler = new PageRenderScheduler(host);
  const output: number[] = [];
  scheduler.setDesiredWork(1, [
    work(0, 0, output), work(1, 1, output), work(2, 2, output),
  ], [], false);
  assert.equal(host.frames.size, 1);

  scheduler.setDesiredWork(2, [work(9, 0, output)], [], true);
  assert.deepEqual(output, [9]);
  assert.equal(host.frames.size, 0);
  assert.equal(scheduler.snapshot().frameScheduled, false);
});

test('stale key는 실행하지 않고 cancelAll은 frame과 idle ownership을 모두 회수한다', () => {
  const host = new FakeHost();
  const scheduler = new PageRenderScheduler(host);
  const output: number[] = [];
  scheduler.setDesiredWork(1, [
    work(0, 0, output, { valid: () => false }),
    work(1, 1, output),
    work(2, 2, output),
  ], [], false);
  host.runFrame();
  assert.deepEqual(output, [1, 2]);
  assert.equal(scheduler.snapshot().staleDropped, 1);

  scheduler.setDesiredWork(2, [], [work(3, 0, output)], false);
  assert.equal(host.idles.size, 1);
  scheduler.cancelAll();
  assert.equal(host.frames.size, 0);
  assert.equal(host.idles.size, 0);
  assert.equal(scheduler.snapshot().visibleQueued, 0);
  assert.equal(scheduler.snapshot().prefetchQueued, 0);
});

test('requestIdleCallback이 없으면 timeout fallback도 매번 한 page만 처리한다', () => {
  const host = new FakeHost();
  host.requestIdle = undefined;
  host.cancelIdle = undefined;
  const scheduler = new PageRenderScheduler(host);
  const output: number[] = [];
  scheduler.setDesiredWork(1, [], [work(0, 0, output), work(1, 1, output)], false);
  assert.equal(host.timers.size, 1);
  host.runTimer();
  assert.deepEqual(output, [0]);
  assert.equal(host.timers.size, 1);
  host.runTimer();
  assert.deepEqual(output, [0, 1]);
});

test('예산 때문에 거절한 선택 prefetch를 scheduler 진단에 누적한다', () => {
  const scheduler = new PageRenderScheduler(new FakeHost());
  scheduler.recordPrefetchAdmissionRejected();
  scheduler.recordPrefetchAdmissionRejected(2);
  assert.equal(scheduler.snapshot().prefetchAdmissionRejected, 3);
});

test('scroll settle은 마지막 입력 하나만 debounce하고 cancelAll에서 회수한다', () => {
  const host = new FakeHost();
  const scheduler = new PageRenderScheduler(host, { scrollSettleDelayMs: 150 });
  const settled: number[] = [];

  scheduler.scheduleScrollSettle(() => settled.push(1));
  const firstTimer = [...host.timers.keys()][0]!;
  scheduler.scheduleScrollSettle(() => settled.push(2));
  assert.equal(host.timers.size, 1);
  assert.ok(host.canceledTimers.includes(firstTimer));
  assert.equal(scheduler.snapshot().scrollSettleScheduled, true);

  host.runTimer();
  assert.deepEqual(settled, [2]);
  assert.equal(scheduler.snapshot().scrollSettleScheduled, false);

  scheduler.scheduleScrollSettle(() => settled.push(3));
  scheduler.cancelAll();
  assert.equal(host.timers.size, 0);
  assert.deepEqual(settled, [2]);
});

test('active target-DPR 전환은 speculative idle 뒤에도 task 경계별로 즉시 이어간다', () => {
  const host = new FakeHost();
  const scheduler = new PageRenderScheduler(host);
  const output: number[] = [];
  scheduler.setDesiredWork(1, [], [
    work(4, 0, output, { workClass: 'prefetch' }),
    work(18, 1, output, { workClass: 'retained-transition' }),
    work(19, 2, output, { workClass: 'retained-transition' }),
  ], false);

  assert.equal(host.idles.size, 1);
  host.runIdle();
  assert.deepEqual(output, [4]);
  assert.equal(host.idles.size, 0);
  assert.equal(host.timers.size, 1, '첫 active 전환은 다음 idle frame을 기다리지 않는다');

  host.runTimer();
  assert.deepEqual(output, [4, 18]);
  assert.equal(host.timers.size, 1, '한 task에 한 쪽만 실행하고 다음 전환도 새 task로 양보한다');
  host.runTimer();
  assert.deepEqual(output, [4, 18, 19]);
  assert.equal(host.timers.size, 0);
});
