import test from 'node:test';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
import { createServer } from 'vite';

class FakeElement {
  parentElement: FakeParent | null = null;
  dataset: Record<string, string> = {};
  style: Record<string, string> = {};
  classList = { add: () => undefined };
  tagName = 'DIV';

  remove(): void {
    this.parentElement?.removeChild(this);
  }

  removeAttribute(name: string): void {
    if (name === 'style') this.style = {};
  }
}

class FakeCanvas extends FakeElement {
  override tagName = 'CANVAS';
  width = 0;
  height = 0;
}

class FakeParent extends FakeElement {
  children: FakeElement[] = [];

  appendChild<T extends FakeElement>(element: T): T {
    element.parentElement?.removeChild(element);
    this.children.push(element);
    element.parentElement = this;
    return element;
  }

  removeChild<T extends FakeElement>(element: T): T {
    const index = this.children.indexOf(element);
    assert.ok(index >= 0, '현재 parent가 소유한 element만 제거해야 한다');
    this.children.splice(index, 1);
    element.parentElement = null;
    return element;
  }

  querySelector(): FakeElement | null {
    return null;
  }
}

test('CanvasView는 fractional DPR 재래스터 뒤에도 page surface CSS 크기를 고정한다', async () => {
  const studioRoot = fileURLToPath(new URL('..', import.meta.url));
  const vite = await createServer({
    root: studioRoot,
    appType: 'custom',
    logLevel: 'silent',
    server: { middlewareMode: true },
  });
  try {
    const { CanvasView } = await vite.ssrLoadModule('/src/view/canvas-view.ts');
    const main = new FakeCanvas();
    const overlay = new FakeCanvas();
    const view = Object.create(CanvasView.prototype) as Record<string, any>;
    view.pages = [{ width: 1123, height: 1587.5 }];
    view.pageSurfaceElements = () => [main, overlay];

    view.applyPageSurfaceCssSize(0, main, 2);

    assert.equal(main.style.width, '2246px');
    assert.equal(main.style.height, '3175px');
    assert.equal(overlay.style.width, '2246px');
    assert.equal(overlay.style.height, '3175px');
  } finally {
    await vite.close();
  }
});

test('CanvasView는 장치 표본을 보존하고 축소 예산·줌 미리보기·DOM 그림은 다시 보간한다', async () => {
  const studioRoot = fileURLToPath(new URL('..', import.meta.url));
  const vite = await createServer({
    root: studioRoot,
    appType: 'custom',
    logLevel: 'silent',
    server: { middlewareMode: true },
  });
  const dprDescriptor = Object.getOwnPropertyDescriptor(globalThis, 'devicePixelRatio');
  try {
    Object.defineProperty(globalThis, 'devicePixelRatio', { configurable: true, value: 1.5 });
    const { CanvasView } = await vite.ssrLoadModule('/src/view/canvas-view.ts');
    const main = new FakeCanvas();
    const overlay = new FakeCanvas();
    const domImagePlane = new FakeElement();
    domImagePlane.style.imageRendering = 'smooth';
    const view = Object.create(CanvasView.prototype) as Record<string, any>;
    view.pages = [{ width: 793.7, height: 1122.5 }];
    view.pageSurfaceElements = () => [main, overlay, domImagePlane];
    view.viewportManager = { getZoom: () => 1 };
    view.virtualScroll = { getPageOffset: () => 140, getPageLeft: () => -1 };

    // backend별 정수 축 반올림이 달라도 같은 장치 표본을 다시 흐리게 만들지 않는다.
    main.width = 1191; main.height = 1684;
    overlay.width = 1190; overlay.height = 1683;
    view.applyPageSurfaceCssSize(0, main, 1);
    assert.equal(main.style.imageRendering, 'crisp-edges');
    assert.equal(overlay.style.imageRendering, 'crisp-edges');
    assert.equal(domImagePlane.style.imageRendering, 'smooth', '원본 DOM 그림의 필터는 덮어쓰지 않는다');
    for (const element of [main, overlay, domImagePlane]) {
      assert.equal(element.style.width, '793.7px');
      assert.equal(element.style.height, '1122.5px');
    }

    // 메모리 정책이 낮은 해상도를 골랐으면 필요한 표시 보간을 유지한다.
    main.width = 794; main.height = 1122;
    overlay.width = 794; overlay.height = 1122;
    view.applyPageSurfaceCssSize(0, main, 1);
    assert.equal(main.style.imageRendering, 'auto');
    assert.equal(overlay.style.imageRendering, 'auto');

    main.width = 1191; main.height = 1684;
    view.applyPageSurfaceCssSize(0, main, 1);
    view.applyZoomPreviewBox(main, 0, 1.25);
    assert.equal(main.style.imageRendering, 'auto', '이전 표면을 확대하는 미리보기는 보간한다');
    view.positionPageElement(main, 0);
    assert.equal(main.style.imageRendering, 'crisp-edges', '원 배율의 캐시 표면 복원 시 표본을 보존한다');

    Object.defineProperty(globalThis, 'devicePixelRatio', { configurable: true, value: 1 });
    view.positionPageElement(main, 0);
    assert.equal(main.style.imageRendering, 'auto', '고해상도 캐시를 낮은 DPR로 표시할 때도 보간한다');
    assert.equal(main.style.width, '793.7px');
    assert.equal(main.style.height, '1122.5px');
  } finally {
    if (dprDescriptor) Object.defineProperty(globalThis, 'devicePixelRatio', dprDescriptor);
    else Reflect.deleteProperty(globalThis, 'devicePixelRatio');
    await vite.close();
  }
});

test('CanvasView는 완성 다층 bundle을 exact key로 재부착하고 raster를 반복하지 않는다', async () => {
  const studioRoot = fileURLToPath(new URL('..', import.meta.url));
  const vite = await createServer({
    root: studioRoot,
    appType: 'custom',
    logLevel: 'silent',
    server: { middlewareMode: true },
  });
  const descriptors = new Map<string, PropertyDescriptor | undefined>();
  for (const name of ['HTMLElement', 'HTMLCanvasElement', 'document']) {
    descriptors.set(name, Object.getOwnPropertyDescriptor(globalThis, name));
  }

  try {
    Object.defineProperty(globalThis, 'HTMLElement', { configurable: true, value: FakeElement });
    Object.defineProperty(globalThis, 'HTMLCanvasElement', { configurable: true, value: FakeCanvas });
    Object.defineProperty(globalThis, 'document', {
      configurable: true,
      value: { createElement: () => new FakeCanvas() },
    });

    const { CanvasView } = await vite.ssrLoadModule('/src/view/canvas-view.ts');
    const { CanvasPool } = await vite.ssrLoadModule('/src/view/canvas-pool.ts');
    const { PageSurfaceLru } = await vite.ssrLoadModule('/src/view/page-surface-lru.ts');
    const parent = new FakeParent();
    const canvasPool = new CanvasPool();
    let complete = true;
    let rasterCount = 0;
    const pageRenderer = {
      getBackend: () => 'canvas2d',
      getRenderProfile: () => 'screen',
      isPageSurfaceComplete: () => complete,
      detachPageSurfaceElements: (
        owner: FakeParent,
        pageIdx: number,
        main: FakeCanvas,
      ) => {
        const page = String(pageIdx);
        const elements = owner.children.filter(element => (
          element === main || element.dataset.rhwpOverlayPage === page
        ));
        for (const element of elements) element.remove();
        return elements;
      },
      attachPageSurfaceElements: (owner: FakeParent, elements: FakeElement[]) => {
        for (const element of elements) owner.appendChild(element);
      },
      cancelReRender: () => undefined,
      removePageLayers: () => undefined,
      releasePageDiagnostics: () => undefined,
    };
    const view = Object.create(CanvasView.prototype) as Record<string, any>;
    view.pages = [{ width: 100, height: 200 }];
    view.renderSurfaceDecisions = new Map([[0, {
      pageIndex: 0,
      layerCount: 2,
      visible: true,
      focused: false,
      effectiveDpr: 1,
      tier: 'screen',
      surfacePixels: 40_000,
      surfaceBytes: 160_000,
    }]]);
    view.renderSurfacePlan = {
      retainedPixelBudget: 100_000,
      retainedSurfacePixels: 40_000,
      visibleSurfacePixels: 40_000,
      withinBudget: true,
      decisions: [...view.renderSurfaceDecisions.values()],
    };
    view.viewportManager = { getZoom: () => 1 };
    view.wasm = { documentDigest: 'blake3:test', documentGeneration: 7 };
    view.activeRendererDecisionKey = 'blake3:test|revision:3|profile:screen|resources:2';
    view.pageRenderer = pageRenderer;
    view.canvasPool = canvasPool;
    view.scrollContent = parent;
    view.nextSurfaceLeaseId = 0;
    view.removeGridOverlay = () => undefined;
    view.renderGridOverlay = () => undefined;
    view.positionPageElement = () => undefined;
    view.applySurfaceDecisionDiagnostics = () => undefined;
    view.pageSurfaceLru = new PageSurfaceLru(
      (bundle: unknown) => view.disposeCachedPageSurface(bundle),
      100_000,
    );
    view.renderCanvas = (_pageIdx: number, canvas: FakeCanvas) => {
      rasterCount += 1;
      canvas.width = 100;
      canvas.height = 200;
      return true;
    };

    const main = canvasPool.acquire(0) as FakeCanvas;
    main.width = 100;
    main.height = 200;
    parent.appendChild(main);
    const overlay = new FakeCanvas();
    overlay.width = 100;
    overlay.height = 200;
    overlay.dataset.rhwpOverlayPage = '0';
    parent.appendChild(overlay);
    const descriptor = view.pageSurfaceDescriptor(0);
    main.dataset.rhwpSurfaceCacheLookupKey = descriptor.lookupKey;
    main.dataset.rhwpSurfaceCacheKey = `${descriptor.lookupKey}|surfaces:main:100x200,canvas:100x200`;
    main.dataset.rhwpActualSurfacePixels = '40000';

    const bundle = view.detachCompletedPageSurface(0);
    assert.ok(bundle);
    assert.equal(bundle.pixelCount, 40_000, 'main과 overlay 실제 physical pixel을 합산한다');
    assert.equal(canvasPool.has(0), false);
    assert.deepEqual(parent.children, []);
    assert.equal(view.pageSurfaceLru.put(bundle), true);

    view.renderPage(0);
    assert.equal(rasterCount, 0, 'warm exact-key hit는 page raster를 호출하지 않는다');
    assert.equal(canvasPool.getCanvas(0), main);
    assert.deepEqual(parent.children, [main, overlay], '원래 bundle DOM 순서를 복원한다');
    assert.equal(view.pageSurfaceLru.snapshot().hits, 1);

    const actualLookupKey = main.dataset.rhwpSurfaceCacheLookupKey;
    view.renderSurfaceDecisions.set(0, {
      ...view.renderSurfaceDecisions.get(0),
      effectiveDpr: 2,
    });
    const secondBundle = view.detachCompletedPageSurface(0);
    assert.ok(secondBundle);
    assert.equal(
      secondBundle.lookupKey,
      actualLookupKey,
      '대기 target DPR과 달라도 완성된 실제 surface exact key로 보존한다',
    );
    view.renderSurfaceDecisions.set(0, {
      ...view.renderSurfaceDecisions.get(0),
      effectiveDpr: 1,
    });
    view.pageSurfaceLru.put(secondBundle);
    view.activeRendererDecisionKey = 'blake3:test|revision:4|profile:screen|resources:2';
    view.renderPage(0);
    assert.equal(rasterCount, 1, 'revision이 다르면 stale bundle을 폐기하고 cold render한다');
    assert.equal(view.pageSurfaceLru.snapshot().invalidations, 1);

    complete = false;
    assert.equal(view.detachCompletedPageSurface(0), null, '미완성 비동기 surface는 hit 후보가 아니다');
  } finally {
    for (const [name, descriptor] of descriptors) {
      if (descriptor) Object.defineProperty(globalThis, name, descriptor);
      else Reflect.deleteProperty(globalThis, name);
    }
    await vite.close();
  }
});

test('surface plan 재래스터가 실패한 쪽은 active surface를 놓아 다음 갱신이 다시 그린다', async () => {
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
    const surfaces = [new FakeCanvas(), new FakeCanvas()];
    const rendered: number[] = [];
    const discarded: number[] = [];
    const view = Object.create(CanvasView.prototype) as Record<string, any>;
    Object.assign(view, {
      pages: Array.from({ length: 2 }, () => ({ width: 1000, height: 1000 })),
      currentVisiblePages: [0, 1],
      currentRetainedPages: [0, 1],
      editingPageIndex: null,
      activePageSnapshot: { pageIndex: 0, source: 'viewport' },
      renderSurfacePlan: null,
      // 이전 결정이 없으면 두 쪽 모두 실제 bitmap scale이 바뀐 재래스터 대상이 된다.
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
      pageSurfaceDescriptor: () => null,
      reconcilePageSurfaceBudget: () => undefined,
      applySurfaceDecisionDiagnostics: () => undefined,
      renderCanvas: (page: number) => {
        rendered.push(page);
        return page !== 1;
      },
      discardActivePageSurface: (page: number) => {
        discarded.push(page);
      },
    });

    view.refreshRenderSurfacePlan(true);

    assert.deepEqual(rendered, [0, 1], '결정이 바뀐 active 쪽은 모두 재래스터를 시도한다');
    assert.deepEqual(
      discarded,
      [1],
      'raster에 실패한 쪽만 active surface를 놓아 has() 건너뛰기로 빈 canvas가 남지 않게 한다',
    );
  } finally {
    if (windowDescriptor) Object.defineProperty(globalThis, 'window', windowDescriptor);
    else Reflect.deleteProperty(globalThis, 'window');
    await vite.close();
  }
});


test('deferred DPR 계획 뒤 strict 갱신은 실제 표면 배율을 확인하고 완성된 대조군을 다시 그리지 않는다', async () => {
  const studioRoot = fileURLToPath(new URL('..', import.meta.url));
  const vite = await createServer({ root: studioRoot, appType: 'custom', logLevel: 'silent', server: { middlewareMode: true } });
  const windowDescriptor = Object.getOwnPropertyDescriptor(globalThis, 'window');
  try {
    Object.defineProperty(globalThis, 'window', { configurable: true, value: { devicePixelRatio: 1.5 } });
    const { CanvasView } = await vite.ssrLoadModule('/src/view/canvas-view.ts');
    const surfaces = [new FakeCanvas(), new FakeCanvas()];
    surfaces[0].dataset.rhwpRenderedZoom = '1';
    surfaces[0].dataset.rhwpRenderScale = '1';
    surfaces[1].dataset.rhwpRenderedZoom = '1';
    surfaces[1].dataset.rhwpRenderScale = '1.5';
    const rendered: number[] = [];
    const view = Object.create(CanvasView.prototype) as Record<string, any>;
    Object.assign(view, {
      pages: [{ width: 800, height: 1100 }, { width: 800, height: 1100 }],
      currentVisiblePages: [0, 1], currentRetainedPages: [0, 1], editingPageIndex: null,
      activePageSnapshot: { pageIndex: 0, source: 'viewport' }, renderSurfacePlan: null,
      renderSurfaceDecisions: new Map(), previousEffectiveDpr: new Map(),
      renderSurfaceEnvironmentKey: null, pendingPrefetchSurfaceReservations: new Map(),
      canvasPool: { activePages: [0, 1], getCanvas: (page: number) => surfaces[page] },
      pageRenderer: { getBackend: () => 'canvas2d', getRenderProfile: () => 'print', getCanvasSurfaceLayerCount: () => 1 },
      viewportManager: { getZoom: () => 1 }, pageSurfaceDescriptor: () => null,
      reconcilePageSurfaceBudget: () => undefined, applySurfaceDecisionDiagnostics: () => undefined,
      renderCanvas: (page: number) => { rendered.push(page); surfaces[page].dataset.rhwpRenderScale = '1.5'; return true; },
      discardActivePageSurface: () => assert.fail('정상 표면을 버리지 않는다'),
    });
    // 스크롤 큐는 아직 paint하지 않았지만 새 DPR 계획을 먼저 확정할 수 있다.
    view.refreshRenderSurfacePlan(false);
    assert.deepEqual(rendered, []);
    view.refreshRenderSurfacePlan(true);
    assert.deepEqual(rendered, [0], '계획만 최신인 쪽을 실제로 그리고 이미 완성된 쪽은 재사용한다');
    view.refreshRenderSurfacePlan(true);
    assert.deepEqual(rendered, [0], '완료 뒤 같은 조건 갱신은 재래스터하지 않는다');
  } finally {
    if (windowDescriptor) Object.defineProperty(globalThis, 'window', windowDescriptor);
    else Reflect.deleteProperty(globalThis, 'window');
    await vite.close();
  }
});


test('CanvasView는 원본 논리 크기를 유지하며 공통 Print 종이 표본의 이중 보간만 피한다', async () => {
  const studioRoot = fileURLToPath(new URL('..', import.meta.url));
  const vite = await createServer({ root: studioRoot, appType: 'custom', logLevel: 'silent', server: { middlewareMode: true } });
  const dprDescriptor = Object.getOwnPropertyDescriptor(globalThis, 'devicePixelRatio');
  try {
    Object.defineProperty(globalThis, 'devicePixelRatio', { configurable: true, value: 1 });
    const { CanvasView } = await vite.ssrLoadModule('/src/view/canvas-view.ts');
    const source = Object.freeze({ width: 800.25, height: 1100.75, marginLeft: 50, printSize: Object.freeze({ width: 798.5, height: 1098.25 }) });
    const view = Object.create(CanvasView.prototype) as Record<string, any>;
    const main = new FakeCanvas();
    Object.assign(view, {
      wasm: { pageCount: 1, getPageInfo: () => source }, pages: [source],
      viewportManager: { cancelPendingZoomRaster() {} }, recalcLayout() {},
      cancelPendingTextEditRefresh() {}, cancelTextEditStaticLayerVerification() {},
      releaseAllRenderedPages() {}, updateVisiblePages() {}, pageSurfaceElements: () => [main],
    });
    let profile = 'print';
    let rasterContent = true;
    let rawSvgCount = 0;
    let backgroundComplex = false;
    let summaryAvailable = true;
    const { PageRenderer } = await vite.ssrLoadModule('/src/view/page-renderer.ts');
    const contentRenderer = Object.create(PageRenderer.prototype) as Record<string, any>;
    Object.assign(contentRenderer, {
      layerSummaryCache: new Map(),
      wasm: { getPageOverlayImages: () => {
        if (!summaryAvailable) throw new Error('메타데이터 확인 불가');
        return JSON.stringify({ hasBehind: false, hasFront: false,
          imageCount: rasterContent ? 1 : 0, rawSvgCount, pageBackgroundComplex: backgroundComplex });
      }, getPageLayerTree: () => JSON.stringify({ root: null }) },
    });
    view.pageRenderer = { getRenderProfile: () => profile,
      hasRasterContent: (page: number) => contentRenderer.hasRasterContent(page), cancelAll() {} };
    view.refreshPages();
    assert.equal(view.pages[0], source, '원본 논리 크기와 편집 metadata를 덮어쓰지 않는다');
    main.width = 799; main.height = 1099;
    view.applyPageSurfaceCssSize(0, main, 1);
    assert.equal(main.style.width, '800.25px');
    assert.equal(main.style.height, '1100.75px');
    assert.equal(main.style.imageRendering, 'crisp-edges', '공통 Print가 발행한 장치 표본은 다시 흐리게 만들지 않는다');
    rasterContent = false;
    view.applyPageSurfaceCssSize(0, main, 1);
    assert.equal(main.style.imageRendering, 'auto', '그림이 없는 Print 쪽의 선과 글자는 compositor의 면적 coverage를 유지한다');
    rawSvgCount = 1;
    view.applyPageSurfaceCssSize(0, main, 1);
    assert.equal(main.style.imageRendering, 'crisp-edges', 'raw SVG도 실제 재생 시 raster가 되므로 표본 소유를 보존한다');
    rawSvgCount = 0;
    backgroundComplex = true;
    view.applyPageSurfaceCssSize(0, main, 1);
    assert.equal(main.style.imageRendering, 'crisp-edges', '본문 image 수가 0이어도 복합 쪽 배경은 그림 표본을 소유할 수 있다');
    backgroundComplex = false;
    summaryAvailable = false;
    view.applyPageSurfaceCssSize(0, main, 1);
    assert.equal(main.style.imageRendering, 'crisp-edges', '확인할 수 없는 메타데이터를 그림이 없다는 증거로 쓰지 않는다');
    summaryAvailable = true;
    rasterContent = true;
    view.applyPageSurfaceCssSize(0, main, 1);
    assert.equal(main.style.imageRendering, 'crisp-edges', '같은 논리 크기의 그림 쪽으로 바뀌면 장치 표본을 다시 보존한다');
    main.width = 400; main.height = 550;
    view.applyPageSurfaceCssSize(0, main, 1);
    assert.equal(main.style.imageRendering, 'auto', '실제 낮은 해상도 표면은 계속 보간한다');
    profile = 'screen';
    main.width = 799; main.height = 1099;
    view.applyPageSurfaceCssSize(0, main, 1);
    assert.equal(main.style.imageRendering, 'auto', 'Screen 표본을 Print metadata로 오인하지 않는다');
    main.width = 801; main.height = 1101;
    view.applyPageSurfaceCssSize(0, main, 1);
    assert.equal(main.style.imageRendering, 'crisp-edges');
  } finally {
    if (dprDescriptor) Object.defineProperty(globalThis, 'devicePixelRatio', dprDescriptor);
    else Reflect.deleteProperty(globalThis, 'devicePixelRatio');
    await vite.close();
  }
});


test('backend 전환 시 active와 익명 pool canvas의 고정 context를 함께 폐기한다', async () => {
  const previousDocument = Object.getOwnPropertyDescriptor(globalThis, 'document');
  const vite = await createServer({
    root: fileURLToPath(new URL('..', import.meta.url)),
    appType: 'custom', logLevel: 'silent', server: { middlewareMode: true },
  });
  try {
    Object.defineProperty(globalThis, 'document', { configurable: true,
      value: { createElement: () => new FakeCanvas() } });
    const { CanvasPool } = await vite.ssrLoadModule('/src/view/canvas-pool.ts');
    const pool = new CanvasPool();
    const active = pool.acquire(0);
    const pooled = pool.acquire(1);
    pool.release(1);
    assert.equal(pool.totalCount, 2);
    pool.discardAll();
    assert.equal(pool.totalCount, 0);
    const next = pool.acquire(0);
    assert.notEqual(next, active);
    assert.notEqual(next, pooled);
    pool.release(0);
    assert.equal(pool.acquire(0), next, '같은 backend에서는 기존 pool 재사용을 유지한다');
  } finally {
    if (previousDocument) Object.defineProperty(globalThis, 'document', previousDocument);
    else Reflect.deleteProperty(globalThis, 'document');
    await vite.close();
  }
});
