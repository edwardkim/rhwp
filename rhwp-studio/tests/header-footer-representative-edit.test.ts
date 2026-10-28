import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

import type { PageInfo } from '../src/core/types.ts';
import {
  headerFooterClipPath,
  resolveHeaderFooterBadgeMetrics,
  resolveHeaderFooterBandBox,
} from '../src/view/header-footer-edit-overlay.ts';
import {
  headerFooterApplyToLabel,
  parseHeaderFooterModeChanged,
} from '../src/engine/header-footer-mode.ts';

const page: PageInfo = {
  pageIndex: 0,
  width: 800,
  height: 1100,
  sectionIndex: 0,
  marginLeft: 80,
  marginRight: 80,
  marginTop: 60,
  marginBottom: 70,
  marginHeader: 40,
  marginFooter: 30,
  bodyLeft: 90,
  bodyRight: 710,
};

test('HF 대표 편집 영역은 WASM이 내보낸 PageAreas 결과를 우선한다', () => {
  const exactPage: PageInfo = {
    ...page,
    headerArea: { x: 91, y: 61, width: 618, height: 39 },
    footerArea: { x: 92, y: 1002, width: 616, height: 68 },
  };

  assert.deepEqual(resolveHeaderFooterBandBox(exactPage, true), exactPage.headerArea);
  assert.deepEqual(resolveHeaderFooterBandBox(exactPage, false), exactPage.footerArea);
});

test('구 WASM은 PageDef 여백으로 같은 HF 영역을 재구성한다', () => {
  assert.deepEqual(resolveHeaderFooterBandBox(page, true), {
    x: 90,
    y: 60,
    width: 620,
    height: 40,
  });
  assert.deepEqual(resolveHeaderFooterBandBox(page, false), {
    x: 90,
    y: 1000,
    width: 620,
    height: 70,
  });
  assert.equal(
    headerFooterClipPath(page, resolveHeaderFooterBandBox(page, true), 0.5),
    'inset(30px 45px 500px 45px)',
  );
});

test('HF 안내 라벨은 고배율에서 완만하게 커지고 최대 두 배로 제한된다', () => {
  assert.deepEqual(resolveHeaderFooterBadgeMetrics(0.5), {
    fontSizePx: 10,
    gapPx: 4,
  });
  assert.deepEqual(resolveHeaderFooterBadgeMetrics(1), {
    fontSizePx: 10,
    gapPx: 4,
  });
  assert.deepEqual(resolveHeaderFooterBadgeMetrics(5), {
    fontSizePx: 20,
    gapPx: 8,
  });
  const atTwoHundred = resolveHeaderFooterBadgeMetrics(2);
  assert.ok(atTwoHundred.fontSizePx > 14 && atTwoHundred.fontSizePx < 15);
  assert.ok(atTwoHundred.gapPx > 5 && atTwoHundred.gapPx < 6);
});

test('HF 편집 상태는 종류·타겟·대표 페이지를 함께 전달한다', () => {
  const state = parseHeaderFooterModeChanged({
    mode: 'footer',
    sectionIdx: 2,
    applyTo: 1,
    previewPage: 7,
  });
  assert.deepEqual(state, {
    mode: 'footer',
    sectionIdx: 2,
    applyTo: 1,
    previewPage: 7,
  });
  assert.equal(headerFooterApplyToLabel(0), '양쪽');
  assert.equal(headerFooterApplyToLabel(1), '짝수 쪽');
  assert.equal(headerFooterApplyToLabel(2), '홀수 쪽');
});

test('CanvasView는 대표 preview와 실제 적용 쪽 overlay를 비인쇄 계층으로 관리한다', () => {
  const source = readFileSync(new URL('../src/view/canvas-view.ts', import.meta.url), 'utf8');
  assert.match(source, /renderHeaderFooterEditPreviewToCanvas\(/);
  assert.match(source, /data-rhwp-hf-edit-page/);
  assert.match(source, /getHeaderFooterEditTarget\(pageIdx/);
  assert.match(source, /setPageMarginGuideEdges\(state\.mode === 'header' \? 'bottom' : 'top'\)/);
  assert.match(
    source,
    /drawPageMarginGuideCorners\(band,\s*guideCanvas,\s*renderScale,\s*'both',\s*undefined,\s*zoom\)/,
  );
  assert.match(source, /hf-edit-guide-canvas/);
  assert.match(source, /removeHeaderFooterEditOverlays\(\)/);
});

test('HF 편집 안내는 내용을 덮지 않고 모서리와 텍스트만 표시한다', () => {
  const css = readFileSync(new URL('../src/styles/editor.css', import.meta.url), 'utf8');
  const representative = css.match(/\.hf-edit-region\.is-representative\s*\{([^}]*)\}/)?.[1] ?? '';
  const related = css.match(/\.hf-edit-region\.is-related\s*\{([^}]*)\}/)?.[1] ?? '';
  const guideCanvas = css.match(/canvas\.hf-edit-guide-canvas\s*\{([^}]*)\}/)?.[1] ?? '';
  const badge = css.match(/\.hf-edit-badge\s*\{([^}]*)\}/)?.[1] ?? '';

  assert.doesNotMatch(representative, /background\s*:/);
  assert.doesNotMatch(representative, /border\s*:/);
  assert.doesNotMatch(related, /background\s*:/);
  assert.match(guideCanvas, /background:\s*transparent/);
  assert.match(guideCanvas, /transform:\s*none/);
  assert.doesNotMatch(css, /\.hf-edit-corner/);
  assert.match(badge, /--hf-edit-badge-gap/);
  assert.match(badge, /calc\(-100% - var\(--hf-edit-badge-gap, 4px\)\)/);
  assert.match(css, /\.hf-edit-badge[\s\S]*color:\s*#333333/);
});
