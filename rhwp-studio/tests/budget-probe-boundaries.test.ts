import test from 'node:test';
import assert from 'node:assert/strict';
import { budgetProbeBoundaries, budgetProbePage } from '../src/dev/budget-probe-boundaries.ts';
import { observeBoundary } from '../src/dev/scroll-observation.ts';

test('budget detail off creates no boundaries and never queries the product', () => {
  const target = new Proxy({}, { get() { assert.fail('unexpected product read'); } });
  assert.deepEqual(budgetProbeBoundaries(false, target, target, target, target), []);
  assert.equal(budgetProbeBoundaries(true, target, target, target, target).length, 8);
});

test('detail page labels distinguish layer/descriptor indices from budget arguments', () => {
  for (const name of ['budget.layerCount', 'budget.overlaySummary', 'budget.treeSummary',
    'budget.descriptor', 'wasm.overlayImages', 'wasm.layerTree']) {
    assert.equal(budgetProbePage(name, [0]), 0);
    assert.equal(budgetProbePage(name, [19]), 19);
    assert.equal(budgetProbePage(name, [false]), null);
  }
  assert.equal(budgetProbePage('budget.refresh', [true]), null);
  assert.equal(budgetProbePage('cache.reconcile', [40_000_000]), null);
});

test('nested metadata observation preserves cold/warm calls, return and restoration', () => {
  let clock = 0;
  let wasmCalls = 0;
  const wasm = { getPageOverlayImages(page: number) { wasmCalls++; clock += 80; return `page:${page}`; } };
  const renderer = {
    cache: new Map<number, number>(),
    getLayerPlaneSummaryFromOverlayImages(page: number) { return wasm.getPageOverlayImages(page); },
    getCanvasSurfaceLayerCount(page: number) {
      if (!this.cache.has(page)) {
        this.getLayerPlaneSummaryFromOverlayImages(page);
        this.cache.set(page, 2);
      }
      return this.cache.get(page);
    },
  };
  const original = renderer.getCanvasSurfaceLayerCount;
  const spans: { name: string; ms: number; page: number | null }[] = [];
  const restores = budgetProbeBoundaries(true, {}, renderer, wasm, {})
    .filter(([target, key]) => typeof (target as Record<string, unknown>)[key] === 'function')
    .map(([target, key, name]) => observeBoundary(target, key, () => clock, () => null,
      call => spans.push({ name, ms: call.endedAt - call.startedAt, page: budgetProbePage(name, call.args) }), assert.fail));
  assert.equal(wasmCalls, 0, 'installation does not warm a cache');
  assert.equal(renderer.getCanvasSurfaceLayerCount(7), 2);
  assert.equal(renderer.getCanvasSurfaceLayerCount(7), 2);
  assert.equal(wasmCalls, 1);
  assert.deepEqual(spans, [
    { name: 'wasm.overlayImages', ms: 80, page: 7 },
    { name: 'budget.overlaySummary', ms: 80, page: 7 },
    { name: 'budget.layerCount', ms: 80, page: 7 },
    { name: 'budget.layerCount', ms: 0, page: 7 },
  ]);
  for (const restore of restores.reverse()) restore();
  assert.equal(renderer.getCanvasSurfaceLayerCount, original);
  renderer.getCanvasSurfaceLayerCount(8);
  assert.equal(wasmCalls, 2);
  assert.equal(spans.length, 4);
});
