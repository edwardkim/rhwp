import test from 'node:test';
import assert from 'node:assert/strict';
import { ZoomSessionObservation } from '../src/dev/zoom-session-observation.ts';

const state = { zoom: 1, animating: false, columns: 1 };
const wheel = { eventTimestamp: 100, deltaX: 0, deltaY: 2, deltaMode: 0, ctrlKey: true, metaKey: false, trusted: true };

test('A/B frame의 미생성 bitmap과 최신 raster 상태를 별도로 보존한다', () => {
  const observation = new ZoomSessionObservation();
  observation.start('d', 0);
  observation.frame('d', 10, { ...state, scrollY: 500, visiblePages: [1, 2, 3],
    missingBitmapPages: [3], currentRasterPages: [1], visibleQueued: 2, prefetchQueued: 1, pendingImages: 0 });
  const snap = observation.snapshot()!;
  assert.deepEqual(snap.frames[0].missingBitmapPages, [3]);
  assert.deepEqual(snap.frames[0].currentRasterPages, [1]);
  snap.frames[0].visiblePages!.push(99);
  assert.deepEqual(observation.snapshot()!.frames[0].visiblePages, [1, 2, 3]);
});

test('animation 수렴과 입력 quiet/raster pending을 구별해 기록한다', () => {
  const observation = new ZoomSessionObservation();
  observation.start('d', 0);
  const waiting = { ...state, inputActive: true, rasterPending: true, zoomGeneration: 2 };
  observation.frame('d', 16, waiting);
  observation.span('d', 'geometry.zoom', 16, 17, null, waiting);
  observation.span('d', 'zoom.ready', 120, 130, null, { ...waiting, inputActive: false, rasterPending: false });
  observation.stop(140, true);
  const s = observation.snapshot()!;
  assert.equal(s.frames[0].animating, false);
  assert.equal(s.frames[0].rasterPending, true);
  assert.equal(s.spans[1].rasterPending, false);
  assert.equal(s.spans[1].zoomGeneration, 2);
  assert.equal(s.counters['zoom.ready'].calls, 1);
});

test('수동 구간은 입력 사이의 정착과 다음 입력을 한 시간축에 보존한다', () => {
  const observation = new ZoomSessionObservation();
  observation.start('doc1', 100);
  observation.input('doc1', 101, wheel, state);
  observation.span('doc1', 'geometry.zoom', 117, 118, null, { ...state, animating: true });
  observation.span('doc1', 'page.releaseAll', 149, 150, null, state);
  observation.span('doc1', 'raster.main', 150, 180, 0, state);
  observation.input('doc1', 200, { ...wheel, deltaY: -2 }, state);
  observation.frame('doc1', 116, state);
  observation.frame('doc1', 190, state);
  observation.stop(300, false);
  observation.stop(350, true);
  const s = observation.snapshot()!;
  assert.equal(s.status, 'stopped');
  assert.equal(s.knownWorkReadyAtStop, false);
  assert.equal(s.endedAt, 300);
  assert.deepEqual(s.inputs.map(i => i.at), [1, 100]);
  assert.deepEqual(s.spans.map(i => i.boundary), ['geometry.zoom', 'page.releaseAll', 'raster.main']);
  assert.equal(s.frames[1].gap, 74);
  assert.equal(s.counters['raster.main'].inclusiveMs, 30);
  assert.equal(s.spans[2].page, 0);
});

test('상한을 넘겨도 호출 합계와 누락 수를 남기며 상세 버퍼는 유한하다', () => {
  const observation = new ZoomSessionObservation(2);
  observation.start('d', 0);
  for (let i = 0; i < 5; i++) {
    observation.input('d', i, wheel, state);
    observation.frame('d', i, state);
    observation.span('d', 'geometry.zoom', i, i + 1, null, state);
    observation.longTask('d', i, 60);
  }
  const s = observation.snapshot()!;
  assert.equal(s.inputs.length, 2);
  assert.equal(s.spans.length, 2);
  assert.equal(s.frames.length, 2);
  assert.equal(s.longTasks.length, 2);
  assert.deepEqual(s.dropped, { inputs: 3, spans: 3, frames: 3, longTasks: 3 });
  assert.equal(s.counters['geometry.zoom'].calls, 5);
});

test('scope 전환·시간 초과·관찰 off는 중단이며 완료로 승격하지 않는다', () => {
  const observation = new ZoomSessionObservation(2, 100);
  observation.start('d1', 0);
  observation.frame('d2', 10, state);
  assert.equal(observation.snapshot()!.status, 'interrupted');
  observation.stop(20, true);
  assert.equal(observation.snapshot()!.knownWorkReadyAtStop, null);
  observation.start('d2', 0);
  observation.frame('d2', 100, state);
  assert.equal(observation.snapshot()!.status, 'timeout');
  observation.start('d2', 200);
  observation.interrupt(210, 'observation off');
  observation.input('d2', 220, wheel, state);
  assert.equal(observation.snapshot()!.inputs.length, 0);
});

test('off·종료·이전 시각은 무작업이고 snapshot과 다음 구간은 독립이다', () => {
  const observation = new ZoomSessionObservation();
  observation.input('d', 0, wheel, state);
  assert.equal(observation.snapshot(), null);
  observation.start('d', 100);
  observation.longTask('old-document', 90, 50);
  assert.equal(observation.recording, true, '이전 구간의 늦은 entry가 새 구간을 중단하지 않는다');
  observation.input('d', 99, wheel, state);
  observation.input('d', 101, { ...wheel, trusted: false }, state);
  const s = observation.snapshot()!;
  s.inputs[0].deltaY = 200;
  assert.equal(observation.snapshot()!.inputs[0].deltaY, 2);
  assert.equal(observation.snapshot()!.inputs[0].trusted, false);
  observation.stop(110, true);
  observation.span('d', 'late', 120, 150, 0, state);
  assert.deepEqual(observation.snapshot()!.counters, {});
  observation.start('d', 200);
  assert.equal(observation.snapshot()!.inputs.length, 0);
  observation.clear();
  assert.equal(observation.snapshot(), null);
  assert.throws(() => new ZoomSessionObservation(0));
  assert.throws(() => new ZoomSessionObservation(1, 0));
});
