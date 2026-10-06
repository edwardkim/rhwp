import test from 'node:test';
import assert from 'node:assert/strict';
import { probeZoomInputState, probeZoomRasterPending } from '../src/dev/probe-zoom-compat.ts';

test('구 기준선은 pending API를 제품에 추가하지 않고 관찰 불가 필드를 생략한다', () => {
  const old = Object.freeze({ isZoomAnimating: () => false });
  assert.deepEqual(probeZoomInputState(old), {});
  assert.equal(probeZoomRasterPending(old), false);
  assert.deepEqual(Object.keys(old), ['isZoomAnimating']);
});

test('현재 후보의 input/pending 상태와 this를 보존한다', () => {
  const current = {
    pending: true,
    getZoomInputState() { return { rasterPending: this.pending, zoomGeneration: 4 }; },
    isZoomRasterPending() { return this.pending; },
  };
  assert.deepEqual(probeZoomInputState(current), { rasterPending: true, zoomGeneration: 4 });
  assert.equal(probeZoomRasterPending(current), true);
  current.pending = false;
  assert.equal(probeZoomRasterPending(current), false);
});
