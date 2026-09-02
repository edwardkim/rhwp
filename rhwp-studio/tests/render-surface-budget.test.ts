import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import {
  planRenderSurfaceBudget,
  renderDprSteps,
  type RenderSurfacePageInput,
} from '../src/view/render-surface-budget.ts';

const A4 = { width: 793.7, height: 1122.5 };

function pages(
  count: number,
  options: {
    visible?: readonly number[];
    focused?: number | null;
    width?: number;
    height?: number;
    layerCount?: number;
  } = {},
): RenderSurfacePageInput[] {
  const visible = new Set(options.visible ?? Array.from({ length: count }, (_, index) => index));
  const focused = options.focused ?? null;
  return Array.from({ length: count }, (_, pageIndex) => ({
    pageIndex,
    width: options.width ?? A4.width,
    height: options.height ?? A4.height,
    layerCount: options.layerCount,
    visible: visible.has(pageIndex),
    focused: focused === pageIndex,
    distanceFromFocus: Math.abs(pageIndex - (focused ?? 0)),
  }));
}

test('4-layer 4쪽의 34% 배치는 예산 이내이면 모두 raw DPR을 유지한다', () => {
  const plan = planRenderSurfaceBudget({
    pages: pages(4, { focused: 2 }),
    zoom: 0.34,
    rawDpr: 2,
    layerCount: 4,
  });

  assert.equal(plan.withinBudget, true);
  assert.deepEqual(plan.decisions.map(decision => decision.effectiveDpr), [2, 2, 2, 2]);
  assert.ok(plan.fullQualityRetainedSurfacePixels < plan.retainedPixelBudget);
});

test('단일 Canvas 페이지는 보수적 fallback 4가 있어도 실제 layerCount 1로 계산한다', () => {
  const plan = planRenderSurfaceBudget({
    pages: pages(3, { visible: [0, 1], focused: 0, layerCount: 1 }),
    zoom: 1,
    rawDpr: 2,
    layerCount: 4,
  });

  assert.equal(plan.withinBudget, true);
  assert.deepEqual(plan.decisions.map(decision => decision.layerCount), [1, 1, 1]);
  assert.deepEqual(plan.decisions.map(decision => decision.effectiveDpr), [2, 2, 2]);
  assert.ok(plan.fullQualityRetainedSurfacePixels < plan.retainedPixelBudget);
});

test('열 수가 아니라 실제 surface 비용으로만 조정 여부를 결정한다', () => {
  const manySmallVisiblePages = planRenderSurfaceBudget({
    pages: pages(8, { width: 100, height: 100, focused: 0 }),
    zoom: 0.25,
    rawDpr: 2,
    layerCount: 4,
  });
  assert.ok(manySmallVisiblePages.decisions.every(decision => decision.tier === 'screen'));

  const twoLargeVisiblePages = planRenderSurfaceBudget({
    pages: pages(2, { width: 3000, height: 3000, focused: 0 }),
    zoom: 1,
    rawDpr: 2,
    layerCount: 4,
  });
  assert.equal(twoLargeVisiblePages.decisions[0]?.effectiveDpr, 2);
  assert.ok((twoLargeVisiblePages.decisions[1]?.effectiveDpr ?? 2) < 2);
});

test('예산 초과분은 DPR 1로 직행하지 않고 모든 후보를 1.5로 먼저 낮춘다', () => {
  const plan = planRenderSurfaceBudget({
    pages: pages(4, { focused: 0, width: 1000, height: 1000 }),
    zoom: 1,
    rawDpr: 2,
    layerCount: 1,
    visiblePixelBudget: 11_000_000,
    retainedPixelBudget: 11_000_000,
  });

  assert.equal(plan.withinBudget, true);
  assert.equal(plan.decisions.filter(decision => decision.tier === 'screen').length, 1);
  assert.equal(plan.decisions.filter(decision => decision.tier === 'balanced').length, 3);
  assert.equal(plan.decisions.filter(decision => decision.tier === 'economy').length, 0);
});

test('retained 예산은 visible 품질보다 화면 밖 prefetch를 먼저 낮춘다', () => {
  const plan = planRenderSurfaceBudget({
    pages: pages(4, {
      visible: [0, 1],
      focused: 0,
      width: 1000,
      height: 1000,
    }),
    zoom: 1,
    rawDpr: 2,
    layerCount: 1,
    visiblePixelBudget: 8_000_000,
    retainedPixelBudget: 12_000_000,
  });

  assert.deepEqual(
    plan.decisions.filter(decision => decision.visible).map(decision => decision.effectiveDpr),
    [2, 2],
  );
  assert.ok(
    plan.decisions.filter(decision => !decision.visible).every(decision => decision.effectiveDpr < 2),
  );
});

test('기본 예산은 100% 두 쪽을 보존하고 초과한 prefetch만 낮춘다', () => {
  const plan = planRenderSurfaceBudget({
    pages: pages(3, { visible: [0, 1], focused: 0 }),
    zoom: 1,
    rawDpr: 2,
    layerCount: 4,
  });

  assert.equal(plan.withinBudget, true);
  assert.ok(
    plan.decisions.filter(decision => decision.visible)
      .every(decision => decision.effectiveDpr === 2),
  );
  assert.equal(
    plan.decisions.filter(decision => !decision.visible && decision.effectiveDpr === 1.5).length,
    1,
  );
  assert.equal(
    plan.decisions.filter(decision => !decision.visible && decision.effectiveDpr === 1).length,
    0,
  );
});

test('같은 단계에서는 편집 쪽에서 가장 먼 페이지를 먼저 조정한다', () => {
  const plan = planRenderSurfaceBudget({
    pages: pages(4, { focused: 0, width: 1000, height: 1000 }),
    zoom: 1,
    rawDpr: 2,
    layerCount: 1,
    visiblePixelBudget: 14_500_000,
    retainedPixelBudget: 14_500_000,
  });

  assert.equal(plan.decisions.find(decision => decision.pageIndex === 3)?.effectiveDpr, 1.5);
  assert.ok(
    plan.decisions.filter(decision => decision.pageIndex !== 3)
      .every(decision => decision.effectiveDpr === 2),
  );
});

test('편집 페이지는 예산을 충족할 수 없는 경우에도 raw DPR을 유지한다', () => {
  const plan = planRenderSurfaceBudget({
    pages: pages(1, { focused: 0, width: 3000, height: 3000 }),
    zoom: 1,
    rawDpr: 2,
    layerCount: 4,
    visiblePixelBudget: 1,
    retainedPixelBudget: 1,
  });

  assert.equal(plan.withinBudget, false);
  assert.equal(plan.decisions[0]?.effectiveDpr, 2);
  assert.equal(plan.decisions[0]?.tier, 'screen');
});

test('포커스가 이동하면 새 편집 쪽은 즉시 raw DPR로 승격된다', () => {
  const common = {
    zoom: 1,
    rawDpr: 2,
    layerCount: 1,
    visiblePixelBudget: 11_000_000,
    retainedPixelBudget: 11_000_000,
  };
  const before = planRenderSurfaceBudget({
    ...common,
    pages: pages(4, { focused: 0, width: 1000, height: 1000 }),
  });
  const previousEffectiveDpr = new Map(
    before.decisions.map(decision => [decision.pageIndex, decision.effectiveDpr]),
  );
  const after = planRenderSurfaceBudget({
    ...common,
    pages: pages(4, { focused: 3, width: 1000, height: 1000 }),
    previousEffectiveDpr,
  });

  assert.equal(after.decisions.find(decision => decision.pageIndex === 3)?.effectiveDpr, 2);
  assert.equal(after.decisions.find(decision => decision.pageIndex === 3)?.tier, 'screen');
  assert.ok((after.decisions.find(decision => decision.pageIndex === 0)?.effectiveDpr ?? 2) < 2);
});

test('예산 히스테리시스는 경계에서 이전 demotion을 유지하고 여유가 생기면 복원한다', () => {
  const common = {
    pages: pages(4, { width: 100, height: 100 }),
    rawDpr: 2,
    layerCount: 1,
    visiblePixelBudget: 120_000,
    retainedPixelBudget: 120_000,
  };
  const over = planRenderSurfaceBudget({ ...common, zoom: 1 });
  const previous = new Map(
    over.decisions.map(decision => [decision.pageIndex, decision.effectiveDpr]),
  );
  const nearBoundary = planRenderSurfaceBudget({
    ...common,
    zoom: Math.sqrt(114_000 / 160_000),
    previousEffectiveDpr: previous,
  });
  const belowRelease = planRenderSurfaceBudget({
    ...common,
    zoom: Math.sqrt(100_000 / 160_000),
    previousEffectiveDpr: previous,
  });

  assert.equal(nearBoundary.heldByHysteresis, true);
  assert.ok(nearBoundary.decisions.some(decision => decision.effectiveDpr < 2));
  assert.equal(belowRelease.heldByHysteresis, false);
  assert.ok(belowRelease.decisions.every(decision => decision.effectiveDpr === 2));
});

test('DPR 3은 3을 보존하고 필요할 때 2, 1.5, 1 순서로 낮춘다', () => {
  assert.deepEqual(renderDprSteps(3), [3, 2, 1.5, 1]);
  assert.deepEqual(renderDprSteps(2), [2, 1.5, 1]);
  assert.deepEqual(renderDprSteps(1), [1]);
});

test('print와 highQuality는 surface 예산으로 낮추지 않는다', () => {
  for (const renderProfile of ['print', 'highQuality'] as const) {
    const plan = planRenderSurfaceBudget({
      pages: pages(4, { width: 3000, height: 3000 }),
      zoom: 1,
      rawDpr: 3,
      layerCount: 4,
      visiblePixelBudget: 1,
      retainedPixelBudget: 1,
      renderProfile,
    });
    assert.ok(plan.decisions.every(decision => decision.effectiveDpr === 3));
    assert.ok(plan.decisions.every(decision => decision.tier === 'export'));
  }
});

test('CanvasView는 가시 집합과 포커스 변경 후 budget plan을 갱신한다', () => {
  const source = readFileSync(new URL('../src/view/canvas-view.ts', import.meta.url), 'utf8');
  const visibleAssignment = source.indexOf('this.currentVisiblePages = visiblePages;');
  const planRefresh = source.indexOf('this.refreshRenderSurfacePlan(!isScroll);', visibleAssignment);
  const visibleWorkCreation = source.indexOf('const visibleWork = isScroll', planRefresh);

  assert.ok(visibleAssignment >= 0 && visibleAssignment < planRefresh);
  assert.ok(planRefresh < visibleWorkCreation);
  assert.match(
    source,
    /setEditingPageIndex[\s\S]*?focused-page-changed[\s\S]*?refreshRenderSurfacePlan\(true\)/,
  );
  assert.match(
    source,
    /environmentKey !== this\.renderSurfaceEnvironmentKey[\s\S]*?previousEffectiveDpr\.clear\(\)/,
  );
  assert.match(source, /getCanvasSurfaceLayerCount\(pageIndex\)/);
  assert.doesNotMatch(
    readFileSync(new URL('../src/view/render-surface-budget.ts', import.meta.url), 'utf8'),
    /pagesPerRow|OVERVIEW_PAGES_PER_ROW/,
  );
});
