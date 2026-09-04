import test from 'node:test';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
import { createServer } from 'vite';

import {
  PageRenderScheduler,
  type PageRenderSchedulerHost,
} from '../src/view/page-render-scheduler.ts';

class FrameHost implements PageRenderSchedulerHost {
  nextId = 0;
  frames = new Map<number, () => void>();
  idles = new Map<number, () => void>();
  timers = new Map<number, () => void>();

  now(): number { return 0; }
  requestFrame(callback: () => void): number {
    const id = ++this.nextId;
    this.frames.set(id, callback);
    return id;
  }
  cancelFrame(id: number): void { this.frames.delete(id); }
  requestIdle(callback: () => void): number {
    const id = ++this.nextId;
    this.idles.set(id, callback);
    return id;
  }
  cancelIdle(id: number): void { this.idles.delete(id); }
  setTimeout(callback: () => void): number {
    const id = ++this.nextId;
    this.timers.set(id, callback);
    return id;
  }
  clearTimeout(id: number): void { this.timers.delete(id); }

  runFrame(): void {
    const next = this.frames.entries().next().value as ([number, () => void] | undefined);
    assert.ok(next);
    this.frames.delete(next[0]);
    next[1]();
  }

  runTimer(): void {
    const next = this.timers.entries().next().value as ([number, () => void] | undefined);
    assert.ok(next);
    this.timers.delete(next[0]);
    next[1]();
  }
}

for (const reason of ['scroll', 'scroll-settled'] as const) {
  for (const focus of [0, null]) {
    test(`${reason} 큐가 대기 중 focus=${focus}로 plan이 바뀌어도 미생성 visible을 완료한다`, async () => {
      const vite = await createServer({
        root: fileURLToPath(new URL('..', import.meta.url)),
        appType: 'custom', logLevel: 'silent', server: { middlewareMode: true },
      });
      const previousWindow = Object.getOwnPropertyDescriptor(globalThis, 'window');
      try {
        Object.defineProperty(globalThis, 'window', { configurable: true, value: { devicePixelRatio: 2 } });
        const { CanvasView } = await vite.ssrLoadModule('/src/view/canvas-view.ts');
        const host = new FrameHost();
        const visible = [0, 1, 2, 3];
        const surfaces = new Map<number, { dataset: Record<string, string> }>([
          [0, { dataset: { rhwpRequestedDpr: '2' } }],
        ]);
        const rasterCalls: number[] = [];
        const view = Object.create(CanvasView.prototype) as Record<string, any>;
        const render = (page: number) => {
          rasterCalls.push(page);
          surfaces.set(page, { dataset: {
            rhwpSurfaceCacheLookupKey: view.pageSurfaceDescriptor(page).lookupKey,
            rhwpRequestedDpr: String(view.renderSurfaceDecisions.get(page).effectiveDpr),
          } });
          return true;
        };
        Object.assign(view, {
          pages: Array.from({ length: 5 }, () => ({ width: 1000, height: 1400 })),
          pageMovement: { direction: 'vertical', wheelHorizontal: false },
          viewportManager: {
            getScrollX: () => 0, getScrollY: () => 0, getZoom: () => 0.75,
            getViewportSize: () => ({ width: 1600, height: 1800 }),
          },
          virtualScroll: {
            getVisibilitySnapshot: () => ({ visiblePages: visible, prefetchPages: visible }),
            getPageAtPoint: () => 0, pageCount: 5,
          },
          canvasPool: {
            get activePages() { return [...surfaces.keys()]; },
            has: (page: number) => surfaces.has(page), getCanvas: (page: number) => surfaces.get(page),
          },
          pageRenderer: {
            getBackend: () => 'canvas2d', getRenderProfile: () => 'screen',
            getCanvasSurfaceLayerCount: () => 4,
          },
          pageSurfaceLru: { put: () => true, hasLookup: () => false },
          pageRenderScheduler: new PageRenderScheduler(host), renderWorkGeneration: 0, lastScrollSample: null,
          currentVisiblePages: visible, currentRetainedPages: visible, editingPageIndex: 4,
          activePageSnapshot: { pageIndex: 0, source: 'viewport' }, activeRendererDecisionKey: 'doc:fixture',
          headerFooterEditState: null, disposed: false, renderSurfacePlan: null,
          renderSurfaceDecisions: new Map(), previousEffectiveDpr: new Map(), renderSurfaceEnvironmentKey: null,
          pendingPrefetchSurfaceReservations: new Map(), eventBus: { emit: () => undefined },
          reconcilePageSurfaceBudget: () => undefined, renderHeaderFooterEditOverlays: () => undefined,
          materiallyVisiblePages: () => visible, renderCanvas: render, renderPage: render,
        });
        view.refreshRenderSurfacePlan(false, 'scroll-settled');
        surfaces.get(0)!.dataset.rhwpSurfaceCacheLookupKey = view.pageSurfaceDescriptor(0).lookupKey;
        view.updateVisiblePages(reason);
        const generation = view.renderWorkGeneration;
        const timers = [...host.timers.keys()];
        view.setEditingPageIndex(focus);
        assert.equal(view.renderWorkGeneration, generation, 'focus만 바뀌면 viewport 세대와 정착 예약을 보존한다');
        assert.deepEqual([...host.timers.keys()], timers);
        assert.ok(!rasterCalls.some(page => page !== 0), 'missing visible을 클릭 callback에서 동기로 몰아 그리지 않는다');
        assert.equal(host.frames.size, 1, '기존 예약 frame은 하나만 유지한다');
        for (let i = 0; host.frames.size > 0 && i < 10; i++) host.runFrame();
        assert.deepEqual(visible.filter(page => !surfaces.has(page)), []);
        for (const page of visible) {
          assert.equal(surfaces.get(page)!.dataset.rhwpSurfaceCacheLookupKey, view.pageSurfaceDescriptor(page).lookupKey);
        }
        assert.equal(view.pageRenderScheduler.snapshot().visibleQueued, 0);
        if (reason === 'scroll') {
          host.runTimer();
          assert.equal(view.pageRenderScheduler.snapshot().scrollSettleScheduled, false);
          assert.deepEqual([...view.renderSurfaceDecisions.values()].map((d: any) => d.effectiveDpr), [2, 2, 2, 2]);
        }
      } finally {
        if (previousWindow) Object.defineProperty(globalThis, 'window', previousWindow);
        else Reflect.deleteProperty(globalThis, 'window');
        await vite.close();
      }
    });
  }
}

test('scroll fast path가 throw해도 정착 복구와 남은 visible frame은 예약돼 있다', async () => {
  const vite = await createServer({
    root: fileURLToPath(new URL('..', import.meta.url)),
    appType: 'custom', logLevel: 'silent', server: { middlewareMode: true },
  });
  try {
    const { CanvasView } = await vite.ssrLoadModule('/src/view/canvas-view.ts');
    const host = new FrameHost();
    const rendered: number[] = [];
    let failed = false;
    const view = Object.create(CanvasView.prototype) as Record<string, any>;
    Object.assign(view, {
      pages: [{ width: 100, height: 200 }, { width: 100, height: 200 }],
      pageMovement: { direction: 'vertical', wheelHorizontal: false },
      viewportManager: { getScrollX: () => 0, getScrollY: () => 0, getViewportSize: () => ({ width: 800, height: 600 }) },
      virtualScroll: { getVisibilitySnapshot: () => ({ visiblePages: [0, 1], prefetchPages: [0, 1] }), getPageAtPoint: () => 0 },
      canvasPool: { activePages: [], has: () => false, getCanvas: () => undefined },
      pageSurfaceLru: { put: () => true, hasLookup: () => false },
      pageRenderScheduler: new PageRenderScheduler(host), renderWorkGeneration: 0, lastScrollSample: null,
      currentVisiblePages: [], currentRetainedPages: [], editingPageIndex: null, headerFooterEditState: null, disposed: false,
      updateActivePageSnapshot: () => undefined, refreshRenderSurfacePlan: () => undefined,
      reconcilePageSurfaceBudget: () => undefined, renderHeaderFooterEditOverlays: () => undefined,
      pageSurfaceDescriptor: (page: number) => ({ lookupKey: `page:${page}`, estimatedPixelCount: 20_000 }),
      renderPage: (page: number) => {
        if (!failed) { failed = true; throw new Error('surface allocation failed'); }
        rendered.push(page);
      },
    });
    assert.throws(() => view.updateVisiblePages('scroll'), /surface allocation failed/);
    assert.equal(host.frames.size, 1);
    assert.equal(host.timers.size, 1);
    host.runFrame();
    assert.deepEqual(rendered, [1]);
    host.runTimer();
    while (host.frames.size > 0) host.runFrame();
    assert.ok(rendered.includes(0), '후속 정착 갱신은 실패했던 visible도 다시 판정한다');
  } finally { await vite.close(); }
});

test('실제 CanvasView update 경계는 많은 scroll visible만 분할하고 initial은 동기 보존한다', async () => {
  const studioRoot = fileURLToPath(new URL('..', import.meta.url));
  const vite = await createServer({
    root: studioRoot,
    appType: 'custom',
    logLevel: 'silent',
    server: { middlewareMode: true },
  });
  try {
    const { CanvasView } = await vite.ssrLoadModule('/src/view/canvas-view.ts');
    const host = new FrameHost();
    const scheduler = new PageRenderScheduler(host);
    const rendered = new Set<number>();
    const calls: number[] = [];
    let keyRevision = 1;
    const canvasPool = {
      activePages: [] as number[],
      has: (page: number) => rendered.has(page),
      getCanvas: () => undefined,
    };
    const view = Object.create(CanvasView.prototype) as Record<string, any>;
    view.pages = Array.from({ length: 4 }, () => ({ width: 100, height: 200 }));
    view.pageMovement = { direction: 'vertical', wheelHorizontal: false };
    view.viewportManager = {
      getScrollX: () => 0,
      getScrollY: () => 100,
      getViewportSize: () => ({ width: 800, height: 600 }),
    };
    view.virtualScroll = {
      getVisibilitySnapshot: () => ({
        geometryRevision: 1,
        scrollX: 0,
        scrollY: 100,
        viewportWidth: 800,
        viewportHeight: 600,
        visiblePages: [0, 1, 2, 3],
        prefetchPages: [0, 1, 2, 3],
      }),
      getPageAtPoint: () => 1,
    };
    view.canvasPool = canvasPool;
    view.pageSurfaceLru = { put: () => true, hasLookup: () => false };
    view.pageRenderScheduler = scheduler;
    view.renderWorkGeneration = 0;
    view.lastScrollSample = null;
    view.currentVisiblePages = [];
    view.currentRetainedPages = [];
    view.editingPageIndex = null;
    view.headerFooterEditState = null;
    view.disposed = false;
    view.updateActivePageSnapshot = () => undefined;
    view.refreshRenderSurfacePlan = () => undefined;
    view.reconcilePageSurfaceBudget = () => undefined;
    view.renderHeaderFooterEditOverlays = () => undefined;
    view.pageSurfaceDescriptor = (page: number) => ({
      lookupKey: `revision:${keyRevision}|page:${page}`,
      estimatedPixelCount: 20_000,
    });
    view.renderPage = (page: number) => {
      rendered.add(page);
      calls.push(page);
    };

    view.updateVisiblePages('scroll');
    assert.deepEqual(calls, [], '3쪽 이상 scroll visible은 입력 callback에서 동기 raster하지 않는다');
    assert.equal(host.frames.size, 1);

    keyRevision = 2;
    view.updateVisiblePages('scroll');
    assert.equal(host.frames.size, 1, '새 scroll도 이미 예약된 frame을 재사용한다');
    host.runFrame();
    assert.deepEqual(calls, [1, 0], 'viewport 중심부터 한 slice 최대 두 쪽을 처리한다');
    host.runFrame();
    assert.deepEqual(calls, [1, 0, 2, 3]);

    rendered.clear();
    calls.length = 0;
    view.updateVisiblePages('initial');
    assert.deepEqual(calls, [0, 1, 2, 3], 'initial visible은 열 수와 무관하게 동기 완료한다');
    assert.equal(host.frames.size, 0);
  } finally {
    await vite.close();
  }
});

test('실제 CanvasView scroll 경계도 1·2 visible은 동기 fast path를 유지한다', async () => {
  const studioRoot = fileURLToPath(new URL('..', import.meta.url));
  const vite = await createServer({
    root: studioRoot,
    appType: 'custom',
    logLevel: 'silent',
    server: { middlewareMode: true },
  });
  try {
    const { CanvasView } = await vite.ssrLoadModule('/src/view/canvas-view.ts');
    const host = new FrameHost();
    const calls: number[] = [];
    const view = Object.create(CanvasView.prototype) as Record<string, any>;
    Object.assign(view, {
      pages: [{ width: 100, height: 200 }, { width: 100, height: 200 }],
      pageMovement: { direction: 'vertical', wheelHorizontal: false },
      viewportManager: {
        getScrollX: () => 0,
        getScrollY: () => 100,
        getViewportSize: () => ({ width: 800, height: 600 }),
      },
      virtualScroll: {
        getVisibilitySnapshot: () => ({ visiblePages: [0, 1], prefetchPages: [0, 1] }),
        getPageAtPoint: () => 0,
      },
      canvasPool: { activePages: [], has: () => false, getCanvas: () => undefined },
      pageSurfaceLru: { put: () => true, hasLookup: () => false },
      pageRenderScheduler: new PageRenderScheduler(host),
      renderWorkGeneration: 0,
      lastScrollSample: null,
      currentVisiblePages: [],
      currentRetainedPages: [],
      editingPageIndex: null,
      headerFooterEditState: null,
      disposed: false,
      updateActivePageSnapshot: () => undefined,
      refreshRenderSurfacePlan: () => undefined,
      reconcilePageSurfaceBudget: () => undefined,
      renderHeaderFooterEditOverlays: () => undefined,
      pageSurfaceDescriptor: (page: number) => ({ lookupKey: `page:${page}`, estimatedPixelCount: 20_000 }),
      renderPage: (page: number) => calls.push(page),
    });

    view.updateVisiblePages('scroll');
    assert.deepEqual(calls, [0, 1]);
    assert.equal(host.frames.size, 0);
  } finally {
    await vite.close();
  }
});

test('CanvasView는 scroll 중 surface를 유지하고 정착 승격은 center-first frame work로 실행한다', async () => {
  const studioRoot = fileURLToPath(new URL('..', import.meta.url));
  const vite = await createServer({
    root: studioRoot,
    appType: 'custom',
    logLevel: 'silent',
    server: { middlewareMode: true },
  });
  try {
    const { CanvasView } = await vite.ssrLoadModule('/src/view/canvas-view.ts');
    const host = new FrameHost();
    const phases: Array<[boolean, string]> = [];
    const calls: number[] = [];
    const view = Object.create(CanvasView.prototype) as Record<string, any>;
    Object.assign(view, {
      pages: Array.from({ length: 3 }, () => ({ width: 100, height: 200 })),
      pageMovement: { direction: 'vertical', wheelHorizontal: false },
      viewportManager: {
        getScrollX: () => 0,
        getScrollY: () => 100,
        getViewportSize: () => ({ width: 800, height: 600 }),
      },
      virtualScroll: {
        getVisibilitySnapshot: () => ({ visiblePages: [0, 1, 2], prefetchPages: [0, 1, 2] }),
        getPageAtPoint: () => 1,
      },
      canvasPool: { activePages: [], has: () => false, getCanvas: () => undefined },
      pageSurfaceLru: { put: () => true, hasLookup: () => false },
      pageRenderScheduler: new PageRenderScheduler(host, { scrollSettleDelayMs: 150 }),
      renderWorkGeneration: 0,
      lastScrollSample: null,
      currentVisiblePages: [],
      currentRetainedPages: [],
      editingPageIndex: 0,
      headerFooterEditState: null,
      disposed: false,
      updateActivePageSnapshot: () => undefined,
      refreshRenderSurfacePlan: (sync: boolean, phase: string) => phases.push([sync, phase]),
      reconcilePageSurfaceBudget: () => undefined,
      renderHeaderFooterEditOverlays: () => undefined,
      pageSurfaceDescriptor: (page: number) => ({ lookupKey: `page:${page}`, estimatedPixelCount: 20_000 }),
      renderPage: (page: number) => calls.push(page),
    });

    view.updateVisiblePages('scroll');
    assert.deepEqual(phases, [[false, 'scrolling']]);
    assert.equal(host.timers.size, 1, '마지막 scroll 뒤 정착 callback 하나를 예약한다');
    assert.deepEqual(calls, []);

    host.runTimer();
    assert.deepEqual(phases, [[false, 'scrolling'], [false, 'scroll-settled']]);
    assert.deepEqual(calls, [], '정착 승격도 timer callback 안에서 raster하지 않는다');
    assert.equal(view.editingPageIndex, 0, 'viewport 정착은 편집 focus를 바꾸지 않는다');
    assert.equal(host.frames.size, 1);

    host.runFrame();
    assert.equal(calls[0], 1, '정착 승격은 기존 편집 focus보다 viewport 중심 쪽을 먼저 그린다');
  } finally {
    await vite.close();
  }
});

test('CanvasView planner는 scroll 중 실제 requested DPR을 잠그고 정착 visible을 raw로 회복한다', async () => {
  const studioRoot = fileURLToPath(new URL('..', import.meta.url));
  const vite = await createServer({
    root: studioRoot,
    appType: 'custom',
    logLevel: 'silent',
    server: { middlewareMode: true },
  });
  const windowDescriptor = Object.getOwnPropertyDescriptor(globalThis, 'window');
  try {
    Object.defineProperty(globalThis, 'window', {
      configurable: true,
      value: { devicePixelRatio: 2 },
    });
    const { CanvasView } = await vite.ssrLoadModule('/src/view/canvas-view.ts');
    const surfaces = [
      { dataset: { rhwpRequestedDpr: '1' } },
      { dataset: { rhwpRequestedDpr: '2' } },
    ];
    const view = Object.create(CanvasView.prototype) as Record<string, any>;
    Object.assign(view, {
      pages: Array.from({ length: 2 }, () => ({ width: 1000, height: 1000 })),
      currentVisiblePages: [0, 1],
      currentRetainedPages: [0, 1],
      editingPageIndex: null,
      activePageSnapshot: { pageIndex: 0, source: 'viewport' },
      renderSurfacePlan: null,
      renderSurfaceDecisions: new Map(),
      previousEffectiveDpr: new Map(),
      renderSurfaceEnvironmentKey: null,
      pendingPrefetchSurfaceReservations: new Map(),
      canvasPool: {
        activePages: [0, 1],
        getCanvas: (page: number) => surfaces[page],
      },
      pageRenderer: {
        getBackend: () => 'canvas2d',
        getRenderProfile: () => 'screen',
        getCanvasSurfaceLayerCount: () => 4,
      },
      viewportManager: { getZoom: () => 1 },
      materiallyVisiblePages: () => [0, 1],
      pageSurfaceDescriptor: () => null,
      reconcilePageSurfaceBudget: () => undefined,
    });

    view.refreshRenderSurfacePlan(false, 'scrolling');
    assert.deepEqual(
      [...view.renderSurfaceDecisions.values()].map((decision: any) => decision.effectiveDpr),
      [1, 2],
    );

    view.refreshRenderSurfacePlan(false, 'scroll-settled');
    assert.deepEqual(
      [...view.renderSurfaceDecisions.values()].map((decision: any) => decision.effectiveDpr),
      [2, 2],
    );
  } finally {
    if (windowDescriptor) Object.defineProperty(globalThis, 'window', windowDescriptor);
    else Reflect.deleteProperty(globalThis, 'window');
    await vite.close();
  }
});

test('scroll exact LRU hit는 raster queue 없이 retained working set에 모두 재부착한다', async () => {
  const studioRoot = fileURLToPath(new URL('..', import.meta.url));
  const vite = await createServer({
    root: studioRoot,
    appType: 'custom',
    logLevel: 'silent',
    server: { middlewareMode: true },
  });
  try {
    const { CanvasView } = await vite.ssrLoadModule('/src/view/canvas-view.ts');
    const host = new FrameHost();
    const attached = new Set<number>();
    const renderCalls: number[] = [];
    const cached = new Set([0, 3]);
    const view = Object.create(CanvasView.prototype) as Record<string, any>;
    Object.assign(view, {
      pages: Array.from({ length: 4 }, () => ({ width: 100, height: 200 })),
      pageMovement: { direction: 'vertical', wheelHorizontal: false },
      viewportManager: {
        getScrollX: () => 0,
        getScrollY: () => 100,
        getViewportSize: () => ({ width: 800, height: 600 }),
      },
      virtualScroll: {
        getVisibilitySnapshot: () => ({ visiblePages: [1, 2], prefetchPages: [0, 1, 2, 3] }),
        getPageAtPoint: () => 1,
      },
      canvasPool: {
        activePages: [],
        has: (page: number) => attached.has(page),
        getCanvas: (page: number) => attached.has(page)
          ? { dataset: { rhwpSurfaceCacheLookupKey: `page:${page}` } }
          : undefined,
      },
      pageSurfaceLru: {
        put: () => true,
        hasLookup: (key: string) => cached.has(Number(key.slice('page:'.length))),
      },
      pageRenderScheduler: new PageRenderScheduler(host),
      renderWorkGeneration: 0,
      lastScrollSample: null,
      currentVisiblePages: [],
      currentRetainedPages: [],
      editingPageIndex: null,
      headerFooterEditState: null,
      disposed: false,
      updateActivePageSnapshot: () => undefined,
      refreshRenderSurfacePlan: () => undefined,
      reconcilePageSurfaceBudget: () => undefined,
      renderHeaderFooterEditOverlays: () => undefined,
      pageSurfaceDescriptor: (page: number) => ({ lookupKey: `page:${page}` }),
      renderPage: (page: number) => {
        renderCalls.push(page);
        attached.add(page);
        cached.delete(page);
      },
    });

    view.updateVisiblePages('scroll');
    assert.deepEqual(
      renderCalls,
      [0, 3, 1, 2],
      'exact hit 둘을 raster 대상 visible fast path보다 먼저 재부착한다',
    );
    assert.equal(host.frames.size, 0, '남은 visible 둘은 fast path에서 동기 완료한다');
    assert.deepEqual([...attached].sort(), [0, 1, 2, 3]);
  } finally {
    await vite.close();
  }
});

test('CanvasView 방향 전환은 overscan 후보를 늘리지 않고 진행 방향 prefetch만 앞세운다', async () => {
  const studioRoot = fileURLToPath(new URL('..', import.meta.url));
  const vite = await createServer({
    root: studioRoot,
    appType: 'custom',
    logLevel: 'silent',
    server: { middlewareMode: true },
  });
  try {
    const { CanvasView } = await vite.ssrLoadModule('/src/view/canvas-view.ts');
    const view = Object.create(CanvasView.prototype) as Record<string, any>;
    Object.assign(view, {
      canvasPool: { getCanvas: () => undefined },
      pageSurfaceLru: {
        hasLookup: () => false,
        snapshot: () => ({
          pixelBudget: 1_000_000,
          reservedPixels: 0,
          cachedPixels: 0,
          totalAccountedPixels: 0,
          overBudgetMandatory: false,
        }),
      },
      pageRenderScheduler: { recordPrefetchAdmissionRejected: () => undefined },
      pendingPrefetchSurfaceReservations: new Map(),
      reconcilePageSurfaceBudget: () => undefined,
      pageSurfaceDescriptor: (page: number) => ({
        lookupKey: `page:${page}`,
        estimatedPixelCount: 20_000,
      }),
    });
    const invoke = (motion: { direction: -1 | 1; speed: number }) => (
      view.buildPrefetchRenderWork([0, 3], [1, 2], motion, 1) as Array<{
        pageIndex: number;
        priority: number;
      }>
    );

    const forward = invoke({ direction: 1, speed: 8 });
    const backward = invoke({ direction: -1, speed: 8 });
    assert.deepEqual(forward.map(work => work.pageIndex).sort(), [0, 3]);
    assert.deepEqual(backward.map(work => work.pageIndex).sort(), [0, 3]);
    assert.ok(
      forward.find(work => work.pageIndex === 3)!.priority
        < forward.find(work => work.pageIndex === 0)!.priority,
    );
    assert.ok(
      backward.find(work => work.pageIndex === 0)!.priority
        < backward.find(work => work.pageIndex === 3)!.priority,
    );
  } finally {
    await vite.close();
  }
});

test('낡은 active surface는 detach admission 전에 목표 DPR 크기로 예약한다', async () => {
  const studioRoot = fileURLToPath(new URL('..', import.meta.url));
  const vite = await createServer({
    root: studioRoot,
    appType: 'custom',
    logLevel: 'silent',
    server: { middlewareMode: true },
  });
  try {
    const { CanvasView } = await vite.ssrLoadModule('/src/view/canvas-view.ts');
    const reconciled: Array<{ budget: number; reserved: number }> = [];
    const view = Object.create(CanvasView.prototype) as Record<string, any>;
    Object.assign(view, {
      currentVisiblePages: [18, 19],
      pendingPrefetchSurfaceReservations: new Map(),
      renderSurfacePlan: {
        retainedPixelBudget: 40_000,
        decisions: [{ pageIndex: 18 }, { pageIndex: 19 }],
      },
      canvasPool: {
        getCanvas: (page: number) => ({
          dataset: {
            rhwpSurfaceCacheLookupKey: `old:${page}`,
            rhwpActualSurfacePixels: '30000',
          },
        }),
      },
      pageSurfaceDescriptor: (page: number) => ({
        lookupKey: `target:${page}`,
        estimatedPixelCount: 7_500,
      }),
      pageSurfaceLru: {
        hasLookup: () => false,
        reconcile: (budget: number, reserved: number) => reconciled.push({ budget, reserved }),
      },
    });

    view.reconcilePageSurfaceBudget();
    assert.deepEqual(reconciled, [{ budget: 40_000, reserved: 15_000 }]);
  } finally {
    await vite.close();
  }
});

test('비가시 missing 후보는 승인 전 mandatory 예약에서 제외하고 실제 headroom으로 gate한다', async () => {
  const studioRoot = fileURLToPath(new URL('..', import.meta.url));
  const vite = await createServer({
    root: studioRoot,
    appType: 'custom',
    logLevel: 'silent',
    server: { middlewareMode: true },
  });
  try {
    const { CanvasView } = await vite.ssrLoadModule('/src/view/canvas-view.ts');
    const rejected: number[] = [];
    const reservations = new Map<number, number>();
    let reconciledReserved = 0;
    let cachedPixels = 15;
    let visiblePixels = 70;
    const view = Object.create(CanvasView.prototype) as Record<string, any>;
    Object.assign(view, {
      currentVisiblePages: [4],
      pendingPrefetchSurfaceReservations: reservations,
      renderSurfacePlan: {
        retainedPixelBudget: 100,
        decisions: [
          { pageIndex: 4 },
          { pageIndex: 5 },
          { pageIndex: 6 },
        ],
      },
      canvasPool: { getCanvas: () => undefined },
      pageSurfaceDescriptor: (page: number) => ({
        lookupKey: `page:${page}`,
        estimatedPixelCount: page === 4 ? visiblePixels : 20,
      }),
      pageSurfaceLru: {
        hasLookup: () => false,
        reconcile: (budget: number, reserved: number) => {
          reconciledReserved = reserved;
          cachedPixels = Math.min(cachedPixels, Math.max(0, budget - reserved));
        },
        snapshot: () => {
          return {
            pixelBudget: 100,
            reservedPixels: reconciledReserved,
            cachedPixels,
            totalAccountedPixels: reconciledReserved + cachedPixels,
            overBudgetMandatory: reconciledReserved > 100,
          };
        },
      },
      pageRenderScheduler: {
        recordPrefetchAdmissionRejected: (count = 1) => rejected.push(count),
      },
    });

    view.reconcilePageSurfaceBudget();
    assert.equal(
      reconciledReserved,
      70,
      'visible missing만 mandatory이며 비가시 후보 둘은 아직 승인되지 않는다',
    );
    assert.equal(view.tryReservePrefetchSurface(5, 35), false);
    assert.deepEqual([...reservations], []);
    assert.deepEqual(rejected, [1]);

    assert.equal(view.tryReservePrefetchSurface(5, 20), true);
    assert.deepEqual([...reservations], [[5, 20]]);
    assert.equal(cachedPixels, 10, '승인한 진행 방향 prefetch는 오래된 LRU headroom만 회수한다');
    visiblePixels = 85;
    view.reconcilePageSurfaceBudget();
    assert.equal(
      view.hasValidPrefetchSurfaceReservation(5, 20),
      false,
      'visible raster 뒤 actual ledger가 커지면 idle dispatch 직전에 승인을 회수한다',
    );
    assert.deepEqual([...reservations], []);
    assert.deepEqual(rejected, [1, 1]);
  } finally {
    await vite.close();
  }
});

test('visible focused 쪽은 기존 surface의 key가 낡았어도 새 비포커스 쪽보다 우선한다', async () => {
  const studioRoot = fileURLToPath(new URL('..', import.meta.url));
  const vite = await createServer({
    root: studioRoot,
    appType: 'custom',
    logLevel: 'silent',
    server: { middlewareMode: true },
  });
  try {
    const { CanvasView } = await vite.ssrLoadModule('/src/view/canvas-view.ts');
    const view = Object.create(CanvasView.prototype) as Record<string, any>;
    Object.assign(view, {
      editingPageIndex: 3,
      canvasPool: {
        getCanvas: (page: number) => page === 3
          ? { dataset: { rhwpSurfaceCacheLookupKey: 'stale' } }
          : undefined,
      },
      pageSurfaceDescriptor: (page: number) => ({ lookupKey: `current:${page}` }),
    });
    const work = view.buildVisibleRenderWork([0, 1, 2, 3], 1, 1) as Array<{
      pageIndex: number;
      priority: number;
    }>;
    const ordered = [...work].sort((a, b) => a.priority - b.priority);
    assert.equal(ordered[0].pageIndex, 3);
    assert.equal(ordered[1].pageIndex, 1);

    const settled = view.buildVisibleRenderWork([0, 1, 2, 3], 1, 1, true) as Array<{
      pageIndex: number;
      priority: number;
    }>;
    const settledOrder = [...settled].sort((a, b) => a.priority - b.priority);
    assert.equal(settledOrder[0].pageIndex, 1, 'scroll 정착은 viewport 중심을 우선한다');
  } finally {
    await vite.close();
  }
});
