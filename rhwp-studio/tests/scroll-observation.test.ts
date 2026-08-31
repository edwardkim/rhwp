import test from 'node:test';
import assert from 'node:assert/strict';
import { currentImageRequest, flowImageState, imageCompletion, observeBoundary, ScrollObservation, surfacePixels, viewportApplied } from '../src/dev/scroll-observation.ts';

test('관찰은 this·인수·반환 값과 원래 prototype descriptor를 보존한다', () => {
  class Host { n = 3; run(a: number) { return this.n + a; } }
  const host = new Host();
  const original = host.run;
  const calls: unknown[] = [];
  const restore = observeBoundary(host, 'run', () => 1, () => 1, call => calls.push(call), assert.fail);
  assert.equal(host.run(4), 7);
  assert.equal(calls.length, 1);
  restore();
  assert.equal(host.run, original);
  assert.equal(Object.hasOwn(host, 'run'), false);
});

test('관찰은 동일 Promise와 thrown undefined까지 보존한다', async () => {
  const promise = Promise.resolve(4);
  const host = { run() { return promise; }, fail() { throw undefined; } };
  const calls: { failed: boolean }[] = [];
  observeBoundary(host, 'run', () => 1, () => null, call => calls.push(call), assert.fail);
  observeBoundary(host, 'fail', () => 1, () => null, call => calls.push(call), assert.fail);
  assert.equal(host.run(), promise);
  let thrown = false;
  try { host.fail(); } catch (error) { thrown = true; assert.equal(error, undefined); }
  assert.equal(thrown, true);
  assert.deepEqual(calls.map(c => c.failed), [false, true]);
  await promise;
});

test('관찰 실패·해제가 제품 결과나 다른 wrapper를 바꾸지 않는다', () => {
  const host = { run() { return 4; } };
  const errors: unknown[] = [];
  const restore = observeBoundary(host, 'run', () => 1, () => null, () => { throw Error('probe'); }, e => errors.push(e));
  assert.equal(host.run(), 4);
  assert.equal(errors.length, 1);
  const another = () => 5;
  host.run = another;
  restore();
  assert.equal(host.run, another);
});

test('늦은 generation·완료 이후 counter/mark는 다음 interaction을 오염시키지 않는다', () => {
  const trace = new ScrollObservation();
  const a = trace.begin('doc1/rev1', 'scroll', 1);
  const b = trace.begin('doc2/rev1', 'zoom', 2);
  trace.mark(a, 'visibleStable', 3);
  trace.count(a, 'raster', 3, 4);
  trace.mark(b, 'visibleStable', 4);
  trace.mark(b, 'retainedComplete', 5);
  trace.finish('complete');
  trace.count(b, 'raster', 4, 9);
  const [old, current] = trace.snapshot();
  assert.equal(old.status, 'superseded');
  assert.deepEqual(current.counters, {});
  assert.deepEqual(current.milestones, { visibleStable: 2, retainedComplete: 3 });
});

test('완료는 명시적 readiness가 필요하고 timeout/중단을 성공으로 세지 않는다', () => {
  const trace = new ScrollObservation();
  trace.begin('doc', 'zoom', 0);
  assert.throws(() => trace.finish('complete'), /milestone/);
  trace.finish('timeout', 'image-pending');
  assert.equal(trace.snapshot()[0].milestones.retainedComplete, undefined);
  assert.equal(trace.snapshot()[0].status, 'timeout');
});

test('기록·span·frame buffer는 bounded이며 snapshot은 독립 복사다', () => {
  const trace = new ScrollObservation(2, 2);
  for (let i = 0; i < 3; i++) {
    const id = trace.begin('doc', 'scroll', 0);
    for (let n = 0; n < 6; n++) { trace.count(id, 'raster', n, n + 1, 3); trace.frame(id, n); }
  }
  const snapshot = trace.snapshot();
  assert.equal(snapshot.length, 2);
  assert.equal(snapshot[1].spans.length, 2);
  assert.equal(snapshot[1].spansDropped, 4);
  assert.equal(snapshot[1].frames.length, 2);
  assert.equal(snapshot[1].framesDropped, 3);
  assert.equal(snapshot[1].counters.raster.units, 18);
  snapshot[1].counters.raster.units = -1;
  assert.equal(trace.snapshot()[1].counters.raster.units, 18);
  trace.clear();
  assert.deepEqual(trace.snapshot(), []);
  assert.throws(() => new ScrollObservation(0));
});

test('surface 비용은 실제 다층·익명 pool dimension 합이며 명목 DPR을 곱하지 않는다', () => {
  assert.equal(surfacePixels([{ width: 540, height: 764 }, { width: 540, height: 764 }, { width: 0, height: 0 }]), 825120);
});

test('scroll setter 뒤 이전 visibility는 완료가 아니며 scope·zoom·양축 ack가 모두 필요하다', () => {
  const old = { scope: 'doc1/1', zoom: 1, x: 0, y: 0 };
  const target = { ...old, y: 300 };
  assert.equal(viewportApplied(target, old), false);
  assert.equal(viewportApplied(target, null), false);
  assert.equal(viewportApplied(target, { ...target }), true);
  for (const changed of [{ scope: 'doc2/1' }, { zoom: .5 }, { x: 20 }, { y: 301 }]) {
    assert.equal(viewportApplied(target, { ...target, ...changed }), false);
  }
});

test('cold DOM image·decode 실패를 완성으로 보지 않고 오래된 문서/request를 기각한다', () => {
  assert.equal(flowImageState([]), 'ready');
  assert.equal(flowImageState([{ complete: false, naturalWidth: 0 }]), 'pending');
  assert.equal(flowImageState([{ complete: true, naturalWidth: 0 }]), 'failed');
  assert.equal(flowImageState([{ complete: true, naturalWidth: 300 }]), 'ready');
  assert.equal(currentImageRequest('d1', 'd1', 2, 2), true);
  assert.equal(currentImageRequest('d2', 'd1', 2, 2), false);
  assert.equal(currentImageRequest('d1', 'd1', 3, 2), false);
  assert.equal(currentImageRequest('d1', 'd1', undefined, 2), false);
});

test('fallback으로 job이 없어져도 관찰된 pending/실패를 준비 완료로 승격하지 않는다', () => {
  assert.equal(imageCompletion('pending'), 'pending');
  assert.equal(imageCompletion('failed'), 'failed');
  assert.equal(imageCompletion(undefined), 'unknown');
  for (const kind of ['decoded', 'cached', 'none'] as const) assert.equal(imageCompletion(kind), 'ready');
});
