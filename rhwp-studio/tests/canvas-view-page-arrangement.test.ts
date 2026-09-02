import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { createServer } from 'vite';

import { VirtualScroll } from '../src/view/virtual-scroll.ts';

const canvasViewSource = readFileSync(
  fileURLToPath(new URL('../src/view/canvas-view.ts', import.meta.url)),
  'utf8',
);

function pages(n: number, width = 800, height = 1000) {
  return Array.from({ length: n }, () => ({ width, height })) as never;
}

function classMethodSource(name: string, nextName: string): string {
  const start = canvasViewSource.search(new RegExp(`\\n  (?:private )?${name}\\(`));
  const remaining = start >= 0 ? canvasViewSource.slice(start + 1) : '';
  const relativeEnd = remaining.search(new RegExp(`\\n  (?:private )?${nextName}\\(`));
  const end = relativeEnd >= 0 ? start + 1 + relativeEnd : -1;
  assert.ok(start >= 0, `${name} 메서드가 있어야 한다`);
  assert.ok(end > start, `${nextName} 경계가 ${name} 뒤에 있어야 한다`);
  return canvasViewSource.slice(start, end);
}

test('같은 행 그룹은 같은 토폴로지 키를 가지며 맞쪽은 두 쪽과 구분된다', () => {
  const scroll = new VirtualScroll(10);

  scroll.setPageDimensions(pages(6), 0.6, 1200, { kind: 'double' });
  const double = scroll.getLayoutTopologyKey();

  scroll.setPageDimensions(pages(6), 0.6, 1200, { kind: 'multiple', columns: 2, rows: 3 });
  assert.equal(scroll.getLayoutTopologyKey(), double, '동일 2열 행 그룹은 Canvas를 재사용할 수 있다');

  scroll.setPageDimensions(pages(6), 0.6, 1200, { kind: 'facing' });
  assert.notEqual(scroll.getLayoutTopologyKey(), double, '맞쪽 첫 빈 슬롯은 다른 행 토폴로지다');
});

test('자동 단일 열과 명시 한 쪽은 같은 토폴로지다', () => {
  const scroll = new VirtualScroll(10);
  scroll.setPageDimensions(pages(4), 0.75, 1200, { kind: 'auto' });
  const automatic = scroll.getLayoutTopologyKey();
  scroll.setPageDimensions(pages(4), 0.75, 1200, { kind: 'single' });
  assert.equal(scroll.getLayoutTopologyKey(), automatic);
});

test('CanvasView는 저장된 쪽 배치를 레이아웃 계산에 전달한다', () => {
  assert.match(
    canvasViewSource,
    /resolvePageViewSettings\([\s\S]*?viewSettings\.pageArrangement,[\s\S]*?viewSettings\.pageMovement/,
  );
  assert.match(
    canvasViewSource,
    /setPageDimensions\([\s\S]*?this\.pageArrangement,[\s\S]*?this\.pageMovement\.direction,[\s\S]*?viewport\.height/,
  );
});

test('CanvasView는 통합 보기 설정 이벤트만 구독하고 단일 필드 wrapper를 노출하지 않는다', () => {
  assert.match(
    canvasViewSource,
    /eventBus\.on\('page-view-settings-changed',[\s\S]*?this\.setPageViewSettings/,
  );
  assert.doesNotMatch(canvasViewSource, /page-arrangement-changed/);
  assert.doesNotMatch(canvasViewSource, /\n  setPageArrangement\(/);
  assert.doesNotMatch(canvasViewSource, /\n  setPageMovement\(/);
  assert.doesNotMatch(canvasViewSource, /\n  getPageArrangement\(/);
  assert.doesNotMatch(canvasViewSource, /\n  getPageMovement\(/);
  assert.match(canvasViewSource, /private setPageViewSettings\(/);
  const method = classMethodSource('setPageViewSettings', 'getViewportManager');
  assert.doesNotMatch(method, /document-(?:changed|mutated)/);
});

test('배치 전환은 중심 앵커를 복원하고 토폴로지가 달라질 때만 Canvas를 해제한다', () => {
  const method = classMethodSource('setPageViewSettings', 'getViewportManager');
  assert.match(method, /calculateAnchoredScroll\(/);
  assert.match(method, /CENTER_ZOOM_ANCHOR/);
  assert.match(method, /previousTopology\s*!==\s*nextTopology/);
  assert.match(method, /releaseAllRenderedPages\(\)/);
});

test('배치와 배율 transaction은 표준 zoom 이벤트를 유지하고 최종 레이아웃만 한 번 계산한다', () => {
  assert.match(
    canvasViewSource,
    /eventBus\.on\('zoom-changed',[\s\S]*?if \(this\.applyingPageViewSettingsTransaction\) return;/,
  );
  const method = classMethodSource('setPageViewSettings', 'getViewportManager');
  assert.match(method, /resolvePageViewSettingsChange\(changeValue\)/);
  assert.match(method, /this\.viewportManager\.setZoom\(/);
  assert.match(
    method,
    /try \{[\s\S]*?setZoom\([\s\S]*?finally \{[\s\S]*?applyingPageViewSettingsTransaction = false/,
  );
  assert.equal(
    method.match(/this\.recalcLayout\(\)/g)?.length,
    1,
    'transaction method에는 최종 recalcLayout 호출이 하나만 있어야 한다',
  );
  assert.match(method, /zoomChanged \|\| previousTopology !== nextTopology/);
  assert.match(
    method,
    /eventBus\.emit\('zoom-level-display', this\.viewportManager\.getZoom\(\)\)/,
  );
});

test('가로 쪽 이동은 배치와 함께 한 번에 전환하고 단일 가시성 snapshot을 사용한다', () => {
  assert.match(
    canvasViewSource,
    /eventBus\.on\('page-view-settings-changed',[\s\S]*?this\.setPageViewSettings/,
  );
  assert.match(
    canvasViewSource,
    /getVisibilitySnapshot\([\s\S]*?scrollX,[\s\S]*?vpWidth/,
  );
  const visibilityMethod = classMethodSource('updateVisiblePages', 'renderHeaderFooterEditOverlays');
  assert.equal(
    visibilityMethod.match(/getVisibilitySnapshot\(/g)?.length,
    1,
    'visible과 prefetch는 같은 snapshot을 한 번만 소비해야 한다',
  );
  assert.doesNotMatch(visibilityMethod, /get(?:Visible|Prefetch)Pages\(/);
  const method = classMethodSource('setPageViewSettings', 'getViewportManager');
  assert.match(method, /resolvePageViewSettings/);
  assert.doesNotMatch(method, /document-(?:changed|mutated)/);
});

test('실제 CanvasView zoom 경로는 자동 열 commit과 Canvas pool 단일 소유권을 유지한다', async () => {
  const studioRoot = fileURLToPath(new URL('..', import.meta.url));
  const vite = await createServer({
    root: studioRoot,
    appType: 'custom',
    logLevel: 'silent',
    server: { middlewareMode: true },
  });
  const previousDocument = Object.getOwnPropertyDescriptor(globalThis, 'document');

  try {
    const { CanvasView } = await vite.ssrLoadModule('/src/view/canvas-view.ts');
    const { CanvasPool } = await vite.ssrLoadModule('/src/view/canvas-pool.ts');
    const children: HTMLCanvasElement[] = [];
    let removeCount = 0;
    const scrollContent = {
      style: {},
      classList: { toggle: () => undefined },
      appendChild: (canvas: HTMLCanvasElement) => {
        children.push(canvas);
        (canvas as unknown as { parentElement: HTMLElement | null }).parentElement =
          scrollContent as unknown as HTMLElement;
        return canvas;
      },
      removeChild: (canvas: HTMLCanvasElement) => {
        const index = children.indexOf(canvas);
        assert.ok(index >= 0, 'DOM에 있는 Canvas만 제거해야 한다');
        children.splice(index, 1);
        (canvas as unknown as { parentElement: HTMLElement | null }).parentElement = null;
        removeCount += 1;
        return canvas;
      },
      querySelectorAll: () => [],
    };
    Object.defineProperty(globalThis, 'document', {
      configurable: true,
      value: {
        createElement: (tagName: string) => {
          assert.equal(tagName, 'canvas');
          return {
            classList: { add: () => undefined },
            dataset: {},
            parentElement: null,
            style: {},
          } as unknown as HTMLCanvasElement;
        },
      },
    });

    const virtualScroll = new VirtualScroll(10);
    const originalSetPageDimensions = virtualScroll.setPageDimensions.bind(virtualScroll);
    let layoutCommitCount = 0;
    virtualScroll.setPageDimensions = (...args: Parameters<VirtualScroll['setPageDimensions']>) => {
      layoutCommitCount += 1;
      originalSetPageDimensions(...args);
    };

    let zoom = 1.01;
    let scrollLeft = 0;
    let scrollTop = 0;
    const viewportManager = {
      getZoom: () => zoom,
      getViewportSize: () => ({ width: 817, height: 600 }),
      getScrollX: () => scrollLeft,
      getScrollY: () => scrollTop,
      setScrollLeft: (value: number) => { scrollLeft = value; },
      setScrollTop: (value: number) => { scrollTop = value; },
      isZoomAnimating: () => false,
    };
    const canvasPool = new CanvasPool();
    const view = Object.create(CanvasView.prototype) as Record<string, unknown>;
    view.virtualScroll = virtualScroll;
    view.canvasPool = canvasPool;
    view.viewportManager = viewportManager;
    view.pages = pages(3, 400, 600);
    view.pageArrangement = { kind: 'auto' };
    view.pageMovement = { direction: 'vertical', wheelHorizontal: false };
    view.scrollContent = scrollContent;
    view.eventBus = { emit: () => undefined };
    view.pageRenderer = {
      cancelAll: () => undefined,
      resetImageRetryState: () => undefined,
      removeAllPageLayers: () => undefined,
    };
    view.renderSurfacePlan = null;
    view.renderSurfaceDecisions = new Map();
    view.cancelPendingTextEditRefresh = () => undefined;
    view.cancelTextEditStaticLayerVerification = () => undefined;
    view.removeHeaderFooterEditOverlays = () => undefined;
    view.removeAllGridOverlays = () => undefined;
    view.updateVisiblePages = () => {
      const canvas = canvasPool.acquire(0);
      (scrollContent.appendChild as (canvas: HTMLCanvasElement) => HTMLCanvasElement)(canvas);
    };

    const initialCanvas = canvasPool.acquire(0);
    (scrollContent.appendChild as (canvas: HTMLCanvasElement) => HTMLCanvasElement)(initialCanvas);

    const invoke = (name: string, ...args: unknown[]): unknown => {
      const method = view[name];
      assert.equal(typeof method, 'function', `${name} 메서드가 있어야 한다`);
      return (method as (...values: unknown[]) => unknown).apply(view, args);
    };
    const assertSingleCanvasOwnership = (): void => {
      assert.deepEqual(canvasPool.activePages, [0]);
      assert.equal(children.length, canvasPool.activePages.length);
      assert.equal(new Set(canvasPool.activePages).size, canvasPool.activePages.length);
      assert.equal(canvasPool.getCanvas(0), children[0]);
    };

    invoke('recalcLayout');
    assert.equal(virtualScroll.getColumns(), 1);

    zoom = 1;
    invoke('onZoomChanged', zoom, { x: 0.5, y: 0.5 });
    assert.equal(virtualScroll.getColumns(), 1, '증가 dead band 안에서는 1열 유지');
    assertSingleCanvasOwnership();

    zoom = 0.99;
    invoke('onZoomChanged', zoom, { x: 0.5, y: 0.5 });
    assert.equal(virtualScroll.getColumns(), 2, 'dead band를 지난 뒤 2열 commit');
    assertSingleCanvasOwnership();

    zoom = 1;
    invoke('onZoomChanged', zoom, { x: 0.5, y: 0.5 });
    assert.equal(virtualScroll.getColumns(), 2, '반대 방향 dead band 안에서는 2열 유지');
    assertSingleCanvasOwnership();

    assert.equal(layoutCommitCount, 4, '초기 1회와 zoom event당 1회만 레이아웃 commit');
    assert.equal(removeCount, 3, 'settled zoom마다 기존 Canvas를 반환한 뒤 하나만 다시 할당');

    view.cancelPendingPrefetch = () => undefined;
    view.currentVisiblePages = [0];
    view.currentRetainedPages = [0];
    view.editingPageIndex = null;
    view.headerFooterEditState = null;
    view.activePageSnapshot = null;
    view.previousEffectiveDpr = new Map();
    (view.pageRenderer as Record<string, unknown>).setPageMarginGuideEdges = () => undefined;
    (scrollContent as Record<string, unknown>).replaceChildren = () => { children.length = 0; };
    invoke('reset');
    assert.equal(virtualScroll.pageCount, 0, 'CanvasView reset은 이전 문서 geometry도 함께 비운다');
    assert.deepEqual(
      virtualScroll.getVisibilitySnapshot(scrollTop, 600, scrollLeft, 817).visiblePages,
      [],
      'reset 뒤 늦은 viewport 갱신은 이전 page를 요청하지 않는다',
    );
  } finally {
    if (previousDocument) {
      Object.defineProperty(globalThis, 'document', previousDocument);
    } else {
      Reflect.deleteProperty(globalThis, 'document');
    }
    await vite.close();
  }
});
