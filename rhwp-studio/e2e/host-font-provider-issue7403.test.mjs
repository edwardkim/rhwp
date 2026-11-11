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

async function verifyCanvas2DFaces(page) {
  const fixtures = Object.fromEntries(['ttc', 'Regular', 'Italic', 'Oblique'].map(name => [name,
    readFileSync(new URL(`../../tests/fixtures/fonts/RHWPHostFixture${name === 'ttc' ? '.ttc' : `-${name}.ttf`}`, import.meta.url)).toString('base64')]));
  const result = await page.evaluate(async fixtures => {
    const { HostCanvasFontSession, withHostCanvasFonts } = await import('/src/core/host-canvas-fonts.ts');
    const { CanvasSupplementalMetricProvider } = await import('/src/core/supplemental-text-metrics.ts');
    const { setRawCanvasFont } = await import('/src/core/canvas-font-raw.ts');
    const local = await import('/src/core/local-fonts.ts');
    const decode = name => Uint8Array.from(atob(fixtures[name]), ch => ch.charCodeAt(0)).buffer;
    const session = new HostCanvasFontSession();
    const canvas = document.createElement('canvas'); canvas.width = 240; canvas.height = 100;
    const ctx = canvas.getContext('2d');
    const installed = () => [...document.fonts].filter(face => face.family.includes('__rhwp_host_face_')).length;
    const checks = [];
    let notify = () => {}, revision = 0, source = 'ttc', index = 0, slant = 'normal';
    const provider = {
      async getSnapshot() { return { revision: String(revision), faces: [{
        id: 'same-face', family: 'Host Canvas Face', fullName: 'Host Canvas Face Selected',
        postscriptName: 'HostCanvas-Selected', style: 'Selected', weight: 400, slant,
      }] }; },
      async readFace() { return { bytes: source === 'invalid' ? new Uint8Array([1, 2, 3]).buffer : decode(source),
        faceIndex: source === 'ttc' ? index : 0 }; },
      subscribe(listener) { notify = listener; return () => {}; },
    };
    await window.rhwpStudio.fonts.setProvider(provider);
    async function prepare() {
      await local.prepareHostFontCatalog();
      const record = local.resolveRendererLocalFont('Host Canvas Face', { weight: 400, slant: slant === 'normal' ? 'normal' : 'italic' });
      await session.prepare([record]);
    }
    function draw(font, raw = false) {
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      if (raw) setRawCanvasFont(ctx, font);
      else withHostCanvasFonts(session, () => { ctx.font = font; });
      ctx.fillStyle = '#111'; ctx.fillText('A가😀', 10, 70);
      return { image: canvas.toDataURL(), width: ctx.measureText('😀').width, descriptor: ctx.font };
    }
    for (const [style, faceIndex, units] of [['Regular', 0, 900], ['Italic', 1, 1080], ['Oblique', 2, 1200]]) {
      index = faceIndex; slant = faceIndex === 0 ? 'normal' : style.toLowerCase(); revision++; notify();
      await prepare();
      const host = draw(`${faceIndex === 0 ? '' : 'italic '}48px "Host Canvas Face"`);
      const reference = new FontFace(`reference-${style}`, decode(style)); await reference.load(); document.fonts.add(reference);
      const expected = draw(`48px "reference-${style}"`, true);
      const metrics = new CanvasSupplementalMetricProvider(); metrics.reset({ document: 1, fonts: revision });
      const measured = await metrics.prepare({ document: 1, fonts: revision }, [
        { key: 'emoji', cluster: '😀', font: `${faceIndex === 0 ? '' : 'italic '}48px "Host Canvas Face"` },
      ], () => document.fonts.ready, () => session.measurementContext(document.createElement('canvas').getContext('2d')));
      checks.push({ style, samePixels: host.image === expected.image, width: host.width,
        expectedWidth: 48 * units / 1000, referenceWidth: expected.width,
        measuredWidth: measured.metrics[0].measuredAdvancePx,
        aliasUsedByPaint: host.descriptor.includes('__rhwp_host_face_'),
        aliasUsedByMeasure: measured.metrics[0].resolvedFont.includes('__rhwp_host_face_'), installed: installed() });
      document.fonts.delete(reference);
    }
    source = 'invalid'; revision++; notify(); await prepare();
    const failed = session.diagnostics();
    source = 'ttc'; index = 0; slant = 'normal'; revision++; notify(); await prepare();
    const recovered = session.diagnostics();
    await window.rhwpStudio.fonts.setProvider(null);
    const detached = installed(); session.reset();
    return { checks, failed, recovered, detached };
  }, fixtures);
  for (const check of result.checks) {
    assert.equal(check.samePixels, true, `${check.style}: TTC Canvas2D output matches standalone FontFace without double slant`);
    assert.equal(check.width, check.referenceWidth);
    assert(Math.abs(check.width - check.expectedWidth) < 0.01, 'independent 1000-unit glyph advance');
    assert.equal(check.measuredWidth, check.width, 'measurement and drawing use the same actual face');
    assert.equal(check.aliasUsedByPaint, true); assert.equal(check.aliasUsedByMeasure, true);
    assert.equal(check.installed, 1, 'replacement releases the previous FontFace');
  }
  assert.equal(result.failed.loaded, 0); assert.equal(result.failed.failed, 1);
  assert.equal(result.recovered.loaded, 1); assert.equal(result.recovered.failed, 0);
  assert.equal(result.detached, 0);
  console.log('Canvas2D face and measurement contracts:', JSON.stringify(result));
}

async function verifyCanvas2DCharOverlap(page) {
  const fixture = readFileSync(new URL('../../samples/basic/issue2007_nested_cell_pagination_42065.hwp', import.meta.url)).toString('base64');
  const result = await page.evaluate(async fixture => {
    const wasm = window.__wasm;
    wasm.loadDocument(Uint8Array.from(atob(fixture), ch => ch.charCodeAt(0)), 'char-overlap.hwp');
    const raw = JSON.parse(wasm.doc.getPageLayerTree(9));
    const runs = [];
    const visit = value => {
      if (!value || typeof value !== 'object') return;
      if (value.type === 'textRun' && value.charOverlap && value.text?.startsWith('\u{F02B1}')) runs.push(value);
      for (const child of Object.values(value)) visit(child);
    };
    visit(raw);
    if (!runs.length) throw new Error('real boxed PUA fixture missing');
    const family = runs[0].style.fontFamily.split(',')[0].trim().replace(/^['"]|['"]$/g, '');
    const font = await (await fetch('/fonts/NotoSerifKR-Regular.woff2')).arrayBuffer();
    await window.rhwpStudio.fonts.setProvider({
      async getSnapshot() { return { revision: 'overlap', faces: [{ id: 'overlap', family,
        fullName: family, postscriptName: family, style: 'Regular', weight: 400, slant: 'normal' }] }; },
      async readFace() { return { bytes: font }; }, subscribe() { return () => {}; },
    });
    await wasm.prepareCanvasMetrics('canvas2d');
    const canvas = document.createElement('canvas');
    const calls = [];
    const original = CanvasRenderingContext2D.prototype.fillText;
    CanvasRenderingContext2D.prototype.fillText = function(text, x, y, ...rest) {
      calls.push({ text: String(text), x, y, font: this.font });
      return original.call(this, text, x, y, ...rest);
    };
    try { wasm.renderPageToCanvas(9, canvas, 1); }
    finally { CanvasRenderingContext2D.prototype.fillText = original; }
    const atTarget = calls.filter(call => runs.some(run => {
      const size = run.style.fontSize, height = run.bbox.height ?? run.bbox.h;
      return Math.abs(call.x - run.bbox.x - size / 2) <= 1
        && Math.abs(call.y - run.bbox.y - height + size / 2) <= 1;
    }));
    const result = { family, loaded: wasm.getHostCanvasFontDiagnostics().loaded,
      calls: atTarget, pageCount: wasm.pageCount };
    await window.rhwpStudio.fonts.setProvider(null);
    return result;
  }, fixture);
  assert.equal(result.pageCount, 17);
  assert(result.loaded > 0);
  assert(result.calls.some(call => call.text === '1' && call.font.includes('__rhwp_host_face_')),
    `Canvas2D charOverlap uses the supplied face at the real boxed numeral: ${JSON.stringify(result)}`);
  assert(!result.calls.some(call => call.text.includes('\u{F02B1}')));
  console.log('Canvas2D real charOverlap:', JSON.stringify(result));
}


// API/environment contract: fixture byte identity and policy, without layout goldens.
async function verifyLayoutEnvironment(page) {
  const fixtures = Object.fromEntries(['ttc', 'Regular', 'Italic'].map(name => [name,
    readFileSync(new URL(`../../tests/fixtures/fonts/RHWPHostFixture${name === 'ttc' ? '.ttc' : `-${name}.ttf`}`, import.meta.url)).toString('base64')]));
  const result = await page.evaluate(async fixtures => {
    const { WasmBridge } = await import('/src/core/wasm-bridge.ts');
    const { getLocalFontState } = await import('/src/core/local-fonts.ts');
    const w = new WasmBridge();
    const decode = key => Uint8Array.from(atob(fixtures[key]), c => c.charCodeAt(0)).buffer;
    const digest = async bytes => [...new Uint8Array(await crypto.subtle.digest('SHA-256', bytes))]
      .map(b => b.toString(16).padStart(2, '0')).join('');
    const family = 'Host Layout Contract';
    let source = 'Regular', index = 0, revision = 0, enabled = true, fail = false, split = false;
    let notify = () => {};
    const reads = [];
    const provider = {
      async getSnapshot() { return { revision: String(revision), faces: enabled ? [{ id: 'face',
        family, fullName: family, postscriptName: 'HostLayout-Selected', style: 'Regular', weight: 400, slant: 'normal' }, ...(split ? [{
          id: 'second', family, fullName: 'HostLayout-Second', postscriptName: 'HostLayout-Second',
          style: 'Regular', weight: 400, slant: 'normal' }] : [])] : [] }; },
      async readFace(id, rev) { reads.push({ id, rev, source, index }); if (fail) throw new Error('denied');
        return { bytes: decode(id === 'second' ? 'Italic' : source), faceIndex: index }; },
      subscribe(callback) { notify = callback; return () => {}; },
    };
    const originalLocal = JSON.stringify(getLocalFontState());
    const storage = localStorage.getItem('rhwp-local-fonts');
    const checks = [];
    function setup() {
      w.createNewDocument(); w.insertText(0, 0, 0, 'A가');
      const fontId = w.findOrCreateFontId(family);
      w.applyCharFormat(0, 0, 0, 2, JSON.stringify({ fontId }));
    }
    setup();
    const doc = w.borrowDocumentHandle();
    const environments = [];
    const originalSet = doc.setFontEnvironment.bind(doc);
    doc.setFontEnvironment = value => { environments.push(value ? JSON.parse(value) : null); return originalSet(value); };
    const originalBytes = [...w.exportHwp()];
    const plan = () => JSON.parse(doc.getDocumentFontSelectionPlan());
    async function prepare() { await w.prepareDeclaredFontEnvironment('canvas2d'); return environments.at(-1); }
    try {
      await rhwpStudio.fonts.setProvider(provider);
      await prepare(); checks.push(['omitted', environments.length === 0]);
      await rhwpStudio.fonts.setProvider(provider, { layout: false });
      await prepare(); checks.push(['false', environments.length === 0]);
      await rhwpStudio.fonts.setProvider(provider, { layout: true });
      const a = await prepare();
      const selected = env => JSON.stringify(env?.selectedPrograms ?? {});
      checks.push(['program-A', selected(a).includes(await digest(decode('Regular')))]);
      checks.push(['source-metrics', a.preserveSourceMetrics === true && !(a.replaceSourceMetricsFor?.length)]);
      split = true; revision++; notify(); const beforeSplit = environments.length; await prepare();
      checks.push(['split-query-identity', environments.length === beforeSplit + 1
        && selected(environments.at(-1)).includes(await digest(decode('Regular')))]);
      split = false;
      source = 'Italic'; revision++; notify(); const b = await prepare();
      checks.push(['same-id-new-bytes', selected(b).includes(await digest(decode('Italic'))) && selected(a) !== selected(b)]);
      source = 'ttc'; index = 0; revision++; notify(); const t0 = await prepare();
      index = 1; revision++; notify(); const t1 = await prepare();
      checks.push(['ttc-face', selected(t0).includes('-face-0') && selected(t1).includes('-face-1') && selected(t0) !== selected(t1)]);
      checks.push(['ttc-sha', selected(t1).includes(await digest(decode('ttc')))]);
      enabled = false; revision++; notify(); await prepare();
      checks.push(['remove', !JSON.stringify(environments.at(-1)).includes('host-')]);
      enabled = true; revision++; notify(); await prepare();
      checks.push(['restore', selected(environments.at(-1)) === selected(t1)]);
      await rhwpStudio.fonts.setProvider(provider, { layout: false }); await prepare();
      checks.push(['disable', !JSON.stringify(environments.at(-1)).includes('host-')]);
      await rhwpStudio.fonts.setProvider(provider, { layout: true }); await prepare();
      fail = true; revision++; notify(); await prepare();
      checks.push(['read-failure', !JSON.stringify(environments.at(-1)).includes('host-')]);
      fail = false; revision++; notify(); await prepare();
      await w.prepareDeclaredFontEnvironment('canvaskit');
      checks.push(['backend', !w.hasHostFontLayoutEnvironment && !JSON.stringify(environments.at(-1)).includes('host-')]);
      await prepare();
      const preferenceKey = 'rhwp-document-font-preferences';
      const oldPreference = localStorage.getItem(preferenceKey);
      try {
        localStorage.setItem(preferenceKey, JSON.stringify({ [w.documentDigest]: {
          documentDigest: w.documentDigest, substitutions: { [family]: family } } }));
        const chosen = await prepare();
        checks.push(['explicit-metric-override', chosen.preserveSourceMetrics === true
          && JSON.stringify(chosen.replaceSourceMetricsFor) === JSON.stringify([family])]);
      } finally {
        if (oldPreference === null) localStorage.removeItem(preferenceKey); else localStorage.setItem(preferenceKey, oldPreference);
      }
      checks.push(['saved-bytes', JSON.stringify([...w.exportHwp()]) === JSON.stringify(originalBytes)]);
      checks.push(['portable-names', !w.renderPageSvg(0).includes('__rhwp_host_face_')]);
      originalSet(JSON.stringify({ id: 'explicit-owner', substitutions: { [family]: family }, preserveSourceMetrics: true }));
      await rhwpStudio.fonts.setProvider(null); await prepare();
      checks.push(['explicit-owner', plan().environmentId === 'explicit-owner']);
      originalSet(undefined);
      for (const transition of ['disable', 'replace', 'document']) {
        setup();
        const pending = [];
        const slow = { ...provider, readFace: (_id, _rev, signal) => new Promise(resolve => pending.push({ resolve, signal })) };
        await rhwpStudio.fonts.setProvider(slow, { layout: true });
        const oldPreparation = w.prepareDeclaredFontEnvironment('canvas2d');
        const deadline = Date.now() + 5000;
        while (!pending.length && Date.now() < deadline) await new Promise(resolve => setTimeout(resolve, 0));
        if (!pending.length) throw new Error('layout read did not start');
        if (transition === 'document') setup();
        else await rhwpStudio.fonts.setProvider(provider, { layout: transition === 'replace' });
        if (transition === 'replace') await w.prepareDeclaredFontEnvironment('canvas2d');
        const expected = JSON.parse(w.borrowDocumentHandle().getDocumentFontSelectionPlan()).environmentId;
        pending.forEach(item => item.resolve({ bytes: decode('Regular'), faceIndex: 0 }));
        const staleResult = await oldPreparation;
        checks.push(['late-' + transition, staleResult === false
          && JSON.parse(w.borrowDocumentHandle().getDocumentFontSelectionPlan()).environmentId === expected]);
        w.borrowDocumentHandle().setFontEnvironment(undefined);
      }
      checks.push(['local-state', originalLocal === JSON.stringify(getLocalFontState())
        && storage === localStorage.getItem('rhwp-local-fonts')]);
      return { checks, reads };
    } finally { w.releaseDocument(); await rhwpStudio.fonts.setProvider(null); }
  }, fixtures);
  console.log('#8946 layout environment:', JSON.stringify(result));
  for (const [name, passed] of result.checks) assert.equal(passed, true, name);
  await createNewDocument(page);
  await clickEditArea(page);
  await typeText(page, 'A');
  await page.evaluate(async () => {
    const w = window.__wasm;
    w.applyCharFormat(0, 0, 0, 1, JSON.stringify({ fontId: w.findOrCreateFontId('Host Layout Contract') }));
    await window.__canvasView.loadDocument();
  });
  await typeText(page, 'A');
  await page.evaluate(() => {
    window.__inputHandler.performUndo();
    window.__documentState.markClean('layout-contract');
    const h = window.__inputHandler.history;
    window.__layout8946 = { bytes: window.__wasm.exportHwp(), undo: [...h.undoStack], redo: [...h.redoStack],
      length: window.__wasm.getParagraphLength(0, 0) };
  });
  await page.evaluate(async bytes => {
    const data = Uint8Array.from(atob(bytes), c => c.charCodeAt(0)).buffer;
    const face = { id: 'face', family: 'Host Layout Contract', fullName: 'Host Layout Contract',
      postscriptName: 'HostLayout-Selected', style: 'Regular', weight: 400, slant: 'normal' };
    window.__layout8946.provider = { async getSnapshot() { return { revision: 'stable', faces: [face] }; },
      async readFace() { return { bytes: data.slice(0), faceIndex: 0 }; }, subscribe() { return () => {}; } };
    await rhwpStudio.fonts.setProvider(window.__layout8946.provider, { layout: true });
  }, fixtures.Regular);
  await page.waitForFunction(() => window.__wasm.hasHostFontLayoutEnvironment
    && window.__wasm.getHostCanvasFontDiagnostics().loaded === 1);
  const history = await page.evaluate(async () => {
    const w = window.__wasm, saved = window.__layout8946, h = window.__inputHandler.history;
    const same = (a, b) => a.length === b.length && a.every((v, i) => v === b[i]);
    const result = { clean: !window.__documentState.isDirty(), bytes: same(saved.bytes, w.exportHwp()),
      undo: same(saved.undo, h.undoStack), redo: same(saved.redo, h.redoStack), redoCount: h.redoStack.length,
      environment: JSON.parse(w.borrowDocumentHandle().getDocumentFontSelectionPlan()).environmentId };
    window.__inputHandler.performRedo(); result.redone = w.getParagraphLength(0, 0) === saved.length + 1;
    window.__inputHandler.performUndo(); result.undone = w.getParagraphLength(0, 0) === saved.length;
    await rhwpStudio.fonts.setProvider(saved.provider, { layout: false });
    return result;
  });
  for (const key of ['clean', 'bytes', 'undo', 'redo', 'redone', 'undone']) assert.equal(history[key], true, key);
  assert(history.redoCount > 0);
  assert.match(history.environment, /^declared-program-/);
  await page.waitForFunction(() => !window.__wasm.hasHostFontLayoutEnvironment);
  console.log('#8946 enabled UI history:', JSON.stringify(history));
  await page.evaluate(async () => { await rhwpStudio.fonts.setProvider(null); delete window.__layout8946; });
}

const backend = process.argv.find(arg => arg.startsWith('--renderer='))?.split('=')[1] || process.env.RHWP_TEST_RENDERER || 'canvaskit';
assert(['canvaskit', 'canvas2d'].includes(backend));
runTest(`Issue #7403 ${backend}: host font identity, replacement and document preservation`, async ({ page }) => {
  await loadApp(page, `?renderer=${backend}&canvaskitSurface=software`);
  await page.evaluate(backend => {
    window.__hostFontDiagnostics = () => {
      if (backend === 'canvaskit') return window.__canvasView.getCurrentCanvasKitRenderDiagnostics();
      const state = window.__wasm.getHostCanvasFontDiagnostics();
      return { localTypefaceCount: state.loaded, localTypefacePendingCount: state.pending };
    };
  }, backend);
  if (backend === 'canvas2d') {
    await verifyCanvas2DFaces(page);
    await verifyCanvas2DCharOverlap(page);
    await verifyLayoutEnvironment(page);
    await createNewDocument(page);
  }
  else await verifyFacePaint(page);
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
      generation: wasm.documentGeneration, svg: wasm.renderPageSvg(0), properties: wasm.getCharPropertiesAt(0, 0, 3) };
    const before = document.querySelector('#scroll-container canvas').toDataURL();
    const count = window.__hostFontDiagnostics()?.localTypefaceCount;
    window.__host7403.replace('sans');
    return { before, count };
  });
  assert.equal(documentResult.count, 2, 'actual document prepares both host faces');
  await page.waitForFunction(before => {
    const diagnostics = window.__hostFontDiagnostics();
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
    return { dirty, generation, expectedGeneration: saved.generation, hwpUnchanged, roundtrips,
      svgUnchanged: saved.svg === wasm.renderPageSvg(0), svgHasAlias: wasm.renderPageSvg(0).includes('__rhwp_host_face_') };
  });
  assert.equal(preserved.dirty, true);
  assert.equal(preserved.generation, preserved.expectedGeneration);
  assert.equal(preserved.hwpUnchanged, true);
  assert.equal(preserved.svgUnchanged, true, 'getPageSvg handler retains portable font names and geometry');
  assert.equal(preserved.svgHasAlias, false);
  for (const roundtrip of preserved.roundtrips) {
    assert(roundtrip.fonts.includes('RHWP Host Smoke'), `${roundtrip.name} retains the document font name`);
    assert.equal(roundtrip.properties.bold, true);
    assert(!roundtrip.fonts.some(name => name.includes('face-') || name.includes('__rhwp_host_face_') || name.includes('host\"')));
  }
  await screenshot(page, `${backend}-host-font-7403-document`);
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
    const d = window.__hostFontDiagnostics();
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
    const countBefore = window.__hostFontDiagnostics()?.localTypefaceCount;
    for (const pending of race.pending) pending.resolve({ bytes: race.bytes });
    await new Promise(resolve => setTimeout(resolve, 150));
    const bytesAfter = window.__wasm.exportHwp();
    return { changedDocument: window.__wasm.documentGeneration !== race.generation,
      sameImage: before === document.querySelector('#scroll-container canvas').toDataURL(),
      sameBytes: bytesBefore.length === bytesAfter.length && bytesBefore.every((value, i) => value === bytesAfter[i]),
      countBefore, countAfter: window.__hostFontDiagnostics()?.localTypefaceCount };
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
  await page.waitForFunction(() => window.__hostFontDiagnostics()?.localTypefaceCount === 1);
  const replacedRace = await page.evaluate(async () => {
    const race = window.__host7403.race;
    const before = document.querySelector('#scroll-container canvas').toDataURL();
    const aborted = race.pending.every(item => item.signal.aborted);
    for (const item of race.pending) item.resolve({ bytes: race.bytes });
    await new Promise(resolve => setTimeout(resolve, 150));
    return { aborted, sameImage: before === document.querySelector('#scroll-container canvas').toDataURL(),
      count: window.__hostFontDiagnostics()?.localTypefaceCount };
  });
  assert.equal(replacedRace.aborted, true);
  assert.equal(replacedRace.sameImage, true);
  assert.equal(replacedRace.count, 1);
  console.log('Document lifecycle contracts:', JSON.stringify({ historyResult, switched, replacedRace }));

  await page.evaluate(async () => {
    window.__host7403.detach = {
      renderer: window.__canvasView.pageRenderer.canvaskitRenderer,
      bytes: window.__wasm.exportHwp(),
    };
    await window.rhwpStudio.fonts.setProvider(null);
  });
  if (backend === 'canvaskit') {
    // #7688 requires a prepared face. Removing the only Next Host source must
    // release it and diagnose the existing Canvas2D fallback, not keep a stale
    // CanvasKit face just to expose a zero-valued current-renderer counter.
    await page.waitForFunction(() => window.__renderBackend === 'canvas2d'
      && window.__host7403.detach.renderer.diagnostics().localTypefaceCount === 0);
    const session = await page.evaluate(() => window.__canvasView.getRendererSessionDiagnostics());
    assert.equal(session.fallbackReason, 'canvaskitResourcePreparationFailed');
    assert.match(session.selectionError, /Next Host/);
  } else {
    await page.waitForFunction(() => window.__hostFontDiagnostics()?.localTypefaceCount === 0);
  }
  const detached = await page.evaluate(() => {
    const before = window.__host7403.detach.bytes, after = window.__wasm.exportHwp();
    return { state: window.rhwpStudio.fonts.getState(),
      bytesUnchanged: before.length === after.length && before.every((value, i) => value === after[i]),
      pages: window.__wasm.pageCount };
  });
  assert.equal(detached.state.active, false);
  assert.equal(detached.state.count, 0);
  assert.equal(detached.bytesUnchanged, true);
  assert.equal(detached.pages, 1);
  console.log('Provider detach contracts:', JSON.stringify(detached));
  console.log(JSON.stringify({ result, preserved: { ...preserved, roundtrips: preserved.roundtrips.map(item => ({ name: item.name, fonts: item.fonts, bold: item.properties.bold })) } }, null, 2));
}, { skipLoadApp: true });
