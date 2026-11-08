import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { closeBrowser, closePage, createPage, launchBrowser, loadApp } from './helpers.mjs';

// Full source/Print admission and real-font visual results: #7998 stage7.
// This boundary checks actual CanvasKit strokes, independent of font outlines.
const browser = await launchBrowser();
const page = await createPage(browser);
try {
  await loadApp(page);
  const bytes = readFileSync(new URL('../../pdf/issue7998/periodic-borders/native-dot-widths.hwpx', import.meta.url));
  const result = await page.evaluate(async ({ bytes, moduleUrl }) => {
    const bridge = window.__wasm;
    bridge.loadDocument(new Uint8Array(bytes), 'native-dot-widths.hwpx');
    bridge.borrowDocumentHandle().setShowTransparentBorders(false);
    const tree = bridge.withPortableMetrics(() => bridge.getPageLayerTreeObject(0, 'print'));
    const transparent = [];
    const visit = node => {
      if (node.kind === 'group') node.children.forEach(visit);
      else if (node.kind === 'clipRect') visit(node.child);
      else {
        node.ops = node.ops.filter(op => op.type !== 'textRun' && op.type !== 'charOverlap');
        for (const op of node.ops) {
          if (op.type === 'rectangle' && op.style?.fillColor === null
              && op.style?.strokeColor === null && op.style?.strokeWidth === 0
              && op.bbox.width > tree.pageWidth * 0.9) transparent.push(op.bbox);
        }
      }
    };
    visit(tree.root);
    const { CanvasKitLayerRenderer } = await import(moduleUrl);
    const renderer = await CanvasKitLayerRenderer.create('default', 'software');
    try {
      const calls = [];
      const api = renderer.canvasKit.PathEffect;
      const make = api.MakeDash;
      api.MakeDash = function (intervals, ...rest) {
        calls.push(Array.from(intervals));
        return make.call(this, intervals, ...rest);
      };
      const canvas = document.createElement('canvas');
      canvas.width = Math.ceil(tree.pageWidth);
      canvas.height = Math.ceil(tree.pageHeight);
      const output = renderer.renderPage(tree, canvas, 1);
      api.MakeDash = make;
      const image = new Image();
      image.src = output.toDataURL('image/png');
      await image.decode();
      const copy = document.createElement('canvas');
      copy.width = canvas.width; copy.height = canvas.height;
      const ctx = copy.getContext('2d');
      ctx.drawImage(image, 0, 0);
      const edgeInk = transparent.map(box => {
        const pixels = ctx.getImageData(Math.ceil(box.x) + 1, Math.floor(box.y) - 1,
          Math.max(1, Math.floor(box.width) - 2), 3).data;
        let ink = 0;
        for (let i = 0; i < pixels.length; i += 4) {
          if (pixels[i + 3] && Math.min(pixels[i], pixels[i + 1], pixels[i + 2]) < 232) ink++;
        }
        return ink;
      });
      return { calls, edgeInk, diagnostics: renderer.diagnostics() };
    } finally {
      renderer.dispose();
    }
  }, { bytes: Array.from(bytes), moduleUrl: process.env.PERIODIC_RENDERER_MODULE
    || '/src/view/canvaskit-renderer.ts' });
  for (const expected of [[1.12, 1.6], [1.6, 2.4], [2.72, 4], [8.32, 12.48]]) {
    assert.ok(result.calls.some(p => p.length === 2 && p.every((v, i) => Math.abs(v - expected[i]) < 1e-6)),
      `independent Print repeat not consumed: ${expected}`);
  }
  assert.ok(result.edgeInk.length > 0, 'transparent source frame must be exercised');
  assert.ok(result.edgeInk.every(ink => ink === 0), 'explicit transparent frames must paint no outline');
  assert.equal(result.diagnostics.lastRenderCompleted, true);
  assert.deepEqual(result.diagnostics.lastUnexpectedUnsupportedOps, []);
  console.log(JSON.stringify(result));
} finally {
  await closePage(page);
  await closeBrowser(browser);
}
