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
