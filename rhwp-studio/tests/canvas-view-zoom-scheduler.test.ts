import test, { before, after } from 'node:test';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
import { createServer, type ViteDevServer } from 'vite';
import { PageRenderScheduler, type PageRenderSchedulerHost } from '../src/view/page-render-scheduler.ts';
import { EventBus } from '../src/core/event-bus.ts';
import { ViewportManager } from '../src/view/viewport-manager.ts';
import { PageSurfaceLru } from '../src/view/page-surface-lru.ts';

let vite: ViteDevServer;
let prototype: Record<string, any>;
let Renderer: any;
before(async () => {
  vite = await createServer({
    root: fileURLToPath(new URL('..', import.meta.url)),
    appType: 'custom', logLevel: 'silent', server: { middlewareMode: true },
  });
  prototype = (await vite.ssrLoadModule('/src/view/canvas-view.ts')).CanvasView.prototype;
  Renderer = (await vite.ssrLoadModule('/src/view/page-renderer.ts')).PageRenderer;
});
after(async () => { await vite?.close(); });

class Host implements PageRenderSchedulerHost {
  time = 0;
  id = 0;
  frames = new Map<number, () => void>();
  now() { return this.time; }
  requestFrame(callback: () => void) { this.frames.set(++this.id, callback); return this.id; }
  cancelFrame(id: number) { this.frames.delete(id); }
  setTimeout() { return ++this.id; }
  clearTimeout() {}
  runFrame() {
    const next = this.frames.entries().next().value;
    assert.ok(next);
    this.frames.delete(next[0]);
    next[1]();
  }
}

function fixture(count = 8, costMs = 0) {
  const host = new Host();
  const surfaces = new Map<number, { dataset: Record<string, string> }>([
    [0, { dataset: { rhwpRenderedZoom: '1', rhwpSurfaceCacheLookupKey: 'old:0' } }],
  ]);
  const visible = Array.from({ length: count }, (_, i) => i);
  const calls: number[] = [];
  const phases: boolean[] = [];
  const state = { zoom: 0.34, pending: false, animating: false };
  const view = Object.create(prototype) as Record<string, any>;
  const render = (page: number) => {
    calls.push(page);
    host.time += costMs;
    surfaces.set(page, { dataset: {
      rhwpRenderedZoom: String(state.zoom),
      rhwpSurfaceCacheLookupKey: view.pageSurfaceDescriptor(page).lookupKey,
    } });
    return true;
  };
  Object.assign(view, {
    pages: visible.map(() => ({ width: 800, height: 1100 })),
    pageMovement: { direction: 'vertical' },
    viewportManager: {
      isZoomAnimating: () => state.animating, isZoomRasterPending: () => state.pending,
      getZoom: () => state.zoom, getScrollX: () => 0, getScrollY: () => 0,
      getViewportSize: () => ({ width: 1200, height: 700 }),
      finishPendingZoomRaster: () => { state.pending = false; view.renderSettledZoom(); },
    },
    virtualScroll: {
      getVisibilitySnapshot: () => ({ visiblePages: visible, prefetchPages: visible }),
      getPageAtPoint: () => 0,
    },
    canvasPool: {
      get activePages() { return [...surfaces.keys()]; },
      has: (page: number) => surfaces.has(page), getCanvas: (page: number) => surfaces.get(page),
    },
    pageSurfaceLru: { hasLookup: () => false, put: () => true,
      snapshot: () => ({ reservedPixels: 0, pixelBudget: 40_000_000 }) },
    pageRenderScheduler: new PageRenderScheduler(host), renderWorkGeneration: 0,
    currentVisiblePages: visible, currentRetainedPages: visible, editingPageIndex: 0,
    headerFooterEditState: null, disposed: false, pendingPrefetchSurfaceReservations: new Map(),
    updateActivePageSnapshot() {}, reconcilePageSurfaceBudget() {}, renderHeaderFooterEditOverlays() {},
    cancelPendingTextEditRefresh() {}, cancelTextEditStaticLayerVerification() {},
    refreshRenderSurfacePlan: (immediate: boolean) => {
      phases.push(immediate);
      if (immediate) for (const page of surfaces.keys()) render(page);
    },
    pageSurfaceDescriptor: (page: number) => ({ lookupKey: `${state.zoom}:${page}`, estimatedPixelCount: 100 }),
    renderCanvas: render, renderPage: render, applySurfaceDecisionDiagnostics() {},
    releaseAllRenderedPages: () => assert.fail('zoom must not globally release'),
    pageRenderer: { cancelAll: () => assert.fail('exact surface image jobs must survive') },
  });
  return { view, host, surfaces, calls, phases, state, visible };
}

function budgetFixture(budget: number, costs: number[], actual = costs) {
  const f = fixture(costs.length);
  const released: number[] = [];
  f.visible.splice(1);
  f.surfaces.clear();
  costs.forEach((_, page) => f.surfaces.set(page, { dataset: {
    rhwpRenderedZoom: '1', rhwpActualSurfacePixels: String(actual[page]),
    rhwpSurfaceCacheLookupKey: `old:${page}`,
  } }));
  Object.assign(f.view, {
    currentRetainedPages: costs.map((_, i) => i),
    renderSurfacePlan: { retainedPixelBudget: budget, decisions: costs.map((_, pageIndex) => ({ pageIndex })) },
    pageSurfaceLru: new PageSurfaceLru(() => {}, budget),
    reconcilePageSurfaceBudget: prototype.reconcilePageSurfaceBudget,
    pageSurfaceDescriptor: (page: number) => ({ lookupKey: `${f.state.zoom}:${page}`, estimatedPixelCount: costs[page] }),
    detachCompletedPageSurface: (page: number) => ({ pageIndex: page }),
    disposeCachedPageSurface: ({ pageIndex }: { pageIndex: number }) => {
      released.push(pageIndex); f.surfaces.delete(pageIndex);
    },
  });
  f.view.virtualScroll.getVisibilitySnapshot = () => ({ visiblePages: f.visible, prefetchPages: costs.map((_, i) => i) });
  return { ...f, released };
}

test('줌 확대: mandatory visible만 예산을 넘어도 preview는 보존하고 offscreen 확대는 거절한다', () => {
  const f = budgetFixture(40, [100, 60], [10, 6]);
  f.state.zoom = 5;
  const visiblePreview = f.surfaces.get(0);
  f.view.renderSettledZoom();
  assert.deepEqual(f.released, [1]);
  assert.equal(f.surfaces.get(0), visiblePreview);
  while (f.host.frames.size) f.host.runFrame();
  assert.deepEqual(f.calls, [0]);
  assert.equal(f.view.pendingPrefetchSurfaceReservations.size, 0);
});

test('줌 정착: 초과분만 먼 offscreen부터 반환하고 headroom 안 warm surface는 보존한다', () => {
  const f = budgetFixture(40, [20, 10, 10, 10]);
  f.view.renderSettledZoom();
  assert.deepEqual(f.released, [3]);
  assert.deepEqual([...f.surfaces.keys()], [0, 1, 2]);
  assert.equal(f.view.pageSurfaceLru.snapshot().reservedPixels, 40);
});

test('줌 축소: 큰 구 preview를 작은 target 비용으로 오인해 보존하지 않는다', () => {
  const f = budgetFixture(40, [10, 5], [10, 100]);
  f.view.renderSettledZoom();
  assert.deepEqual(f.released, [1]);
  assert.equal(f.view.pendingPrefetchSurfaceReservations.get(1), 5, '작아진 새 surface는 정상 prefetch admission 가능');
});

test('화면 밖 편집 surface 반환은 focus를 옮기지 않고 재진입 strict 렌더를 막지 않는다', () => {
  const f = budgetFixture(40, [100, 60]);
  f.view.editingPageIndex = 1;
  f.view.renderSettledZoom();
  assert.deepEqual(f.released, [1]);
  assert.equal(f.view.editingPageIndex, 1);
  f.visible.splice(0, 1, 1);
  f.view.updateVisiblePages('strict');
  assert.ok(f.surfaces.has(1));
  assert.ok(f.calls.includes(1));
  assert.equal(f.view.editingPageIndex, 1);
});

test('예산 안 exact warm surface는 반복 줌 정착에서도 반환하거나 다시 그리지 않는다', () => {
  const f = budgetFixture(40, [20, 10]);
  for (const [page, canvas] of f.surfaces) {
    canvas.dataset.rhwpRenderedZoom = String(f.state.zoom);
    canvas.dataset.rhwpSurfaceCacheLookupKey = f.view.pageSurfaceDescriptor(page).lookupKey;
  }
  for (let i = 0; i < 3; i++) f.view.renderSettledZoom();
  assert.deepEqual(f.released, []);
  assert.deepEqual(f.calls, []);
  assert.equal(f.host.frames.size, 0);
});

test('미완료 offscreen 반환은 이미지 작업 취소 경로를 거치며 모든 Canvas backing을 비운다', t => {
  const oldCanvas = Object.getOwnPropertyDescriptor(globalThis, 'HTMLCanvasElement');
  class Canvas { width = 100; height = 100; }
  Object.defineProperty(globalThis, 'HTMLCanvasElement', { configurable: true, value: Canvas });
  t.after(() => {
    if (oldCanvas) Object.defineProperty(globalThis, 'HTMLCanvasElement', oldCanvas);
    else Reflect.deleteProperty(globalThis, 'HTMLCanvasElement');
  });
  const f = budgetFixture(40, [100, 60]);
  const elements = [new Canvas(), new Canvas(), { tagName: 'IMG' }];
  const cancelled: number[] = [];
  f.view.detachCompletedPageSurface = () => null;
  f.view.pageSurfaceElements = () => elements;
  f.view.discardActivePageSurface = (page: number) => {
    cancelled.push(page); f.surfaces.delete(page);
  };
  f.view.renderSettledZoom();
  assert.deepEqual(cancelled, [1]);
  for (const element of elements) {
    if (element instanceof Canvas) assert.equal(element.width * element.height, 0);
  }
  assert.ok(f.surfaces.has(0));
});

test('줌 정착의 같은 위치 scroll은 화면 밖 page를 DPR 2→1로 중복 raster하지 않는다', t => {
  const oldWindow = Object.getOwnPropertyDescriptor(globalThis, 'window');
  Object.defineProperty(globalThis, 'window', { configurable: true, value: { devicePixelRatio: 2 } });
  t.after(() => {
    if (oldWindow) Object.defineProperty(globalThis, 'window', oldWindow);
    else Reflect.deleteProperty(globalThis, 'window');
  });
  const f = fixture(2);
  const tasks = new Map<number, { at: number; callback: () => void }>();
  f.host.setTimeout = (callback: () => void, delay: number) => {
    tasks.set(++f.host.id, { at: f.host.time + delay, callback }); return f.host.id;
  };
  f.host.clearTimeout = (id: number) => { tasks.delete(id); };
  const raster: Array<[number, number]> = [];
  Object.assign(f.view, {
    pages: [{ width: 1122.5, height: 1587.4 }, { width: 1122.5, height: 1587.4 }],
    pageRenderer: {
      getBackend: () => 'canvas2d', getRenderProfile: () => 'screen',
      getCanvasSurfaceLayerCount: (page: number) => page === 0 ? 4 : 3,
    },
    activeRendererDecisionKey: 'fixture', renderSurfaceDecisions: new Map(),
    previousEffectiveDpr: new Map(), renderSurfaceEnvironmentKey: null,
    refreshRenderSurfacePlan: prototype.refreshRenderSurfacePlan,
    pageSurfaceDescriptor: prototype.pageSurfaceDescriptor,
    renderCanvas: (page: number) => {
      const dpr = f.view.renderSurfaceDecisions.get(page).effectiveDpr;
      raster.push([page, dpr]);
      f.surfaces.set(page, { dataset: {
        rhwpRequestedDpr: String(dpr), rhwpRenderedZoom: String(f.state.zoom),
        rhwpSurfaceCacheLookupKey: f.view.pageSurfaceDescriptor(page).lookupKey,
      } });
      return true;
    },
  });
  f.state.zoom = 0.5;
  f.view.refreshRenderSurfacePlan(false);
  for (const page of [0, 1]) f.view.renderCanvas(page);
  raster.length = 0;
  f.visible.splice(1);
  f.view.virtualScroll.getVisibilitySnapshot = () => ({ visiblePages: [0], prefetchPages: [0, 1] });
  f.state.zoom = 1;
  f.view.renderSettledZoom();
  f.view.updateVisiblePages('scroll'); // zoom 앵커 setter가 만든 native scroll의 지연 전달
  for (let i = 0; i < 10 && (f.host.frames.size || tasks.size); i++) {
    while (f.host.frames.size) f.host.runFrame();
    const next = [...tasks.entries()].sort((a, b) => a[1].at - b[1].at)[0];
    if (next) { tasks.delete(next[0]); f.host.time = next[1].at; next[1].callback(); }
  }
  // 같은 physical scale이어도 구 zoom preview는 최종 zoom/DPR로 한 번 확정해야 한다.
  assert.deepEqual(raster, [[0, 2], [1, 1]]);
  assert.equal(f.surfaces.get(1)!.dataset.rhwpRenderedZoom, '1');
  assert.equal(f.surfaces.get(1)!.dataset.rhwpRequestedDpr, '1');
  assert.equal(f.view.pageRenderScheduler.snapshot().scrollSettleScheduled, false);
});

test('같은 위치의 반복 지연 scroll은 zoom visible 큐 세대를 교체하지 않는다', () => {
  const f = fixture(2);
  f.view.renderSettledZoom();
  const generation = f.view.renderWorkGeneration;
  for (let i = 0; i < 3; i++) f.view.updateVisiblePages('scroll');
  assert.equal(f.view.renderWorkGeneration, generation);
  assert.deepEqual(f.calls, [], '동기 scroll fast path로 줌 큐를 우회하지 않는다');
  f.host.runFrame();
  assert.equal(f.calls.length, 2);
});

for (const reason of ['zoom-settled', 'scroll', 'scroll-settled', 'strict'] as const) {
  test(`${reason} 예약: 줌 정착만 visible 후 frame 양보를 요청한다`, () => {
    const f = fixture(2);
    const scheduler = f.view.pageRenderScheduler;
    const original = scheduler.setDesiredWork.bind(scheduler);
    const requests: boolean[] = [];
    scheduler.setDesiredWork = (...args: any[]) => {
      requests.push(args[4]?.yieldAfterVisible ?? false);
      return original(...args);
    };
    f.view.updateVisiblePages(reason);
    assert.deepEqual(requests, [reason === 'zoom-settled']);
  });
}

test('physical key가 같아도 visible의 구 zoom preview는 최신 배율로 한 번 확정한다', () => {
  const f = fixture(1);
  f.surfaces.get(0)!.dataset.rhwpSurfaceCacheLookupKey = f.view.pageSurfaceDescriptor(0).lookupKey;
  f.view.renderSettledZoom();
  assert.deepEqual(f.calls, []);
  f.host.runFrame();
  assert.deepEqual(f.calls, [0]);
  assert.equal(f.surfaces.get(0)!.dataset.rhwpRenderedZoom, '0.34');
  f.view.renderSettledZoom();
  assert.equal(f.host.frames.size, 0);
});

for (const changed of ['x', 'y', 'width', 'height', 'zoom', 'strict', 'generation'] as const) {
  test(`줌 정착 뒤 ${changed} 변경은 같은 위치 scroll 제거에 가려지지 않는다`, () => {
    const f = fixture(2);
    f.view.renderSettledZoom();
    const vm = f.view.viewportManager;
    if (changed === 'x') vm.getScrollX = () => 0.5;
    if (changed === 'y') vm.getScrollY = () => 0.5;
    if (changed === 'width') vm.getViewportSize = () => ({ width: 1199, height: 700 });
    if (changed === 'height') vm.getViewportSize = () => ({ width: 1200, height: 699 });
    if (changed === 'zoom') f.state.zoom = 0.5;
    if (changed === 'strict') f.view.updateVisiblePages('strict');
    if (changed === 'generation') f.view.cancelPendingPrefetch();
    const generation = f.view.renderWorkGeneration;
    f.view.updateVisiblePages('scroll');
    assert.equal(f.view.renderWorkGeneration, generation + 1);
  });
}

test('줌 계획 도중 실패하면 같은 위치의 scroll 복구를 막지 않는다', () => {
  const f = fixture(2);
  const refresh = f.view.refreshRenderSurfacePlan;
  f.view.refreshRenderSurfacePlan = () => { throw new Error('plan failed'); };
  assert.throws(() => f.view.renderSettledZoom(), /plan failed/);
  f.view.refreshRenderSurfacePlan = refresh;
  const generation = f.view.renderWorkGeneration;
  f.view.updateVisiblePages('scroll');
  assert.equal(f.view.renderWorkGeneration, generation + 1);
  assert.equal(f.calls.length, 2);
});

for (const count of [1, 2, 8]) {
  test(`zoom ${count}쪽: 동기 fast path 없이 기존 surface를 유지하고 새 visible부터 분할한다`, () => {
    const f = fixture(count);
    const old = f.surfaces.get(0);
    f.view.renderSettledZoom();
    assert.deepEqual(f.calls, []);
    assert.deepEqual(f.phases, [false]);
    assert.equal(f.surfaces.get(0), old);
    assert.equal(f.view.pageRenderScheduler.snapshot().visibleQueued, count);
    f.host.runFrame();
    assert.equal(f.calls.length, Math.min(count, 2));
    if (count > 2) {
      assert.deepEqual(f.calls, [1, 2], '빈 쪽부터 그리며 기존 focus preview는 지우지 않는다');
      assert.equal(f.surfaces.get(0), old);
    }
    while (f.host.frames.size) f.host.runFrame();
    assert.equal(f.calls.length, count);
    for (const page of f.visible) {
      assert.equal(f.surfaces.get(page)?.dataset.rhwpSurfaceCacheLookupKey, f.view.pageSurfaceDescriptor(page).lookupKey);
    }
    f.view.renderSettledZoom();
    assert.equal(f.host.frames.size, 0, '같은 배율의 exact 결과는 다시 그리지 않는다');
    assert.equal(f.calls.length, count);
  });
}

test('zoom 한 쪽이 soft budget을 넘으면 다음 쪽 전에 frame을 양보한다', () => {
  const f = fixture(8, 20);
  f.view.renderSettledZoom();
  f.host.runFrame();
  assert.deepEqual(f.calls, [1]);
  assert.equal(f.view.pageRenderScheduler.snapshot().visibleQueued, 7);
});

test('재입력은 이전 zoom 큐를 취소하고 다음 정착은 최신 key만 완료한다', () => {
  const f = fixture();
  f.view.renderSettledZoom();
  f.host.runFrame();
  const stale = f.view.buildVisibleRenderWork(f.visible, 0, f.view.renderWorkGeneration);
  const oldSurface = f.surfaces.get(0);
  f.view.cancelPendingPrefetch();
  assert.equal(f.host.frames.size, 0);
  assert(stale.every((work: any) => !work.isValid()));
  f.state.zoom = 0.75;
  f.view.renderSettledZoom();
  assert.equal(f.surfaces.get(0), oldSurface);
  while (f.host.frames.size) f.host.runFrame();
  for (const page of f.visible) assert.equal(f.surfaces.get(page)?.dataset.rhwpRenderedZoom, '0.75');
});

test('strict는 quiet ready가 비동기 예약으로 바뀌어도 최신 visible을 응답 전에 완료한다', () => {
  const f = fixture();
  f.state.pending = true;
  f.view.updateVisiblePages('strict');
  assert.deepEqual(f.phases, [false, true]);
  assert.equal(f.surfaces.size, 8);
  assert.equal(f.host.frames.size, 0);
  for (const canvas of f.surfaces.values()) assert.equal(canvas.dataset.rhwpRenderedZoom, '0.34');
});

test('실제 휠 재입력 뒤 첫 animation보다 먼저 도착한 구 render frame은 raster/decode를 시작하지 않는다', async t => {
  const f = fixture(3);
  const oldRequest = globalThis.requestAnimationFrame;
  const oldCancel = globalThis.cancelAnimationFrame;
  globalThis.requestAnimationFrame = callback => f.host.requestFrame(() => callback(f.host.time));
  globalThis.cancelAnimationFrame = id => f.host.cancelFrame(id);
  const timers = new Map<number, { at: number; callback: () => void }>();
  const bus = new EventBus();
  const vm = new ViewportManager(bus, {
    now: () => f.host.time,
    schedule: (callback, delay) => {
      const id = ++f.host.id;
      timers.set(id, { at: f.host.time + delay, callback });
      return id as unknown as ReturnType<typeof setTimeout>;
    },
    cancel: id => { timers.delete(id as unknown as number); },
  });
  const renderer = new Renderer({});
  let decodeStarted = 0;
  renderer.buildImageRetryKey = () => null;
  renderer.prefetchLayerImages = async () => { decodeStarted++; return true; };
  renderer.reRenderPageCanvases = () => {};
  t.after(() => {
    vm.cancelPendingZoomRaster();
    f.view.pageRenderScheduler.cancelAll();
    renderer.cancelAll();
    globalThis.requestAnimationFrame = oldRequest;
    globalThis.cancelAnimationFrame = oldCancel;
  });
  vm.setZoom(f.state.zoom);
  f.view.viewportManager = vm;
  let geometryEvents = 0;
  // 실제 이벤트 순서만 연결한다. geometry 자체의 계산은 preview 전용 테스트가 검증한다.
  bus.on('zoom-changed', () => {
    geometryEvents++;
    f.state.zoom = vm.getZoom();
    f.view.cancelPendingPrefetch();
    if (!vm.isZoomAnimating() && !vm.isZoomRasterPending()) f.view.renderSettledZoom();
  });
  bus.on('zoom-raster-ready', generation => f.view.onZoomRasterReady(generation));
  for (const method of ['renderPage', 'renderCanvas']) {
    const render = f.view[method];
    f.view[method] = (page: number) => {
      const result = render(page);
      renderer.scheduleReRender(page, { parentElement: {} }, vm.getZoom() * 2, 1, 0, {});
      return result;
    };
  }
  const oldSurface = f.surfaces.get(0);
  f.view.renderSettledZoom();
  const generation = f.view.renderWorkGeneration;
  (vm as any).onWheel({
    ctrlKey: true, metaKey: false, deltaX: 0, deltaY: 40, deltaMode: 0,
    clientX: 0, clientY: 0, preventDefault() {},
  });
  assert.equal(vm.isZoomAnimating(), true);
  assert.equal(vm.isZoomRasterPending(), true);
  assert.equal(vm.getZoom(), 0.34);
  assert.equal(geometryEvents, 0);
  assert.equal(f.view.renderWorkGeneration, generation, '첫 geometry 전에는 key와 작업 세대가 그대로다');
  f.host.runFrame(); // 구 render frame이 새 animation frame보다 먼저 예약되어 있다.
  await Promise.resolve();
  assert.deepEqual(f.calls, []);
  assert.equal(decodeStarted, 0, '거부한 main의 후속 decode도 생성하지 않는다');
  assert.equal(f.surfaces.get(0), oldSurface);
  assert.equal(f.view.pageRenderScheduler.snapshot().staleDropped, 3);
  assert.equal(f.host.frames.size, 1, '구 큐의 busy loop 없이 실제 animation만 남는다');
  for (let step = 0; step < 100 && (f.host.frames.size || timers.size); step++) {
    f.host.time += 16;
    if (f.host.frames.size) f.host.runFrame();
    for (const [id, timer] of timers) {
      if (timer.at <= f.host.time) { timers.delete(id); timer.callback(); }
    }
  }
  await Promise.resolve();
  await Promise.resolve();
  assert.equal(vm.isZoomRasterPending(), false);
  assert.equal(vm.isZoomAnimating(), false);
  assert.equal(f.host.frames.size, 0);
  assert.equal(timers.size, 0);
  assert.equal(f.calls.length, 3);
  assert.equal(decodeStarted, 3, '최신 정착의 정상 후처리는 유지한다');
  for (const page of f.visible) assert.equal(f.surfaces.get(page)?.dataset.rhwpRenderedZoom, String(vm.getZoom()));
});

for (const phase of ['pending', 'animating'] as const) {
  for (const workClass of ['visible', 'retained-transition', 'prefetch'] as const) {
    test(`${phase} 동안 ${workClass} dispatch를 거부하고 prefetch 예약을 반환한다`, () => {
      const f = fixture(1);
      f.view.pendingPrefetchSurfaceReservations.set(0, 100);
      f.view.hasValidPrefetchSurfaceReservation = () => true;
      const work = f.view.createPageRenderWork(0, '0.34:0', 0, workClass, f.view.renderWorkGeneration, 100);
      assert.equal(work.isValid(), true);
      f.state[phase] = true;
      assert.equal(work.isValid(), false, 'geometry/key가 그대로여도 입력 또는 animation 대기면 차단한다');
      if (workClass === 'prefetch') assert.equal(f.view.pendingPrefetchSurfaceReservations.size, 0);
      assert.equal(f.surfaces.size, 1);
      assert.deepEqual(f.calls, []);
    });
  }
}

test('구 zoom bitmap은 실제 비용과 target 중 큰 쪽을 예약하고 exact에서는 실제 비용으로 복귀한다', () => {
  const f = fixture(1);
  const ledger: number[] = [];
  f.view.pageSurfaceLru.reconcile = (_budget: number, reserved: number) => ledger.push(reserved);
  f.view.renderSurfacePlan = { retainedPixelBudget: 40_000_000, decisions: [{ pageIndex: 0 }] };
  f.surfaces.get(0)!.dataset.rhwpActualSurfacePixels = '1000';
  prototype.reconcilePageSurfaceBudget.call(f.view);
  assert.equal(ledger.at(-1), 1000, '축소 target 100으로 실제 1000을 숨기지 않는다');
  f.surfaces.get(0)!.dataset.rhwpActualSurfacePixels = '50';
  prototype.reconcilePageSurfaceBudget.call(f.view);
  assert.equal(ledger.at(-1), 100, '확대 target도 미리 예약한다');
  Object.assign(f.surfaces.get(0)!.dataset, {
    rhwpRenderedZoom: '0.34', rhwpSurfaceCacheLookupKey: '0.34:0',
  });
  prototype.reconcilePageSurfaceBudget.call(f.view);
  assert.equal(ledger.at(-1), 50);
});

test('같은 DPR이어도 strict plan은 queue에 남은 구 zoom surface를 다시 그린다', () => {
  const oldWindow = Object.getOwnPropertyDescriptor(globalThis, 'window');
  try {
    Object.defineProperty(globalThis, 'window', { configurable: true, value: { devicePixelRatio: 2 } });
    const f = fixture(1);
    Object.assign(f.view, {
      renderSurfaceDecisions: new Map(), previousEffectiveDpr: new Map(),
      activePageSnapshot: { pageIndex: 0 }, renderSurfaceEnvironmentKey: null,
      pageRenderer: { getBackend: () => 'canvas2d', getRenderProfile: () => 'screen', getCanvasSurfaceLayerCount: () => 1 },
    });
    prototype.refreshRenderSurfacePlan.call(f.view, false);
    prototype.refreshRenderSurfacePlan.call(f.view, true);
    assert.deepEqual(f.calls, [0]);
    prototype.refreshRenderSurfacePlan.call(f.view, true);
    assert.deepEqual(f.calls, [0], '완료한 같은 zoom/DPR에는 불필요한 raster가 없다');
  } finally {
    if (oldWindow) Object.defineProperty(globalThis, 'window', oldWindow);
    else Reflect.deleteProperty(globalThis, 'window');
  }
});

test('같은 그림의 pending decode는 새 bitmap/scale에 재연결하고 오래된 완료를 무시한다', async () => {
  const renderer = new Renderer({});
  const completions: Array<(value: boolean) => void> = [];
  const repaints: unknown[][] = [];
  renderer.buildImageRetryKey = () => 'same-document-and-images';
  renderer.prefetchLayerImages = () => new Promise<boolean>(resolve => completions.push(resolve));
  renderer.reRenderPageCanvases = (...args: unknown[]) => repaints.push(args);
  const oldCanvas = { parentElement: {} };
  const newCanvas = { parentElement: {} };
  try {
    renderer.scheduleReRender(0, oldCanvas, 2, 1, 0, {});
    await Promise.resolve();
    renderer.scheduleReRender(0, newCanvas, 0.68, 1, 0, {});
    await Promise.resolve();
    assert.equal(completions.length, 2);
    completions[0](true);
    await Promise.resolve();
    assert.deepEqual(repaints, []);
    completions[1](true);
    await Promise.resolve();
    assert.equal(repaints.length, 1);
    assert.equal(repaints[0][1], newCanvas);
    assert.equal(repaints[0][2], 0.68);
    renderer.scheduleReRender(0, newCanvas, 1, 1, 0, {});
    await Promise.resolve();
    assert.equal(completions.length, 2, '이미 완료된 동일 그림은 다시 prefetch하지 않는다');
  } finally { renderer.cancelAll(); }
});

for (const cancel of ['page', 'all'] as const) {
  test(`${cancel} 취소는 미완료 그림의 재시도를 막지 않고 완료된 다른 그림은 재사용한다`, async () => {
    const renderer = new Renderer({});
    const completions: Array<(value: boolean) => void> = [];
    const repaints: unknown[][] = [];
    renderer.buildImageRetryKey = () => 'same-images';
    renderer.prefetchLayerImages = () => new Promise<boolean>(resolve => completions.push(resolve));
    renderer.reRenderPageCanvases = (...args: unknown[]) => repaints.push(args);
    const oldCanvas = { parentElement: {} };
    const newCanvas = { parentElement: {} };
    try {
      renderer.scheduleReRender(1, oldCanvas, 2, 1, 0, {});
      await Promise.resolve();
      completions[0](true);
      await Promise.resolve();
      renderer.scheduleReRender(0, oldCanvas, 2, 1, 0, {});
      await Promise.resolve();
      if (cancel === 'page') renderer.cancelReRender(0);
      else renderer.cancelAll();
      completions[1](true);
      await Promise.resolve();
      assert.equal(repaints.length, 1, '취소된 완료는 기존/새 canvas 어느 쪽도 덮지 않는다');
      renderer.scheduleReRender(0, newCanvas, 0.68, 1, 0, {});
      await Promise.resolve();
      assert.equal(completions.length, 3, '취소를 완료로 취급해 재시도를 생략하면 안 된다');
      completions[2](true);
      await Promise.resolve();
      assert.equal(repaints.length, 2);
      assert.equal(repaints[1][1], newCanvas);
      assert.equal(repaints[1][2], 0.68);
      assert.equal(renderer.reRenderJobs.size, 0);
      renderer.scheduleReRender(1, newCanvas, 0.68, 1, 0, {});
      await Promise.resolve();
      assert.equal(completions.length, 3, '완료된 다른 페이지의 decode 재사용은 유지한다');
    } finally { renderer.cancelAll(); }
  });
}

for (const boundary of ['page', 'all', 'revision', 'backend', 'dispose'] as const) {
  test(`${boundary}: microtask 전 취소한 이미지 작업은 WASM 조회조차 시작하지 않는다`, async () => {
    const queries: string[] = [];
    const renderer = new Renderer({
      documentDigest: 'document-a', documentGeneration: 1,
      getPageSourceImageKeys: () => { queries.push('keys'); return null; },
      getPageLayerTree: () => { queries.push('tree'); return '{}'; },
    });
    renderer.buildImageRetryKey = () => null;
    renderer.reRenderPageCanvases = () => assert.fail('취소된 repaint');
    try {
      renderer.scheduleReRender(0, { parentElement: {} }, 2, 1, 0, {});
      if (boundary === 'page') renderer.cancelReRender(0);
      if (boundary === 'all') renderer.cancelAll();
      if (boundary === 'revision') renderer.invalidateDocumentRevision();
      if (boundary === 'backend') renderer.configure('canvaskit', 'screen', null);
      if (boundary === 'dispose') renderer.dispose();
      await Promise.resolve();
      await Promise.resolve();
      assert.deepEqual(queries, []);
      assert.equal(renderer.reRenderJobs.size, 0);
      assert.equal(renderer.prefetchRequestTokens.size, 0);
    } finally { renderer.cancelAll(); }
  });
}

test('동일 surface에 연속 예약한 20개 배율 중 최신 이미지 작업만 시작한다', async () => {
  const renderer = new Renderer({});
  const tokens: number[] = [];
  const repaints: unknown[][] = [];
  renderer.buildImageRetryKey = () => 'same-images';
  renderer.prefetchLayerImages = async (_page: number, _svg: number, token: number) => {
    tokens.push(token);
    return true;
  };
  renderer.reRenderPageCanvases = (...args: unknown[]) => repaints.push(args);
  const reusedCanvas = { parentElement: {} };
  try {
    for (let i = 1; i <= 20; i++) renderer.scheduleReRender(0, reusedCanvas, i / 10, 1, 0, {});
    await Promise.resolve();
    await Promise.resolve();
    assert.equal(tokens.length, 1);
    assert.equal(repaints.length, 1);
    assert.equal(repaints[0][1], reusedCanvas);
    assert.equal(repaints[0][2], 2);
    assert.equal(renderer.reRenderJobs.size, 0);
  } finally { renderer.cancelAll(); }
});

test('반복 역방향 줌은 부분 완료 surface를 유지하며 마지막 배율로 수렴한다', () => {
  const f = fixture(8, 20);
  for (const zoom of [0.34, 2, 0.5, 5, 0.34, 1]) {
    f.state.zoom = zoom;
    f.view.renderSettledZoom();
    f.host.runFrame();
    const surfaces = [...f.surfaces.entries()];
    f.view.cancelPendingPrefetch();
    assert.deepEqual([...f.surfaces.entries()], surfaces);
    assert.equal(f.host.frames.size, 0);
    assert.equal(f.view.pendingPrefetchSurfaceReservations.size, 0);
  }
  f.view.renderSettledZoom();
  while (f.host.frames.size) f.host.runFrame();
  for (const page of f.visible) assert.equal(f.surfaces.get(page)?.dataset.rhwpRenderedZoom, '1');
  const calls = f.calls.length;
  f.view.updateVisiblePages('strict');
  assert.equal(f.host.frames.size, 0);
  assert.equal(f.surfaces.size, 8);
  assert(f.calls.length >= calls);
});

test('문서 교체는 구 zoom 큐·surface·미시작 이미지 작업을 버리고 같은 page 번호를 재사용한다', async () => {
  const f = fixture(3);
  const wasm = { documentDigest: 'a', documentGeneration: 1 };
  const renderer = new Renderer(wasm);
  let decodeStarted = 0;
  renderer.buildImageRetryKey = () => null;
  renderer.prefetchLayerImages = async () => { decodeStarted++; return true; };
  renderer.reRenderPageCanvases = () => assert.fail('구 문서 repaint');
  Object.assign(f.view, {
    wasm, pageRenderer: renderer, rendererSelectionEpoch: 0,
    activePageSnapshot: { pageIndex: 0 }, previousEffectiveDpr: new Map(),
    renderSurfaceDecisions: new Map(),
    rendererSession: { beginDocument: (digest: string) => assert.equal(digest, 'b') },
    cancelAutoRendererReselection() {}, removeHeaderFooterEditOverlays() {}, removeAllGridOverlays() {},
    scrollContent: { querySelectorAll: () => [], replaceChildren() {} },
    showBlankPagePlaceholder() {}, eventBus: { emit() {} },
    releaseAllRenderedPages: prototype.releaseAllRenderedPages,
    pageSurfaceDescriptor: (page: number) => ({ lookupKey: `${wasm.documentDigest}:${f.state.zoom}:${page}`, estimatedPixelCount: 100 }),
  });
  f.view.viewportManager.cancelPendingZoomRaster = () => {
    const pending = f.state.pending;
    f.state.pending = false;
    return pending;
  };
  f.view.virtualScroll.reset = () => {};
  f.view.canvasPool.releaseAll = () => f.surfaces.clear();
  f.view.pageSurfaceLru.clear = () => {};
  try {
    f.view.renderSettledZoom();
    const oldWork = f.view.buildVisibleRenderWork(f.visible, 0, f.view.renderWorkGeneration);
    renderer.scheduleReRender(0, { parentElement: {} }, 2, 1, 0, {});
    f.state.pending = true;
    wasm.documentDigest = 'b';
    wasm.documentGeneration++;
    f.view.prepareDocumentLoad();
    await Promise.resolve();
    assert.equal(f.state.pending, false);
    assert.equal(f.host.frames.size, 0);
    assert.equal(f.surfaces.size, 0);
    assert.equal(f.view.pages.length, 0);
    assert.equal(decodeStarted, 0);
    assert(oldWork.every((work: any) => !work.isValid()));
    f.view.pages = f.visible.map(() => ({ width: 800, height: 1100 }));
    f.view.renderSettledZoom();
    while (f.host.frames.size) f.host.runFrame();
    assert.equal(f.surfaces.size, 3);
    for (const canvas of f.surfaces.values()) assert.match(canvas.dataset.rhwpSurfaceCacheLookupKey, /^b:/);
  } finally { renderer.cancelAll(); f.view.pageRenderScheduler.cancelAll(); }
});

for (const completion of ['decode', 'fallback', 'reject'] as const) {
  test(`${completion}: 구 callback은 새 job의 timer·surface를 건드리지 않고 최신 fallback은 살아 있다`, async t => {
    let nextId = 0;
    const timers = new Map<number, () => void>();
    t.mock.method(globalThis, 'setTimeout', (callback: () => void) => {
      timers.set(++nextId, callback);
      return nextId;
    });
    t.mock.method(globalThis, 'clearTimeout', (id: number) => timers.delete(id));
    const renderer = new Renderer({});
    const decodes: Array<{ resolve(value: boolean): void; reject(error: Error): void }> = [];
    const repaints: unknown[][] = [];
    renderer.buildImageRetryKey = () => null;
    renderer.prefetchLayerImages = () => new Promise<boolean>((resolve, reject) => decodes.push({ resolve, reject }));
    renderer.reRenderPageCanvases = (...args: unknown[]) => repaints.push(args);
    const canvas = { parentElement: {} };
    try {
      renderer.scheduleReRender(0, canvas, 2, 1, 0, {});
      await Promise.resolve();
      const oldFallback = timers.values().next().value!;
      renderer.scheduleReRender(0, canvas, 0.68, 1, 0, {});
      await Promise.resolve();
      const latestJob = renderer.reRenderJobs.get(0);
      if (completion === 'decode') decodes[0].resolve(true);
      if (completion === 'fallback') oldFallback(); // 이미 가져간 구 callback의 지연 실행도 무해하다.
      if (completion === 'reject') decodes[0].reject(new Error('old decode failure'));
      await Promise.resolve();
      await Promise.resolve();
      assert.deepEqual(repaints, []);
      assert.equal(renderer.reRenderJobs.get(0), latestJob);
      assert.equal(timers.size, 1);
      decodes[1].resolve(false);
      await Promise.resolve();
      timers.values().next().value!();
      assert.equal(repaints.length, 1);
      assert.equal(repaints[0][1], canvas);
      assert.equal(repaints[0][2], 0.68);
      assert.equal(timers.size, 0);
      assert.equal(renderer.reRenderJobs.size, 0);
    } finally { renderer.cancelAll(); }
  });
}
