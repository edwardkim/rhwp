import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runTest, loadApp, createNewDocument, screenshot, clickEditArea, typeText } from './helpers.mjs';

async function verifyFacePaint(page) {
  const fixtures = Object.fromEntries(['ttc', 'Regular', 'Italic', 'Oblique'].map(name => [name,
    readFileSync(new URL(`../../tests/fixtures/fonts/RHWPHostFixture${name === 'ttc' ? '.ttc' : `-${name}.ttf`}`, import.meta.url)).toString('base64')]));
  const result = await page.evaluate(async fixtures => {
    const { CanvasKitLayerRenderer } = await import('/src/view/canvaskit-renderer.ts');
    const local = await import('/src/core/local-fonts.ts');
    const { collectHostFontRequests } = await import('/src/core/host-font-requests.ts');
    const decode = name => Uint8Array.from(atob(fixtures[name]), ch => ch.charCodeAt(0)).buffer;
    const renderer = await CanvasKitLayerRenderer.create('default', 'software');
    const off = local.onHostFontsChanged(() => renderer.resetDocumentResources());
    const canvas = document.createElement('canvas'); canvas.width = 180; canvas.height = 100;
    let source = 'ttc', faceIndex = 1, slant = 'italic';
    const reads = [];
    const provider = {
      async getSnapshot() { return { revision: `${source}-${faceIndex}-${slant}`, faces: [{
        id: 'face', family: 'RHWP Host Fixture', fullName: 'RHWP Host Fixture Selected',
        postscriptName: 'HostFixture-Selected', style: 'Selected', weight: 400, slant,
      }] }; },
      async readFace(id) { reads.push(id); return { bytes: decode(source), faceIndex: source === 'ttc' ? faceIndex : 0 }; },
      subscribe() { return () => {}; },
    };
    async function render(type, italic = true) {
      await window.rhwpStudio.fonts.setProvider(provider);
      const tree = { pageWidth: 180, pageHeight: 100, root: { kind: 'leaf',
        bounds: { x: 0, y: 0, width: 180, height: 100 }, ops: [{
          type, text: 'A가', bbox: { x: 10, y: 10, width: 150, height: 70 }, baseline: 56,
          rotation: 0, positions: [0, 70, 140], positionsComplete: true,
          charOverlap: type === 'charOverlap' ? { borderType: 1, innerCharSize: 80 } : undefined,
          style: { fontFamily: italic ? 'RHWP Host Fixture' : 'HostFixture-Selected',
            fontSize: 64, italic, color: '#111111' },
        }] } };
      await renderer.prepareHostFonts(collectHostFontRequests(tree));
      renderer.renderPage(tree, canvas, 1);
      const diagnostics = renderer.diagnostics();
      if (diagnostics.lastRenderError || diagnostics.lastUnsupportedOps.length) throw new Error(JSON.stringify(diagnostics));
      return { image: canvas.toDataURL(), count: diagnostics.localTypefaceCount,
        failed: diagnostics.localTypefaceLoadFailureCount };
    }
    const checks = [];
    const pictures = [];
    for (const type of ['textRun', 'charOverlap']) {
      source = 'Regular'; slant = 'normal';
      const upright = await render(type, false);
      for (const [style, index] of [['Italic', 1], ['Oblique', 2]]) {
        source = 'ttc'; faceIndex = index; slant = style.toLowerCase();
        const fromCollection = await render(type);
        source = style;
        const standalone = await render(type, false);
        checks.push({ type, style, count: fromCollection.count,
          sameAsStandalone: fromCollection.image === standalone.image,
          differsFromUpright: fromCollection.image !== upright.image });
        pictures.push({ title: `${type}: TTC face ${index} (${style})`, image: fromCollection.image });
      }
    }
    source = 'ttc'; faceIndex = 99;
    const invalid = await render('textRun');
    faceIndex = 1; slant = 'italic';
    const recovered = await render('textRun');
    const panel = document.createElement('div'); panel.id = 'host-font-face-contract';
    panel.style.cssText = 'position:fixed;inset:0;z-index:999999;background:white;color:black;padding:24px;overflow:auto';
    for (const picture of pictures) {
      const title = document.createElement('h3'); title.textContent = picture.title;
      const img = document.createElement('img'); img.src = picture.image; panel.append(title, img);
    }
    document.body.append(panel);
    off(); renderer.dispose(); await window.rhwpStudio.fonts.setProvider(null);
    return { checks, invalid, recoveredCount: recovered.count, reads: reads.length };
  }, fixtures);
  for (const check of result.checks) {
    assert.equal(check.count, 1, `${check.type}/${check.style} prepares selected host face`);
    assert.equal(check.sameAsStandalone, true, `${check.type}/${check.style}: TTC matches independent TTF, no double skew`);
    assert.equal(check.differsFromUpright, true, `${check.type}/${check.style}: selected outline is visible`);
  }
  assert.equal(result.invalid.count, 0);
  assert.equal(result.invalid.failed, 1, 'invalid TTC index fails closed');
  assert.equal(result.recoveredCount, 1);
  await screenshot(page, 'host-font-7403-ttc-slants');
  await page.evaluate(() => document.querySelector('#host-font-face-contract').remove());
  console.log('Face paint contracts:', JSON.stringify({ ...result, invalid: { count: result.invalid.count, failed: result.invalid.failed } }));
}

runTest('Issue #7403 host font identity, replacement and document preservation', async ({ page }) => {
  await loadApp(page, '?renderer=canvaskit&canvaskitSurface=software');
  await verifyFacePaint(page);
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
    const renderer = await CanvasKitLayerRenderer.create('default', 'software');
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
  // Preserve real undo and redo entries across a font-only refresh, including a saved document.
  console.log('history: start');
  await clickEditArea(page);
  await page.evaluate(() => window.__inputHandler.cursor.moveTo({ sectionIndex: 0, paragraphIndex: 0,
    charOffset: window.__wasm.getParagraphLength(0, 0) }));
  console.log('history: typing');
  await typeText(page, 'Z');
  await page.evaluate(() => {
    window.__inputHandler.performUndo();
    window.__documentState.markClean('host-save');
    const history = window.__inputHandler.history;
    window.__host7403.history = { undo: [...history.undoStack], redo: [...history.redoStack],
      bytes: window.__wasm.exportHwp(), length: window.__wasm.getParagraphLength(0, 0),
      before: document.querySelector('#scroll-container canvas').toDataURL() };
    window.__host7403.replace('serif');
  });
  await page.waitForFunction(() => {
    const d = window.__canvasView.getCurrentCanvasKitRenderDiagnostics();
    return d?.localTypefaceCount === 2 && d.localTypefacePendingCount === 0
      && document.querySelector('#scroll-container canvas').toDataURL() !== window.__host7403.history.before;
  });
  console.log('history: refreshed');
  const historyResult = await page.evaluate(() => {
    const saved = window.__host7403.history;
    const history = window.__inputHandler.history;
    const same = (a, b) => a.length === b.length && a.every((value, i) => value === b[i]);
    const result = { clean: !window.__documentState.isDirty(),
      bytesUnchanged: same(saved.bytes, window.__wasm.exportHwp()),
      undoUnchanged: same(saved.undo, history.undoStack), redoUnchanged: same(saved.redo, history.redoStack),
      redoCount: history.redoStack.length, beforeLength: saved.length };
    window.__inputHandler.performRedo(); result.redoneLength = window.__wasm.getParagraphLength(0, 0);
    window.__inputHandler.performUndo(); result.undoneLength = window.__wasm.getParagraphLength(0, 0);
    return result;
  });
  assert.equal(historyResult.clean, true);
  assert.equal(historyResult.bytesUnchanged, true);
  assert.equal(historyResult.undoUnchanged, true);
  assert.equal(historyResult.redoUnchanged, true);
  assert(historyResult.redoCount > 0);
  assert.equal(historyResult.redoneLength, historyResult.beforeLength + 1);
  assert.equal(historyResult.undoneLength, historyResult.beforeLength);

  console.log('history: verified');
  // A provider deliberately ignores AbortSignal. Late bytes must not repaint a new document.
  await page.evaluate(async () => {
    const bytes = await (await fetch('/fonts/NotoSerifKR-Regular.woff2')).arrayBuffer();
    const nextBytes = await (await fetch('/fonts/NotoSansKR-Regular.woff2')).arrayBuffer();
    const pending = [];
    const faces = [
      { id: 'regular', family: 'RHWP Host Smoke', fullName: 'RHWP Host Smoke Regular',
        postscriptName: 'Slow-Regular', style: 'Regular', weight: 400, slant: 'normal' },
      { id: 'next', family: 'Next Host', fullName: 'Next Host Regular',
        postscriptName: 'Next-Regular', style: 'Regular', weight: 400, slant: 'normal' },
    ];
    const provider = {
      async getSnapshot() { return { revision: 'same', faces }; },
      readFace(id, revision, signal) {
        if (id === 'next') return Promise.resolve({ bytes: nextBytes });
        return new Promise(resolve => pending.push({ signal, resolve }));
      },
      subscribe() { return () => {}; },
    };
    window.__host7403.race = { pending, bytes, nextBytes, faces, provider,
      generation: window.__wasm.documentGeneration };
    await window.rhwpStudio.fonts.setProvider(provider);
  });
  await page.waitForFunction(() => window.__host7403.race.pending.length > 0);
  console.log('race: switching document');
  await createNewDocument(page);
  await page.evaluate(async () => {
    const wasm = window.__wasm;
    wasm.insertText(0, 0, 0, '새 문서 ABC');
    wasm.applyCharFormat(0, 0, 0, 8, JSON.stringify({ fontId: wasm.findOrCreateFontId('Next Host'), fontSize: 2400 }));
    await window.__canvasView.loadDocument();
  });
  console.log('race: current document ready');
  const switched = await page.evaluate(async () => {
    const race = window.__host7403.race;
    const before = document.querySelector('#scroll-container canvas').toDataURL();
    const bytesBefore = window.__wasm.exportHwp();
    const countBefore = window.__canvasView.getCurrentCanvasKitRenderDiagnostics()?.localTypefaceCount;
    for (const pending of race.pending) pending.resolve({ bytes: race.bytes });
    await new Promise(resolve => setTimeout(resolve, 150));
    const bytesAfter = window.__wasm.exportHwp();
    return { changedDocument: window.__wasm.documentGeneration !== race.generation,
      sameImage: before === document.querySelector('#scroll-container canvas').toDataURL(),
      sameBytes: bytesBefore.length === bytesAfter.length && bytesBefore.every((value, i) => value === bytesAfter[i]),
      countBefore, countAfter: window.__canvasView.getCurrentCanvasKitRenderDiagnostics()?.localTypefaceCount };
  });
  assert.equal(switched.changedDocument, true);
  assert.equal(switched.sameImage, true);
  assert.equal(switched.sameBytes, true);
  assert.equal(switched.countBefore, 1);
  assert.equal(switched.countAfter, 1);

  console.log('race: late document read discarded');
  // Reattach a slow provider, replace it using the same ID/revision, then release the stale read.
  await page.evaluate(async () => {
    const race = window.__host7403.race;
    race.pending = [];
    race.slow = { ...race.provider,
      readFace: (_id, _revision, signal) => new Promise(resolve => race.pending.push({ signal, resolve })) };
    await window.rhwpStudio.fonts.setProvider(race.slow);
  });
  await page.waitForFunction(() => window.__host7403.race.pending.length > 0);
  await page.evaluate(async () => {
    const race = window.__host7403.race;
    await window.rhwpStudio.fonts.setProvider({ ...race.provider, readFace: async () => ({ bytes: race.nextBytes }) });
  });
  await page.waitForFunction(() => window.__canvasView.getCurrentCanvasKitRenderDiagnostics()?.localTypefaceCount === 1);
  const replacedRace = await page.evaluate(async () => {
    const race = window.__host7403.race;
    const before = document.querySelector('#scroll-container canvas').toDataURL();
    const aborted = race.pending.every(item => item.signal.aborted);
    for (const item of race.pending) item.resolve({ bytes: race.bytes });
    await new Promise(resolve => setTimeout(resolve, 150));
    return { aborted, sameImage: before === document.querySelector('#scroll-container canvas').toDataURL(),
      count: window.__canvasView.getCurrentCanvasKitRenderDiagnostics()?.localTypefaceCount };
  });
  assert.equal(replacedRace.aborted, true);
  assert.equal(replacedRace.sameImage, true);
  assert.equal(replacedRace.count, 1);
  console.log('Document lifecycle contracts:', JSON.stringify({ historyResult, switched, replacedRace }));

  await page.evaluate(async () => { await window.rhwpStudio.fonts.setProvider(null); });
  await page.waitForFunction(() => window.__canvasView.getCurrentCanvasKitRenderDiagnostics()?.localTypefaceCount === 0);
  console.log(JSON.stringify({ result, preserved: { ...preserved, roundtrips: preserved.roundtrips.map(item => ({ name: item.name, fonts: item.fonts, bold: item.properties.bold })) } }, null, 2));
}, { skipLoadApp: true });
