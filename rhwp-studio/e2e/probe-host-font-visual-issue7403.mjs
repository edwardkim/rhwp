/** #7403: 한컴 원문을 동일 HCRBatang face로 그린 CanvasKit/Canvas2D 시각 증거.
 * RHWP_HOST_FONT_PATH=.../HANBatang.ttf node e2e/run-with-vite.mjs --
 *   node e2e/probe-host-font-visual-issue7403.mjs --mode=headless
 * 폰트 바이너리는 로컬에서만 읽으며 증거 폴더에는 PNG와 진단만 저장한다.
 */
import assert from 'node:assert/strict';
import { readFileSync, mkdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { runTest, loadApp } from './helpers.mjs';

assert(process.env.RHWP_HOST_FONT_PATH, 'RHWP_HOST_FONT_PATH must supply the verified HCRBatang face');
const font = readFileSync(process.env.RHWP_HOST_FONT_PATH).toString('base64');
const document = readFileSync(new URL('../../samples/re-01-hangul-only-hancom.hwp', import.meta.url)).toString('base64');
const out = path.resolve(process.env.RHWP_HOST_FONT_EVIDENCE || '../output/host-font-7403');
mkdirSync(out, { recursive: true });
const backend = process.argv.find(arg => arg.startsWith('--renderer='))?.split('=')[1] || 'canvaskit';
assert(['canvaskit', 'canvas2d'].includes(backend));

runTest('#7403 HCRBatang host face versus Hancom reference input', async ({ page }) => {
  await loadApp(page, `?renderer=${backend}&canvaskitSurface=software`);
  const result = await page.evaluate(async ({ font, document, backend }) => {
    const decode = input => Uint8Array.from(atob(input), ch => ch.charCodeAt(0));
    const local = await import('/src/core/local-fonts.ts');
    const { CanvasKitLayerRenderer } = await import('/src/view/canvaskit-renderer.ts');
    const { collectHostFontRequests } = await import('/src/core/host-font-requests.ts');
    const wasm = window.__wasm;
    const info = wasm.loadDocument(decode(document), 're-01-hangul-only-hancom.hwp');
    await window.__canvasView.loadDocument();
    const tree = wasm.getPageLayerTreeObject(0, 'screen');
    const renderer = backend === 'canvaskit' ? await CanvasKitLayerRenderer.create('default', 'software') : null;
    const canvas = window.document.createElement('canvas');
    const pageInfo = wasm.getPageInfo(0);
    canvas.width = Math.ceil(pageInfo.width); canvas.height = Math.ceil(pageInfo.height);
    const paintFonts = [];
    const draw = () => {
      const original = CanvasRenderingContext2D.prototype.fillText;
      CanvasRenderingContext2D.prototype.fillText = function(...args) {
        paintFonts.push(this.font); return original.apply(this, args);
      };
      try {
      if (renderer) renderer.renderPage(tree, canvas, 1, pageInfo);
      else wasm.renderPageToCanvas(0, canvas, 1);
      return canvas.toDataURL();
      } finally { CanvasRenderingContext2D.prototype.fillText = original; }
    };
    const canvasDiagnostics = () => ({ localTypefaceCount: wasm.getHostCanvasFontDiagnostics().loaded,
      ...wasm.getHostCanvasFontDiagnostics() });
    const withoutHost = draw();
    const reads = [];
    const provider = {
      async getSnapshot() { return { revision: 'hcr-batang', faces: [{
        id: 'batang', family: '함초롬바탕', fullName: '함초롬바탕 Regular',
        postscriptName: 'HCRBatang', style: 'Regular', weight: 400, slant: 'normal',
      }] }; },
      async readFace(id) { reads.push(id); return { bytes: decode(font).buffer }; },
      subscribe() { return () => {}; },
    };
    await window.rhwpStudio.fonts.setProvider(provider);
    await local.prepareHostFontCatalog();
    const records = collectHostFontRequests(tree);
    if (renderer) await renderer.prepareHostFonts(records);
    else await wasm.prepareCanvasMetrics('canvas2d');
    paintFonts.length = 0;
    const withHost = draw();
    const hostAliasUsedByPaint = paintFonts.some(font => font.includes('__rhwp_host_face_'));
    const diagnostics = renderer?.diagnostics() ?? canvasDiagnostics();
    await window.__canvasView.refreshFontResources();
    const studioDiagnostics = renderer ? window.__canvasView.getCurrentCanvasKitRenderDiagnostics() : canvasDiagnostics();
    const studio = window.document.querySelector('#scroll-container canvas').toDataURL();
    await window.rhwpStudio.fonts.setProvider(null);
    renderer?.resetDocumentResources();
    if (!renderer) await wasm.prepareCanvasMetrics('canvas2d');
    const detached = draw();
    renderer?.dispose();
    return { backend, hostAliasUsedByPaint, withoutHost, withHost, detached, studio, fonts: info.fontsUsed, reads,
      selected: records.map(record => record.postscriptName), diagnostics, studioDiagnostics,
      pageInfo, width: canvas.width, height: canvas.height };
  }, { font, document, backend });
  assert.deepEqual(result.selected, ['HCRBatang']);
  assert(result.reads.length > 0);
  assert.equal(result.diagnostics.localTypefaceCount, 1);
  assert.equal(result.studioDiagnostics.localTypefaceCount, 1);
  if (backend === 'canvaskit') {
    assert.equal(result.diagnostics.lastRenderError, null);
    assert.deepEqual(result.diagnostics.lastUnsupportedOps, []);
    assert.notEqual(result.withHost, result.withoutHost, 'host face changes real pixels');
  } else {
    assert(result.hostAliasUsedByPaint, 'actual Canvas2D paint consumes the host FontFace');
    // The OS can already have the same HCRBatang; equal pixels are valid in this control.
    assert.equal(result.diagnostics.failed, 0);
  }
  assert.equal(result.detached, result.withoutHost, 'detach restores the original no-provider rendering');
  for (const name of ['withoutHost', 'withHost', 'detached', 'studio']) {
    writeFileSync(path.join(out, `${name}.png`), Buffer.from(result[name].split(',')[1], 'base64'));
    delete result[name];
  }
  writeFileSync(path.join(out, 'diagnostics.json'), JSON.stringify(result, null, 2));
  console.log(JSON.stringify(result, null, 2));
}, { skipLoadApp: true });
