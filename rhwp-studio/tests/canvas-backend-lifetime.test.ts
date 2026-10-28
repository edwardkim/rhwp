import test from 'node:test';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
import { createServer } from 'vite';
import { CanvasPool } from '../src/view/canvas-pool.ts';

// Canvas width/height reset clears backing pixels, but cannot change context mode.
class Canvas {
  width = 0; height = 0;
  parentElement: { removeChild(canvas: Canvas): void } | null = null;
  dataset: Record<string, string> = {};
  classList = { add() {} };
  private context: string | null = null;
  getContext(kind: string): object | null {
    this.context ??= kind;
    return this.context === kind ? {} : null;
  }
  removeAttribute() {}
}

test('renderer transition replaces incompatible active and retained Canvas contexts', async () => {
  const vite = await createServer({ configFile: fileURLToPath(new URL('../vite.config.ts', import.meta.url)),
    server: { middlewareMode: true }, appType: 'custom' });
  const original = Object.getOwnPropertyDescriptor(globalThis, 'document');
  try {
    const { CanvasView } = await vite.ssrLoadModule('/src/view/canvas-view.ts');
    Object.defineProperty(globalThis, 'document', { configurable: true, value: { createElement: () => new Canvas() } });
    for (const [previous, previousContext, selected, selectedContext] of [
      ['canvaskit', 'webgl2', 'canvas2d', '2d'], ['canvas2d', '2d', 'canvaskit', 'webgl2'],
    ]) {
      const pool = new CanvasPool();
      const active = pool.acquire(0); assert.ok(active.getContext(previousContext));
      const retained = pool.acquire(1); assert.ok(retained.getContext(previousContext)); pool.release(1);
      let backend = previous;
      const view = Object.create(CanvasView.prototype) as Record<string, any>;
      view.viewportManager = { cancelPendingZoomRaster: () => false };
      view.zoomRefreshRequired = false;
      view.activeRendererDecisionKey = 'old';
      view.pageSurfaceLru = { clear() {} };
      view.canvasPool = pool;
      view.pageRenderer = { getBackend: () => backend, configure: (next: string) => {
        const changed = backend !== next; backend = next; return changed;
      }, invalidateDocumentRevision() {} };
      view.eventBus = { emit() {} };
      view.releaseAllRenderedPages = () => pool.releaseAll();
      view.applyRendererSelection({ backend: selected, canvaskitRenderer: null,
        diagnostics: { decisionKey: 'new', renderProfile: 'screen', fallbackReason: 'canvaskitResourcePreparationFailed' } });
      assert.deepEqual(pool.activePages, []);
      const next = pool.acquire(0);
      assert.notEqual(next, active); assert.notEqual(next, retained);
      assert.ok(next.getContext(selectedContext), 'new backend must receive a usable context');
      pool.release(0);
      view.applyRendererSelection({ backend: selected, canvaskitRenderer: null,
        diagnostics: { decisionKey: 'revision', renderProfile: 'screen', fallbackReason: null } });
      assert.equal(pool.acquire(0), next, 'same-backend revisions retain compatible pooled canvases');
    }
  } finally {
    if (original) Object.defineProperty(globalThis, 'document', original);
    else delete (globalThis as any).document;
    await vite.close();
  }
});
