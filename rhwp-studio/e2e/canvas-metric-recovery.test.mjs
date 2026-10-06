import { mkdirSync, writeFileSync } from 'node:fs';
import { assert, loadHwpFile, runTest, setTestCase } from './helpers.mjs';

// Controlled descriptor faults, not evidence of a naturally failing font/platform.
const negative = process.env.RHWP_METRIC_RECOVERY_NEGATIVE === '1';
await runTest('Canvas metric descriptor recovery (#7084)', async ({ page }) => {
  const results = [];
  for (const ext of ['hwp', 'hwpx']) {
    for (const mode of ['transient', 'persistent', 'stale-view', 'stale-document', 'stale-font', 'ordinary']) {
      setTestCase(`${ext}: ${mode}`);
      await loadHwpFile(page, `issue3587/c-form-labnote-001-stage11-filled.${ext}`);
      const result = await page.evaluate(async ({ mode, negative, ext }) => {
        const w = window.__wasm;
        const view = window.__canvasView;
        await w.prepareCanvasMetrics('canvas2d');
        if (!w.doc.canvasMetricsActive()) throw Error('Active supplemental metrics required');
        view.gotoPage(1);
        const frames = async () => {
          for (let i = 0; i < 6; i++) await new Promise(resolve => requestAnimationFrame(resolve));
        };
        await frames();
        const replacement = mode === 'stale-document'
          ? new Uint8Array(await (await fetch(`/samples/issue3587/c-form-labnote-001-stage11-filled.${ext}`)).arrayBuffer())
          : null;
        const proto = CanvasRenderingContext2D.prototype;
        const font = Object.getOwnPropertyDescriptor(proto, 'font');
        const renderer = view.pageRenderer;
        const original = {
          paint: renderer.renderPage, prepare: w.prepareCanvasMetrics,
          invalidate: w.invalidateCanvasMetrics, refresh: view.refreshPages,
          error: console.error, recover: view.canvasMetricRecovery.recover,
        };
        const errors = [];
        let prepares = 0, invalidates = 0, refreshes = 0, painting = false;
        let fault = true, successfulPaints = 0, guard;
        let release;
        const wait = new Promise(resolve => { release = resolve; });
        const generation = w.canvasMetricFontGeneration;
        Object.defineProperty(proto, 'font', { ...font, get() {
          const value = font.get.call(this);
          return painting && fault ? `${value}, serif` : value;
        } });
        console.error = (...args) => errors.push(args.map(String).join(' '));
        renderer.renderPage = function (...args) {
          if (mode === 'ordinary' && fault) throw Error('ordinary render fault');
          painting = true;
          try {
            const result = original.paint.apply(this, args);
            successfulPaints++;
            return result;
          } finally { painting = false; }
        };
        w.invalidateCanvasMetrics = function () { invalidates++; return original.invalidate.call(this); };
        w.prepareCanvasMetrics = function (backend, current) {
          prepares++;
          guard = current;
          // Hold the real metric session at its font-ready boundary, not a copied implementation.
          if (mode.startsWith('stale-')) {
            const session = this.canvasMetrics;
            const prepare = session.prepare;
            session.prepare = function (...args) {
              args[4] = () => wait;
              return prepare.apply(this, args);
            };
            try { return original.prepare.call(this, backend, current); }
            finally { session.prepare = prepare; }
          }
          return original.prepare.call(this, backend, current);
        };
        view.refreshPages = function () { refreshes++; return original.refresh.call(this); };
        if (negative) view.canvasMetricRecovery.recover = async () => false;
        let returned, staleGuard = null;
        try {
          returned = view.renderCanvas(1, document.createElement('canvas'));
          if (mode !== 'persistent') fault = false;
          if (mode.startsWith('stale-')) {
            await Promise.resolve(); // Recovery has entered the real fonts-ready await.
            if (mode === 'stale-document') w.loadDocument(replacement, `replacement.${ext}`);
            else if (mode === 'stale-font') w.invalidateCanvasMetricFonts();
            else view.prepareDocumentLoad();
            staleGuard = guard?.() ?? null;
            release();
          }
          await frames();
          if (mode === 'persistent') {
            // More paint failures in the same generation still cannot refill the budget.
            view.renderCanvas(1, document.createElement('canvas'));
            await frames();
          }
          return {
            returned, prepares, invalidates, refreshes, successfulPaints, errors, staleGuard,
            generationUnchanged: generation === w.canvasMetricFontGeneration,
            active: w.doc.canvasMetricsActive(),
            pageSurfaceRestored: !!view.canvasPool.getCanvas(1)?.parentElement,
          };
        } finally {
          release();
          Object.defineProperty(proto, 'font', font);
          renderer.renderPage = original.paint;
          w.prepareCanvasMetrics = original.prepare;
          w.invalidateCanvasMetrics = original.invalidate;
          view.refreshPages = original.refresh;
          console.error = original.error;
          view.canvasMetricRecovery.recover = original.recover;
        }
      }, { mode, negative, ext });
      results.push({ ext, mode, ...result });
      console.log(JSON.stringify(results.at(-1)));
      assert(result.returned === false, 'initial faulty paint rejected');
      assert(result.generationUnchanged === (mode !== 'stale-font'), 'only the external font change advances its generation');
      assert(result.prepares === (mode === 'ordinary' ? 0 : 1), 'preparation count is bounded and error-specific');
      assert(result.invalidates === result.prepares + (mode === 'stale-font' ? 1 : 0), 'one recovery invalidation plus explicit external font change only');
      assert(result.refreshes === (mode.startsWith('stale-') || mode === 'ordinary' ? 0 : 1), 'only current recovery refreshes');
      if (mode === 'transient') {
        assert(result.active && result.successfulPaints > 0 && result.pageSurfaceRestored,
          'automatic recovery restores an attached page surface with active metrics');
      }
      if (mode === 'persistent') assert(result.errors.length >= 2, 'persistent painter failure remains diagnostic');
      if (mode.startsWith('stale-')) assert(result.staleGuard === false, 'obsolete async preparation is rejected');
    }
  }
  const output = new URL('../../output/7084/stage6/', import.meta.url);
  mkdirSync(output, { recursive: true });
  writeFileSync(new URL(negative ? 'browser-recovery-negative.json' : 'browser-recovery.json', output), JSON.stringify(results, null, 2));
});
