import test, { before, after } from 'node:test';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
import { createServer, type ViteDevServer } from 'vite';
import { VirtualScroll } from '../src/view/virtual-scroll.ts';
import type { PageArrangement } from '../src/view/page-arrangement.ts';
import { ViewportManager } from '../src/view/viewport-manager.ts';
import { EventBus } from '../src/core/event-bus.ts';

let vite: ViteDevServer;
let canvasViewPrototype: Record<string, any>;
before(async () => {
  vite = await createServer({
    root: fileURLToPath(new URL('..', import.meta.url)),
    appType: 'custom', logLevel: 'silent', server: { middlewareMode: true },
  });
  canvasViewPrototype = (await vite.ssrLoadModule('/src/view/canvas-view.ts')).CanvasView.prototype;
});
after(async () => { await vite?.close(); });

function fixture(arrangement: PageArrangement, direction: 'vertical' | 'horizontal') {
  const virtualScroll = new VirtualScroll(10);
  const pages = Array.from({ length: 6 }, (_, index) => ({
    width: index % 2 === 0 ? 800 : 600, height: 1100,
  }));
  const state = { zoom: 1, animating: false, pending: false, generation: 1, width: 1200, height: 700, x: 0, y: 400 };
  const counts = { preview: 0, dimensions: 0, queries: 0, styleWrites: 0 };
  const trace: string[] = [];
  const makeElement = (renderedZoom: number) => ({
    dataset: { rhwpRenderedZoom: String(renderedZoom) },
    style: new Proxy({} as Record<string, string>, {
      set: (style, name: string, value: string) => {
        counts.styleWrites++;
        style[name] = value;
        return true;
      },
    }),
  });
  // 서로 다른 renderedZoom의 기존 surface도 각 bitmap 배율을 기준으로 preview한다.
  const main = new Map([0, 1, 2].map(page => [page, makeElement(page === 1 ? 0.75 : 1)]));
  const overlays = new Map([0, 1, 2].map(page => [
    page, Array.from({ length: 3 }, () => makeElement(Number(main.get(page)!.dataset.rhwpRenderedZoom))),
  ]));
  const allElements = () => [...main].flatMap(([page, canvas]) => [canvas, ...overlays.get(page)!]);
  const snapshot = () => allElements().map(element => ({ ...element.style }));
  const scrollContent = {
    style: {}, classList: { toggle: () => undefined },
    querySelectorAll: (selector: string) => {
      counts.queries++;
      const match = selector.match(/data-rhwp-overlay-page="(\d+)"/);
      assert.ok(match);
      const page = Number(match[1]);
      assert.ok(selector.includes('data-rhwp-grid-page="' + page + '"'));
      assert.ok(selector.includes('data-rhwp-hf-edit-page="' + page + '"'));
      return overlays.get(page)!;
    },
  };
  const view = Object.create(canvasViewPrototype) as Record<string, any>;
  const originalDimensions = virtualScroll.setPageDimensions.bind(virtualScroll);
  virtualScroll.setPageDimensions = (...args: Parameters<VirtualScroll['setPageDimensions']>) => {
    counts.dimensions++;
    originalDimensions(...args);
  };
  Object.assign(view, {
    virtualScroll, pages, scrollContent, pageArrangement: arrangement,
    pageMovement: { direction, wheelHorizontal: false },
    viewportManager: {
      getZoom: () => state.zoom, isZoomAnimating: () => state.animating,
      isZoomRasterPending: () => state.pending,
      isCurrentZoomRasterReady: (generation: number) => generation === state.generation && !state.pending,
      cancelPendingZoomRaster: () => {
        const pending = state.pending;
        state.pending = false;
        if (pending) { state.animating = false; state.generation++; }
        return pending;
      },
      getViewportSize: () => ({ width: state.width, height: state.height }),
      getScrollX: () => state.x, getScrollY: () => state.y,
      setScrollLeft: (value: number) => { state.x = value; },
      setScrollTop: (value: number) => {
        state.y = Math.max(0, Math.min(value, virtualScroll.getTotalHeight() - state.height));
      },
    },
    canvasPool: { get activePages() { return [...main.keys()]; }, getCanvas: (page: number) => main.get(page) },
    eventBus: { emit: (event: string) => {
      assert.equal(event, 'zoom-level-display');
      trace.push('display');
    } },
    cancelPendingTextEditRefresh: () => trace.push('cancel-edit'),
    cancelTextEditStaticLayerVerification: () => trace.push('cancel-verification'),
    cancelPendingPrefetch: () => trace.push('cancel-prefetch'),
    releaseAllRenderedPages: () => trace.push('release'),
    pageRenderer: { cancelAll: () => trace.push('cancel-render') },
    updateVisiblePages: (reason: string) => trace.push(reason),
  });
  view.updateRenderedPageZoomPreview = () => {
    counts.preview++;
    canvasViewPrototype.updateRenderedPageZoomPreview.call(view);
  };
  view.recalcLayout();
  const resetCounts = () => {
    Object.assign(counts, { preview: 0, dimensions: 0, queries: 0, styleWrites: 0 });
    trace.length = 0;
  };
  resetCounts();
  return { view, virtualScroll, state, counts, trace, main, overlays, snapshot, resetCounts };
}

const arrangements: PageArrangement[] = [
  { kind: 'auto' }, { kind: 'single' }, { kind: 'double' },
  { kind: 'facing' }, { kind: 'multiple', columns: 3, rows: 2 },
];
for (const arrangement of arrangements) {
  test(arrangement.kind + ': 수렴 뒤 quiet 대기에도 preview·공통 좌표를 유지한다', () => {
    const f = fixture(arrangement, 'vertical');
    f.state.pending = true;
    f.state.animating = false;
    for (const zoom of [0.99, 0.5, 0.34, 0.9, 1.13]) {
      f.state.zoom = zoom;
      f.resetCounts();
      f.view.onZoomChanged(zoom, { x: 0.3, y: 0.7 });
      assert.equal(f.counts.preview, 1);
      assert.equal(f.counts.dimensions, 1);
      assert(!f.trace.includes('release'));
      const beforeScroll = f.snapshot();
      const beforeTrace = [...f.trace];
      canvasViewPrototype.updateVisiblePages.call(f.view, 'scroll');
      canvasViewPrototype.updateVisiblePages.call(f.view, 'scroll-settled');
      assert.deepEqual(f.trace, beforeTrace);
      assert.deepEqual(f.snapshot(), beforeScroll);
      for (const [page, canvas] of f.main) {
        const scale = zoom / Number(canvas.dataset.rhwpRenderedZoom);
        assert(canvas.style.transform.endsWith('scale(' + scale + ')'));
        assert.equal(canvas.style.top, f.virtualScroll.getPageOffset(page) + 'px');
      }
    }
    f.resetCounts();
    f.view.onZoomRasterReady(f.state.generation);
    assert.deepEqual(f.trace, [], 'pending 상태에서는 ready를 받아도 렌더하지 않는다');
    f.state.pending = false;
    f.view.onZoomRasterReady(f.state.generation - 1);
    assert.deepEqual(f.trace, [], '이전 세대의 ready를 무시한다');
    f.view.onZoomRasterReady(f.state.generation);
    assert.equal(f.counts.dimensions, 0, 'ready에서 geometry/anchor를 다시 계산하지 않는다');
    assert.deepEqual(f.trace, ['cancel-edit', 'cancel-verification', 'zoom-settled']);
  });
}
for (const direction of ['vertical', 'horizontal'] as const) {
  for (const arrangement of arrangements) {
    test(direction + '/' + arrangement.kind + ': animation은 같은 geometry에 preview를 한 번만 적용한다', () => {
      const f = fixture(arrangement, direction);
      f.state.animating = true;
      // 열 전환·반대 방향·광폭 pan과 renderedZoom이 다른 bitmap을 함께 확인한다.
      for (const zoom of [0.75, 0.5, 0.34, 0.5, 1, 1.55]) {
        f.resetCounts();
        f.state.zoom = zoom;
        f.view.onZoomChanged(zoom, { x: 0.3, y: 0.7 });
        assert.equal(f.counts.preview, 1, 'animation event당 중복 preview가 없어야 한다');
        assert.equal(f.counts.dimensions, 1, 'authoritative geometry는 매 event 갱신한다');
        assert.equal(f.counts.queries, 3, '활성 페이지마다 overlay를 한 번 조회한다');
        assert.equal(f.counts.styleWrites, 3 * 4 * 4, 'main + 세 overlay에 각각 네 style만 쓴다');
        assert.deepEqual(f.trace, ['display', 'cancel-edit', 'cancel-verification', 'cancel-prefetch']);
        for (const [page, canvas] of f.main) {
          const pageLeft = f.virtualScroll.getPageLeft(page);
          const scale = zoom / Number(canvas.dataset.rhwpRenderedZoom);
          const expected = {
            top: f.virtualScroll.getPageOffset(page) + 'px',
            left: pageLeft >= 0 ? pageLeft + 'px' : '50%',
            transform: (pageLeft >= 0 ? '' : 'translateX(-50%) ') + 'scale(' + scale + ')',
            transformOrigin: pageLeft >= 0 ? 'top left' : 'top center',
          };
          assert.deepEqual({ ...canvas.style }, expected);
          for (const overlay of f.overlays.get(page)!) assert.deepEqual({ ...overlay.style }, expected);
        }
        const once = f.snapshot();
        // 기존 경로처럼 앵커 scroll 복원 뒤 한 번 더 실행해도 최종 요소 좌표는 같아야 한다.
        f.view.updateRenderedPageZoomPreview();
        assert.deepEqual(f.snapshot(), once);
        assert.equal(f.counts.styleWrites, 3 * 4 * 4 * 2, '두 번째 호출은 같은 style을 다시 쓴다');
      }
    });
  }
}

test('animation 중 직접 resize 재계산도 preview를 한 번 유지한다', () => {
  const f = fixture({ kind: 'auto' }, 'vertical');
  f.state.animating = true;
  f.state.zoom = 0.5;
  f.view.onZoomChanged(0.5, { x: 0.5, y: 0.5 });
  for (const width of [900, 1700, 1200]) {
    f.resetCounts();
    f.state.width = width;
    f.view.recalcLayout();
    assert.equal(f.counts.preview, 1);
    assert.equal(f.counts.dimensions, 1);
    const once = f.snapshot();
    f.view.updateRenderedPageZoomPreview();
    assert.deepEqual(f.snapshot(), once);
    assert.deepEqual(f.trace, [], '직접 layout은 zoom 알림이나 raster를 새로 만들지 않는다');
  }
});

test('버튼 zoom 도중 동일 크기의 resize 알림은 전면 해제·동기 raster를 우회하지 않는다', () => {
  const f = fixture({ kind: 'auto' }, 'vertical');
  f.state.animating = true;
  f.state.zoom = 0.5;
  f.view.onZoomChanged(0.5, { x: 0.5, y: 0.5 });
  const before = f.snapshot();
  f.resetCounts();
  f.view.onViewportResize();
  assert.deepEqual(f.trace, []);
  assert.equal(f.counts.dimensions, 0);
  assert.deepEqual(f.snapshot(), before);
});

for (const pending of [false, true]) {
  test(`문서 로드 앵커 억제는 줌 resize 지연과 함께 유지된다 (quiet=${pending})`, () => {
    const f = fixture({ kind: 'auto' }, 'vertical');
    f.state.animating = !pending;
    f.state.pending = pending;
    f.view.suppressResizeScrollAnchor = true;
    f.state.width -= 15;
    let anchorQueries = 0;
    f.view.getZoomPageBox = () => { anchorQueries++; return null; };
    const scrollY = f.state.y;
    f.view.onViewportResize();
    assert.equal(anchorQueries, 0, '문서 교체 resize에는 이전 페이지 앵커를 사용하지 않는다');
    assert.equal(f.state.y, scrollY);
    assert.equal(f.view.suppressResizeScrollAnchor, false);
    assert.deepEqual(f.trace, ['cancel-prefetch'], 'preview 중 resize는 동기 raster를 실행하지 않는다');
    assert.equal(f.state.pending, pending);
    f.state.width += 15;
    f.view.onViewportResize();
    assert.equal(anchorQueries, 1, '다음 실제 resize에는 읽던 위치 보존을 다시 허용한다');
  });

  test(`zoom 중 높이만 15px 바뀌면 geometry만 갱신하고 마지막 정착을 유지한다 (quiet=${pending})`, () => {
    const f = fixture({ kind: 'auto' }, 'vertical');
    f.state.animating = !pending;
    f.state.pending = pending;
    f.state.zoom = 0.5;
    f.view.onZoomChanged(0.5, { x: 0.5, y: 0.5 });
    for (const height of [685, 700]) {
      f.state.height = height;
      f.resetCounts();
      f.view.onViewportResize();
      assert.equal(f.counts.dimensions, 1);
      assert.equal(f.counts.preview, 1);
      assert.deepEqual(f.trace, ['cancel-prefetch']);
      assert.equal(f.state.pending, pending);
      assert.equal(f.state.animating, !pending);
      assert.equal(f.view.layoutViewportSize.height, height);
    }
    f.state.animating = false;
    f.state.pending = false;
    f.view.onZoomRasterReady(f.state.generation);
    assert.equal(f.trace.filter(x => x === 'zoom-settled').length, 1);
  });
}

test('정착 및 직접 zoom은 전면 해제 없이 개별 갱신까지 main/layer preview를 유지한다', () => {
  const f = fixture({ kind: 'auto' }, 'vertical');
  for (const zoom of [0.34, 1, 1.55]) {
    f.resetCounts();
    f.state.zoom = zoom;
    f.view.onZoomChanged(zoom, { x: 0.5, y: 0.5 });
    assert.equal(f.counts.preview, 0);
    assert.equal(f.counts.dimensions, 1);
    assert.deepEqual(f.trace, [
      'display', 'cancel-edit', 'cancel-verification', 'zoom-settled',
    ]);
    for (const [page, canvas] of f.main) {
      const scale = zoom / Number(canvas.dataset.rhwpRenderedZoom);
      if (scale !== 1) assert(canvas.style.transform.endsWith(`scale(${scale})`));
      for (const overlay of f.overlays.get(page)!) assert.deepEqual(overlay.style, canvas.style);
    }
  }
});

test('문서가 없으면 zoom은 preview와 geometry를 만들지 않는다', () => {
  const f = fixture({ kind: 'auto' }, 'vertical');
  f.view.pages = [];
  f.state.animating = true;
  f.view.onZoomChanged(0.5, { x: 0.5, y: 0.5 });
  assert.deepEqual(f.counts, { preview: 0, dimensions: 0, queries: 0, styleWrites: 0 });
  assert.deepEqual(f.trace, []);
});

test('실제 ViewportManager→CanvasView: 작은 연속 wheel 20건은 전체 해제 없이 최종 예약 1회다', () => {
  const f = fixture({ kind: 'auto' }, 'vertical');
  const bus = new EventBus();
  let at = 0;
  let nextId = 0;
  const timers = new Map<number, () => void>();
  const vm = new ViewportManager(bus, {
    now: () => at,
    schedule: callback => {
      timers.set(++nextId, callback);
      return nextId as unknown as ReturnType<typeof setTimeout>;
    },
    cancel: id => { timers.delete(id as unknown as number); },
  });
  Object.assign(vm, { viewportWidth: f.state.width, viewportHeight: f.state.height });
  f.view.viewportManager = vm;
  bus.on('zoom-changed', (zoom, anchor) => f.view.onZoomChanged(zoom, anchor));
  bus.on('zoom-raster-ready', generation => f.view.onZoomRasterReady(generation));
  for (let i = 0; i < 20; i++) {
    at = i * 16;
    (vm as unknown as { onWheel(e: unknown): void }).onWheel({
      ctrlKey: true, deltaY: .05, deltaX: 0, deltaMode: 0, preventDefault() {},
    });
    assert.equal(f.trace.filter(x=>x==='release').length, 0);
    assert.equal(f.counts.preview, i + 1);
    assert.equal(vm.isZoomAnimating(), false);
  }
  const geometryCalls = f.counts.dimensions;
  at = 424;
  [...timers.values()][0]();
  assert.equal(f.trace.filter(x=>x==='release').length, 0);
  assert.equal(f.trace.filter(x=>x==='zoom-settled').length, 1);
  assert.equal(f.counts.dimensions, geometryCalls, 'ready에서는 geometry를 중복 갱신하지 않는다');
  assert.equal(timers.size, 0);
});

test('quiet 대기 중 실제 폭 resize도 마지막 줌의 페이지별 예약을 유지한다', () => {
  const f = fixture({ kind: 'single' }, 'vertical');
  f.state.pending = true;
  f.state.zoom = .5;
  f.view.onZoomChanged(.5, { x: .5, y: .5 });
  f.resetCounts();
  f.view.onViewportResize();
  assert.deepEqual(f.trace, [], '동일 크기 ResizeObserver 알림은 quiet를 깨지 않는다');
  f.state.width = 1000;
  f.view.onViewportResize();
  assert.equal(f.state.pending, true);
  assert.deepEqual(f.trace, ['cancel-prefetch']);
  const before = [...f.trace];
  f.view.onZoomRasterReady(f.state.generation - 1);
  assert.deepEqual(f.trace, before);
  f.state.pending = false;
  f.view.onZoomRasterReady(f.state.generation);
  assert.equal(f.trace.filter(x => x === 'zoom-settled').length, 1);
});

for (const direction of ['vertical', 'horizontal'] as const) {
  for (const arrangement of arrangements) {
    for (const pending of [false, true]) {
      test(`${direction}/${arrangement.kind}/quiet=${pending}: zoom 중 폭 변경은 전역 해제 없이 공통 좌표를 갱신한다`, () => {
        const f = fixture(arrangement, direction);
        f.state.zoom = .5;
        f.state.animating = !pending;
        f.state.pending = pending;
        f.view.onZoomChanged(.5, { x: .5, y: .5 });
        const generation = f.state.generation;
        for (const width of [1185, 1200, 700, 1700]) {
          f.state.width = width;
          f.resetCounts();
          f.view.onViewportResize();
          assert.deepEqual(f.trace, ['cancel-prefetch']);
          assert.equal(f.counts.dimensions, 1);
          assert.equal(f.counts.preview, 1);
          assert.equal(f.state.pending, pending);
          assert.equal(f.state.animating, !pending);
          assert.equal(f.state.generation, generation);
          assert.equal(f.view.layoutViewportSize.width, width);
          for (const [page, canvas] of f.main) {
            assert.equal(canvas.style.top, `${f.virtualScroll.getPageOffset(page)}px`);
            for (const overlay of f.overlays.get(page)!) assert.deepEqual(overlay.style, canvas.style);
          }
          assert(f.state.x >= 0);
          assert(f.state.x <= Math.max(0, f.virtualScroll.getTotalWidth() - width));
        }
        f.state.animating = false;
        f.state.pending = false;
        f.resetCounts();
        if (pending) f.view.onZoomRasterReady(generation);
        else f.view.onZoomChanged(.5, { x: .5, y: .5 });
        assert.equal(f.trace.filter(x => x === 'zoom-settled').length, 1);
        assert(!f.trace.includes('release'));
      });
    }
  }
}

test('idle grid resize는 기존 전면 갱신 계약을 유지한다', () => {
  const f = fixture({ kind: 'multiple', columns: 3, rows: 2 }, 'vertical');
  f.state.width = 1000;
  f.view.onViewportResize();
  assert.deepEqual(f.trace, ['cancel-edit', 'cancel-verification', 'release', 'cancel-render', 'resize']);
});

test('quiet 중 부분 편집은 전체 최신 줌 갱신 경로로 넘긴다', () => {
  const f = fixture({ kind: 'single' }, 'vertical');
  f.state.pending = true;
  let refreshed = 0;
  f.view.refreshPages = () => { refreshed++; f.view.viewportManager.cancelPendingZoomRaster(); };
  f.view.refreshInvalidatedPage({ pageIndex: 0, reason: 'text-edit' });
  assert.equal(refreshed, 1);
  assert.equal(f.state.pending, false);
});

test('비동기 renderer 선택이 교체돼도 중단한 줌의 전체 갱신 책임을 승계한다', async () => {
  const f = fixture({ kind: 'single' }, 'vertical');
  f.state.pending = true;
  f.view.rendererSelectionEpoch = 0;
  f.view.zoomRefreshRequired = false;
  const selection = { backend: 'canvas2d', diagnostics: { decisionKey: 'same' } };
  f.view.activeRendererDecisionKey = 'same';
  f.view.rendererSession = {
    invalidateDocument() {}, resolve: async () => selection, isCurrent: () => true,
  };
  f.view.pageRenderer.configure = () => false;
  f.view.eventBus.emit = () => undefined;
  const superseded = f.view.selectNextDocumentRevision();
  const latest = f.view.selectNextDocumentRevision();
  assert.equal(await superseded, null);
  assert.equal((await latest).backendChanged, true, 'backend가 같아도 최신 줌 전체 갱신은 필요하다');
  assert.equal(f.state.pending, false);
  assert.equal(f.view.zoomRefreshRequired, true);
  f.view.pages = [];
  canvasViewPrototype.refreshPages.call(f.view);
  assert.equal(f.view.zoomRefreshRequired, false);
});

for (const boundary of ['new-selection', 'document', 'dispose'] as const) {
  test(`${boundary}: renderer 초기화 뒤 늦게 도착한 구 선택을 적용하지 않는다`, async () => {
    const f = fixture({ kind: 'single' }, 'vertical');
    f.state.pending = true;
    f.view.rendererSelectionEpoch = 0;
    f.view.zoomRefreshRequired = false;
    const completions: Array<(selection: unknown) => void> = [];
    const applied: string[] = [];
    f.view.rendererSession = {
      invalidateDocument() {}, isCurrent: () => true,
      resolve: () => new Promise(resolve => completions.push(resolve)),
    };
    f.view.applyRendererSelection = (selection: { id: string }) => { applied.push(selection.id); return true; };
    const old = f.view.selectNextDocumentRevision();
    await Promise.resolve();
    assert.equal(completions.length, 1);
    assert.equal(f.state.pending, false);
    if (boundary === 'new-selection') {
      const latest = f.view.selectNextDocumentRevision();
      await Promise.resolve();
      completions[1]({ id: 'latest' });
      assert.equal((await latest).backendChanged, true);
    } else if (boundary === 'document') {
      f.view.rendererSelectionEpoch++; // prepareDocumentLoad가 동기적으로 변경하는 epoch.
    } else {
      f.view.disposed = true;
    }
    completions[0]({ id: 'old' });
    assert.equal(await old, null);
    assert.deepEqual(applied, boundary === 'new-selection' ? ['latest'] : []);
  });
}
