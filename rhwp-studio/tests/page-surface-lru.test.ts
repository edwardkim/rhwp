import test from 'node:test';
import assert from 'node:assert/strict';

import {
  estimateSurfaceBytes,
  pageSurfaceCacheKey,
  PageSurfaceLru,
  quantizeRenderScaleTier,
  TOTAL_SURFACE_PIXEL_BUDGET,
} from '../src/view/page-surface-lru.ts';

test('같은 page/revision/backend/tier 왕복은 render 없이 cache hit가 난다', () => {
  const lru = new PageSurfaceLru(10_000);
  const evicted: number[] = [];
  const onEvict = (pageIdx: number) => evicted.push(pageIdx);
  const key = (pageIdx: number, revision = 1, scale = 2) => pageSurfaceCacheKey({
    pageIdx,
    revision,
    backend: 'canvas2d',
    renderScaleTier: quantizeRenderScaleTier(scale),
  });

  lru.put(4, key(4), 1000, onEvict);
  lru.put(5, key(5), 1000, onEvict);
  assert.equal(lru.touch(4, key(4)), true);
  assert.equal(lru.touch(5, key(5)), true);
  assert.equal(lru.touch(4, key(4, 2)), false, 'revision이 바뀌면 miss');
  assert.equal(lru.touch(4, key(4, 1, 3)), false, 'renderScale tier가 바뀌면 miss');
  assert.deepEqual(evicted, []);
  assert.equal(lru.stats().hits, 2);
  assert.equal(lru.stats().misses, 2);
});

test('eviction은 고정 행 수가 아니라 surface 픽셀 예산을 따른다', () => {
  const lru = new PageSurfaceLru(3_000);
  const evicted: number[] = [];
  const keyOf = (pageIdx: number) => pageSurfaceCacheKey({
    pageIdx,
    revision: 1,
    backend: 'canvaskit',
    renderScaleTier: 1,
  });

  lru.put(0, keyOf(0), 1000, (pageIdx) => evicted.push(pageIdx), new Set([2]));
  lru.put(1, keyOf(1), 1000, (pageIdx) => evicted.push(pageIdx), new Set([2]));
  lru.put(2, keyOf(2), 1000, (pageIdx) => evicted.push(pageIdx), new Set([2]));
  lru.put(3, keyOf(3), 1000, (pageIdx) => evicted.push(pageIdx), new Set([2]));

  assert.ok(evicted.includes(0) || evicted.includes(1) || evicted.includes(3));
  assert.equal(evicted.includes(2), false, 'visible 페이지는 예산이 넘어도 유지');
  assert.ok(lru.stats().pixels <= 3_000 || lru.has(2, keyOf(2)));
  assert.equal(estimateSurfaceBytes(1000), 4000);
  assert.equal(TOTAL_SURFACE_PIXEL_BUDGET, 67_108_864);
});

test('문서 변경 키와 dispose clear는 이전 표면을 버린다', () => {
  const lru = new PageSurfaceLru(8_000);
  const key1 = pageSurfaceCacheKey({
    pageIdx: 0, revision: 1, backend: 'canvas2d', renderScaleTier: 1,
  });
  const key2 = pageSurfaceCacheKey({
    pageIdx: 0, revision: 2, backend: 'canvas2d', renderScaleTier: 1,
  });
  lru.put(0, key1, 1000, () => {});
  assert.equal(lru.has(0, key1), true);
  lru.put(0, key2, 1000, () => {});
  assert.equal(lru.has(0, key1), false);
  assert.equal(lru.has(0, key2), true);
  lru.clear();
  assert.equal(lru.has(0, key2), false);
  assert.equal(lru.stats().size, 0);
});
