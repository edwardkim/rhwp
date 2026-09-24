import assert from 'node:assert/strict';
import { runTest, loadApp, createNewDocument, screenshot } from './helpers.mjs';

runTest('Issue #7403 host font identity, replacement and document preservation', async ({ page }) => {
  await loadApp(page, '?renderer=canvaskit&canvaskitSurface=raster');
  const result = await page.evaluate(async () => {
    const { CanvasKitLayerRenderer } = await import('/src/view/canvaskit-renderer.ts');
    const local = await import('/src/core/local-fonts.ts');
    const { collectHostFontRequests } = await import('/src/core/host-font-requests.ts');
    const { fontFamilyChainForDisplay } = await import('/src/core/font-substitution.ts');
    const family = 'RHWP Host Smoke';
    const cssBefore = fontFamilyChainForDisplay(family);
    const storedBefore = localStorage.getItem('rhwp-local-fonts');
    let notify = () => {};
    let revision = 'serif';
    let fail = false;
    const reads = [];
    const provider = {
      async getSnapshot() {
        return { revision, faces: [400, 700].map(weight => ({
          id: `face-${weight}`, family, fullName: `${family} ${weight === 400 ? 'Regular' : 'Bold'}`,
          postscriptName: `RHWPHostSmoke-${weight}`, style: weight === 400 ? 'Regular' : 'Bold',
          weight, slant: 'normal',
        })) };
      },
      async readFace(id, version, signal) {
        reads.push({ id, version });
        if (fail) throw new Error('simulated revoked access');
        const name = version === 'sans' ? 'NotoSansKR' : 'NotoSerifKR';
        const bytes = await (await fetch(`/fonts/${name}-${id.endsWith('700') ? 'Bold' : 'Regular'}.woff2`, { signal })).arrayBuffer();
        return { bytes };
      },
      subscribe(listener) { notify = listener; return () => { notify = () => {}; }; },
    };
    await window.rhwpStudio.fonts.setProvider(provider);
    const metadataReads = reads.length;
    const renderer = await CanvasKitLayerRenderer.create('default', 'raster');
    const off = local.onHostFontsChanged(() => renderer.resetDocumentResources());
    const canvas = document.createElement('canvas');
    canvas.width = 650; canvas.height = 150;
    const tree = { pageWidth: 650, pageHeight: 150, root: { kind: 'leaf',
      bounds: { x: 0, y: 0, width: 650, height: 150 },
      ops: [false, true].map((bold, i) => ({ type: 'textRun', text: '한글 글꼴 가나다 ABC 0123',
        bbox: { x: 15, y: 10 + i * 65, width: 610, height: 48 }, baseline: 40,
        style: { fontFamily: family, fontSize: 32, bold, color: '#111111' },
      })) } };
    async function render(input = tree) {
      await local.prepareHostFontCatalog();
      await renderer.prepareHostFonts(collectHostFontRequests(input));
      renderer.renderPage(input, canvas, 1);
      const diagnostic = renderer.diagnostics();
      if (diagnostic.lastRenderError) throw new Error(diagnostic.lastRenderError);
      return { image: canvas.toDataURL(), diagnostic };
    }
    const regularBold = await render();
    const explicit = structuredClone(tree);
    explicit.root.ops[0].style.fontFamily = `${family} Regular`;
    explicit.root.ops[1].style.fontFamily = `${family} Bold`;
    explicit.root.ops[1].style.bold = false;
    const exactFace = await render(explicit);
    revision = 'sans'; notify();
    const replaced = await render();
    fail = true; revision = 'failed'; notify();
    const failed = await render();
    fail = false; revision = 'recovered'; notify();
    const recovered = await render();
    const cssAfter = fontFamilyChainForDisplay(family);
    const storedAfter = localStorage.getItem('rhwp-local-fonts');
    const images = [regularBold, replaced, recovered];
    const panel = document.createElement('div');
    panel.id = 'host-font-proof';
    panel.style.cssText = 'position:fixed;inset:0;z-index:999999;background:white;color:#111;padding:24px;overflow:auto';
    ['Serif Regular / Bold', 'Same names, replacement bytes: Sans', 'Recovered: Serif'].forEach((label, i) => {
      const title = document.createElement('h3'); title.textContent = label;
      const image = document.createElement('img'); image.src = images[i].image;
      panel.append(title, image);
    });
    document.body.append(panel);
    off(); renderer.dispose();
    window.__host7403 = { reads, provider, notify: () => notify(),
      replace: version => { revision = version; notify(); } };
    return { metadataReads, reads, cssBefore, cssAfter, storedBefore, storedAfter,
      exactMatchesStyles: regularBold.image === exactFace.image,
      replacementChangesPixels: regularBold.image !== replaced.image,
      recoveryRestoresPixels: regularBold.image === recovered.image,
      failedCount: failed.diagnostic.localTypefaceLoadFailureCount,
      registered: regularBold.diagnostic.localTypefaceCount };
  });
  assert.equal(result.metadataReads, 0);
  assert.equal(result.registered, 2);
  assert.equal(result.exactMatchesStyles, true, 'family+bold selects actual Bold without double emboldening');
  assert.equal(result.replacementChangesPixels, true, 'same names and count load replacement bytes');
  assert.equal(result.failedCount, 2);
  assert.equal(result.recoveryRestoresPixels, true);
  assert.equal(result.cssAfter, result.cssBefore);
  assert.equal(result.storedAfter, result.storedBefore);
  await screenshot(page, 'host-font-7403-faces');
  await page.evaluate(() => document.querySelector('#host-font-proof').remove());

  await createNewDocument(page);
  const documentResult = await page.evaluate(async () => {
    const wasm = window.__wasm;
    const text = '한글 가나다 ABC 0123';
    wasm.insertText(0, 0, 0, text);
    const fontId = wasm.findOrCreateFontId('RHWP Host Smoke');
    wasm.applyCharFormat(0, 0, 0, text.length, JSON.stringify({ fontId, fontSize: 2400 }));
    wasm.applyCharFormat(0, 0, 3, 6, JSON.stringify({ bold: true }));
    await window.__canvasView.loadDocument();
    window.__documentState.markDirty('host-font-smoke');
    window.__host7403.saved = { hwp: wasm.exportHwp(), hwpx: wasm.exportHwpx(),
      generation: wasm.documentGeneration, properties: wasm.getCharPropertiesAt(0, 0, 3) };
    const before = document.querySelector('#scroll-container canvas').toDataURL();
    const count = window.__canvasView.getCurrentCanvasKitRenderDiagnostics()?.localTypefaceCount;
    window.__host7403.replace('sans');
    return { before, count };
  });
  assert.equal(documentResult.count, 2, 'actual document prepares both host faces');
  await page.waitForFunction(before => {
    const diagnostics = window.__canvasView.getCurrentCanvasKitRenderDiagnostics();
    return diagnostics?.localTypefaceCount === 2 && diagnostics.localTypefacePendingCount === 0
      && document.querySelector('#scroll-container canvas').toDataURL() !== before;
  }, { timeout: 15000 }, documentResult.before);
  const preserved = await page.evaluate(async () => {
    const wasm = window.__wasm;
    const saved = window.__host7403.saved;
    const sameBytes = (a, b) => a.length === b.length && a.every((value, i) => value === b[i]);
    const dirty = window.__documentState.isDirty();
    const generation = wasm.documentGeneration;
    const hwpUnchanged = sameBytes(saved.hwp, wasm.exportHwp());
    // ZIP timestamps need not be byte identical; inspect the reopened HWPX document instead.
    const { WasmBridge } = await import('/src/core/wasm-bridge.ts');
    const reopened = new WasmBridge();
    const roundtrips = [];
    for (const [name, data] of [['smoke.hwp', wasm.exportHwp()], ['smoke.hwpx', wasm.exportHwpx()]]) {
      const info = reopened.loadDocument(data, name);
      roundtrips.push({ name, fonts: info.fontsUsed, properties: reopened.getCharPropertiesAt(0, 0, 3) });
    }
    return { dirty, generation, expectedGeneration: saved.generation, hwpUnchanged, roundtrips };
  });
  assert.equal(preserved.dirty, true);
  assert.equal(preserved.generation, preserved.expectedGeneration);
  assert.equal(preserved.hwpUnchanged, true);
  for (const roundtrip of preserved.roundtrips) {
    assert(roundtrip.fonts.includes('RHWP Host Smoke'), `${roundtrip.name} retains the document font name`);
    assert.equal(roundtrip.properties.bold, true);
    assert(!roundtrip.fonts.some(name => name.includes('face-') || name.includes('host\"')));
  }
  await screenshot(page, 'host-font-7403-document');
  await page.evaluate(async () => { await window.rhwpStudio.fonts.setProvider(null); });
  await page.waitForFunction(() => window.__canvasView.getCurrentCanvasKitRenderDiagnostics()?.localTypefaceCount === 0);
  console.log(JSON.stringify({ result, preserved: { ...preserved, roundtrips: preserved.roundtrips.map(item => ({ name: item.name, fonts: item.fonts, bold: item.properties.bold })) } }, null, 2));
}, { skipLoadApp: true });
