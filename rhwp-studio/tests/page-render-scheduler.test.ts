import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import {
  PageRenderScheduler,
  syncVisibleRenderBudget,
  type SchedulerHost,
} from '../src/view/page-render-scheduler.ts';

test('단일·두 쪽 첫 페인트는 보이는 쪽을 한 프레임에서 모두 그린다', () => {
  assert.equal(syncVisibleRenderBudget(1, 1), 1);
  assert.equal(syncVisibleRenderBudget(2, 2), 2);
  assert.equal(syncVisibleRenderBudget(4, 4), 1, '한 행 여러 쪽은 slice');
  assert.equal(syncVisibleRenderBudget(4, 0), 0);
});

test('새 스크롤 generation은 남은 visible slice를 버리고 render를 호출하지 않는다', () => {
  const frames: Array<(time: number) => void> = [];
  const host: SchedulerHost = {
    requestAnimationFrame(callback) {
      frames.push(callback);
      return frames.length;
    },
    cancelAnimationFrame() {
      frames.length = 0;
    },
  };
  const scheduler = new PageRenderScheduler(host);
  const rendered: number[] = [];
  const gen1 = scheduler.beginFrame();
  scheduler.scheduleVisible([10, 11, 12], gen1, (pageIdx) => rendered.push(pageIdx));

  assert.equal(frames.length, 1);
  frames[0](0);
  assert.deepEqual(rendered, [10]);
  assert.equal(scheduler.pendingVisibleCount(), 2);

  const gen2 = scheduler.beginFrame();
  assert.equal(scheduler.pendingVisibleCount(), 0);
  assert.notEqual(gen2, gen1);

  scheduler.scheduleVisible([20], gen2, (pageIdx) => rendered.push(pageIdx));
  assert.equal(frames.length, 1);
  frames[0](0);
  assert.deepEqual(rendered, [10, 20]);
});

test('일반 휠은 Ctrl/Meta 없이 zoom을 바꾸지 않는다', () => {
  const source = readFileSync(new URL('../src/view/viewport-manager.ts', import.meta.url), 'utf8');
  const wheel = source.slice(source.indexOf('private onWheel'), source.indexOf('private wheelDeltaPixels'));
  assert.match(wheel, /if \(!e\.ctrlKey && !e\.metaKey\)/);
  const zoomCall = wheel.indexOf('smoothZoomTo');
  const modifierReturn = wheel.indexOf('return;');
  assert.ok(zoomCall > modifierReturn, 'Ctrl/Meta가 없는 경로는 zoom 호출 전에 return해야 한다');
});
