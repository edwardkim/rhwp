import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import type { PageInfo } from '../src/core/types.ts';
import { clampRenderScale } from '../src/view/render-backend.ts';
import {
  canvasCssSize,
  DEFAULT_PAGE_LAYER_COUNT,
  MAX_VISIBLE_CANVAS_PIXELS,
  overviewDprBucket,
  pageCanvasPixels,
  quantizeDprBucket,
  resolveAdaptiveRenderScale,
  type AdaptiveRenderScaleInput,
} from '../src/view/adaptive-render-scale.ts';

const A4 = { width: 793.7, height: 1122.5 };
const A4_LANDSCAPE = { width: 1122.5, height: 793.7 };

function baseInput(
  overrides: Partial<AdaptiveRenderScaleInput> = {},
): AdaptiveRenderScaleInput {
  return {
    pageWidth: A4.width,
    pageHeight: A4.height,
    zoom: 1,
    rawDpr: 2,
    pagesPerRow: 1,
    visiblePageCount: 1,
    retainedPageCount: 1,
    layerCount: DEFAULT_PAGE_LAYER_COUNT,
    interaction: 'idle',
    renderProfile: 'screen',
    ...overrides,
  };
}

test('overview 3쪽 이상은 raw DPR 2를 쓰지 않고 bucket 1을 쓴다', () => {
  const overview = resolveAdaptiveRenderScale(baseInput({
    pagesPerRow: 3,
    visiblePageCount: 6,
    retainedPageCount: 8,
    zoom: 0.25,
  }));
  assert.equal(overview.tier, 'overview');
  assert.equal(overview.bucket, 1);
  assert.equal(overview.effectiveDpr, 1);
  assert.equal(overviewDprBucket(2), 1);
});

test('DPR 2 overview는 동일 CSS 크기 full-DPR 대비 Canvas 픽셀을 50% 이상 줄인다', () => {
  const zoom = 0.36;
  const full = resolveAdaptiveRenderScale(baseInput({
    zoom,
    pagesPerRow: 1,
    rawDpr: 2,
  }));
  const overview = resolveAdaptiveRenderScale(baseInput({
    zoom,
    pagesPerRow: 4,
    visiblePageCount: 8,
    retainedPageCount: 10,
    rawDpr: 2,
  }));
  assert.equal(full.effectiveDpr, 2);
  assert.equal(overview.effectiveDpr, 1);
  assert.ok(
    overview.canvasPixels <= full.canvasPixels * 0.5,
    `overview ${overview.canvasPixels} vs full ${full.canvasPixels}`,
  );
  assert.equal(overview.cssWidth, full.cssWidth);
  assert.equal(overview.cssHeight, full.cssHeight);
});

test('tier가 바뀌어도 CSS 페이지 크기는 1 CSS px 이내로 같다', () => {
  const zoom = 0.5;
  const screen = resolveAdaptiveRenderScale(baseInput({ zoom, pagesPerRow: 2, rawDpr: 2 }));
  const overview = resolveAdaptiveRenderScale(baseInput({ zoom, pagesPerRow: 4, rawDpr: 2 }));
  const screenCss = canvasCssSize(A4.width, A4.height, zoom, screen.renderScale);
  const overviewCss = canvasCssSize(A4.width, A4.height, zoom, overview.renderScale);
  assert.ok(Math.abs(screenCss.width - overviewCss.width) <= 1);
  assert.ok(Math.abs(screenCss.height - overviewCss.height) <= 1);
  assert.ok(Math.abs(screenCss.width - A4.width * zoom) <= 1);
  assert.ok(Math.abs(overviewCss.height - A4.height * zoom) <= 1);
});

test('DPR bucket은 1/1.5/2이며 경계에서 히스테리시스를 둔다', () => {
  assert.equal(quantizeDprBucket(1.0), 1);
  assert.equal(quantizeDprBucket(1.2), 1);
  assert.equal(quantizeDprBucket(1.5), 1.5);
  assert.equal(quantizeDprBucket(2.0), 2);
  assert.equal(quantizeDprBucket(1.3, 1), 1);
  assert.equal(quantizeDprBucket(1.4, 1), 1.5);
  assert.equal(quantizeDprBucket(1.8, 1.5), 1.5);
  assert.equal(quantizeDprBucket(1.9, 1.5), 2);
  assert.equal(quantizeDprBucket(1.7, 2), 2);
  assert.equal(quantizeDprBucket(1.6, 2), 1.5);
});

test('편집·포커스 페이지는 overview에서도 screen tier로 승격된다', () => {
  const neighbor = resolveAdaptiveRenderScale(baseInput({
    pagesPerRow: 4,
    zoom: 0.25,
    rawDpr: 2,
    visiblePageCount: 8,
  }));
  const focused = resolveAdaptiveRenderScale(baseInput({
    pagesPerRow: 4,
    zoom: 0.25,
    rawDpr: 2,
    visiblePageCount: 8,
    isEditing: true,
  }));
  assert.equal(neighbor.tier, 'overview');
  assert.equal(neighbor.effectiveDpr, 1);
  assert.equal(focused.tier, 'screen');
  assert.equal(focused.effectiveDpr, 2);
});

test('단일 쪽·두 쪽 일반 편집은 DPR 2를 유지한다', () => {
  for (const pagesPerRow of [1, 2]) {
    const result = resolveAdaptiveRenderScale(baseInput({
      pagesPerRow,
      zoom: 1,
      rawDpr: 2,
      isEditing: true,
    }));
    assert.equal(result.tier, 'screen');
    assert.equal(result.effectiveDpr, 2);
    assert.equal(result.bucket, 2);
    assert.equal(result.renderScale, 2);
  }
});

test('print·highQuality는 화면용 적응형 정책을 쓰지 않는다', () => {
  for (const renderProfile of ['print', 'highQuality'] as const) {
    const result = resolveAdaptiveRenderScale(baseInput({
      pagesPerRow: 4,
      zoom: 0.25,
      rawDpr: 2,
      renderProfile,
      visiblePageCount: 12,
    }));
    assert.equal(result.tier, 'export');
    assert.equal(result.effectiveDpr, 2);
    assert.equal(result.renderScale, 0.5);
  }
});

test('핀치 중에는 이전 bucket을 preview로 유지한다', () => {
  const result = resolveAdaptiveRenderScale(baseInput({
    pagesPerRow: 2,
    rawDpr: 2,
    zoom: 1,
    interaction: 'pinch',
    previousBucket: 1,
  }));
  assert.equal(result.tier, 'preview');
  assert.equal(result.effectiveDpr, 1);
  assert.equal(result.bucket, 1);
});

test('overview 총 surface는 보이는 페이지·레이어 수를 반영한다', () => {
  const result = resolveAdaptiveRenderScale(baseInput({
    pagesPerRow: 4,
    zoom: 0.25,
    rawDpr: 2,
    visiblePageCount: 8,
    retainedPageCount: 12,
    layerCount: 4,
  }));
  const onePage = pageCanvasPixels(A4.width, A4.height, 0.25, 1);
  assert.equal(result.estimatedVisibleSurfacePixels, onePage * 8 * 4);
  assert.equal(result.estimatedRetainedSurfacePixels, onePage * 12 * 4);
  assert.ok(result.estimatedVisibleSurfacePixels <= MAX_VISIBLE_CANVAS_PIXELS);
});

test('A4/가로, DPR 1/2/3, zoom 25/36/50/100%, 1/2/4쪽 매트릭스', () => {
  const papers = [A4, A4_LANDSCAPE];
  const dprs = [1, 2, 3];
  const zooms = [0.25, 0.36, 0.5, 1];
  const columns = [1, 2, 4];
  for (const paper of papers) {
    for (const rawDpr of dprs) {
      for (const zoom of zooms) {
        for (const pagesPerRow of columns) {
          const result = resolveAdaptiveRenderScale(baseInput({
            pageWidth: paper.width,
            pageHeight: paper.height,
            rawDpr,
            zoom,
            pagesPerRow,
            visiblePageCount: Math.max(1, pagesPerRow * 2),
            retainedPageCount: Math.max(2, pagesPerRow * 3),
          }));
          const css = canvasCssSize(
            paper.width,
            paper.height,
            zoom,
            result.renderScale,
          );
          assert.ok(Math.abs(css.width - paper.width * zoom) <= 1);
          assert.ok(Math.abs(css.height - paper.height * zoom) <= 1);
          if (pagesPerRow >= 3) {
            assert.equal(result.tier, 'overview');
            if (rawDpr >= 2) {
              const fullPixels = pageCanvasPixels(paper.width, paper.height, zoom, rawDpr);
              assert.ok(
                result.canvasPixels <= fullPixels * 0.5 + paper.width + paper.height,
                `pixels ${result.canvasPixels} full ${fullPixels} dpr=${rawDpr} zoom=${zoom}`,
              );
            }
          } else {
            assert.equal(result.tier, 'screen');
            if (rawDpr === 2) assert.equal(result.effectiveDpr, 2);
            if (rawDpr === 1) assert.equal(result.effectiveDpr, 1);
            if (rawDpr === 3) assert.equal(result.effectiveDpr, 3);
          }
        }
      }
    }
  }
});

test('clampRenderScale는 적응형 결과 위에 기존 67M 상한을 유지한다', () => {
  const pageInfo = { width: A4.width, height: A4.height } as PageInfo;
  const overview = resolveAdaptiveRenderScale(baseInput({
    pagesPerRow: 4,
    zoom: 0.25,
    rawDpr: 2,
  }));
  assert.equal(clampRenderScale(pageInfo, overview.renderScale), overview.renderScale);
  const screen = resolveAdaptiveRenderScale(baseInput({
    pagesPerRow: 1,
    zoom: 1,
    rawDpr: 2,
  }));
  assert.equal(clampRenderScale(pageInfo, screen.renderScale), screen.renderScale);
});

test('CanvasView는 표시 zoom과 render scale을 분리하고 Canvas2D/CanvasKit가 같은 renderScale을 쓴다', () => {
  const canvasView = readFileSync(new URL('../src/view/canvas-view.ts', import.meta.url), 'utf8');
  const pageRenderer = readFileSync(new URL('../src/view/page-renderer.ts', import.meta.url), 'utf8');
  assert.match(canvasView, /resolveAdaptiveRenderScale\(/);
  assert.match(canvasView, /clampRenderScale\(pageInfo,\s*decision\.renderScale\)/);
  assert.equal(canvasView.includes('clampRenderScale(pageInfo, zoom * rawDpr)'), false);
  assert.match(canvasView, /renderPage\(pageIdx,\s*canvas,\s*renderScale,\s*zoom,\s*dpr,\s*renderContext\)/);
  assert.match(pageRenderer, /canvas\.width\s*=\s*Math\.max\(1,\s*Math\.ceil\(pageInfo\.width \* renderScale\)\)/);
  assert.match(pageRenderer, /this\.canvaskitRenderer\.renderPage\(tree,\s*canvas,\s*renderScale,\s*pageInfo\)/);
  assert.match(pageRenderer, /getRenderProfile\(\): LayerRenderProfile/);
});
