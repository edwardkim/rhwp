import fs from 'node:fs';
import {
  assert, closeBrowser, closePage, createPage, launchBrowser, loadApp, loadHwpFile,
} from './helpers.mjs';

// Original input / independent Print sweep prerequisite: #7688 stage5/6.
// Resource selection is verified through actual renderPage, not just readiness.
const browser = await launchBrowser();
const page = await createPage(browser);
try {
  await loadApp(page, '?renderer=canvaskit&canvaskitSurface=software');
  await loadHwpFile(page, 'group-drawing-02.hwp');
  const original = await page.evaluate(() => ({
    backend: window.__renderBackend,
    diagnostics: window.__canvasView.pageRenderer.getCurrentCanvasKitRenderDiagnostics(),
  }));
  assert(original.backend === 'canvaskit', 'real CanvasKit must render the original page');
  const failures = [];
  if (!(original.diagnostics?.bundledTypefaceCount > 0)) failures.push('document font bytes skipped in explicit mode');
  if (original.diagnostics?.unregisteredFontFallbacks !== 0) failures.push('original family silently became default Noto Sans KR');

  let styles;
  try { styles = await page.evaluate(async () => {
    const { CanvasKitLayerRenderer } = await import('/src/view/canvaskit-renderer.ts');
    const results = [];
    for (const profile of ['screen', 'print']) {
      const tree = window.__wasm.withPortableMetrics(() => window.__wasm.getPageLayerTreeObject(0, profile));
      const renderers = {};
      try {
        // Use real SFNT payloads. FontTools independently confirms OS/2 weights
        // 400/700; controls select each payload directly without synthetic bold.
        for (const [name, files] of Object.entries({
          pair: ['Regular', 'Bold'], reversed: ['Bold', 'Regular'],
          regular: ['Regular'], bold: ['Bold'],
        })) {
          const renderer = await CanvasKitLayerRenderer.create('default', 'software');
          renderers[name] = renderer;
          await renderer.prepareBundledFonts(files.map(file => ({
            url: `fonts/NotoSerifKR-${file}.woff2`, aliases: ['Audit Family'],
          })));
        }
        const paint = (renderer, bold, italic = false) => {
          const input = structuredClone(tree);
          const stack = [input.root];
          while (stack.length) {
            const node = stack.pop();
            if (node.kind === 'group') stack.push(...node.children);
            else if (node.kind === 'clipRect') stack.push(node.child);
            else for (const op of node.ops) {
              if (op.type === 'textRun' || op.type === 'charOverlap') {
                op.style = { ...op.style, fontFamily: 'Audit Family', bold, italic };
              }
            }
          }
          const canvas = document.createElement('canvas');
          canvas.width = Math.ceil(input.pageWidth); canvas.height = Math.ceil(input.pageHeight);
          const output = renderer.renderPage(input, canvas, 1);
          return output.toDataURL('image/png');
        };
        const regular = paint(renderers.regular, false);
        const bold = paint(renderers.bold, false);
        if (regular === bold) throw Error('independent real Regular/Bold outline controls are identical');
        for (const name of ['pair', 'reversed']) {
          const actualRegular = paint(renderers[name], false);
          const actualBold = paint(renderers[name], true);
          if (actualRegular !== regular) throw Error(`${profile}/${name}: Regular did not consume its supplied outline`);
          if (actualBold !== bold) throw Error(`${profile}/${name}: Bold used the wrong face or synthesized bold twice`);
          if (paint(renderers[name], false, true) !== paint(renderers.regular, false, true)) {
            throw Error(`${profile}/${name}: absent italic face did not preserve synthetic italic`);
          }
          if (paint(renderers[name], true, true) !== paint(renderers.bold, false, true)) {
            throw Error(`${profile}/${name}: bold italic doubled the available real Bold outline`);
          }
        }
        for (const renderer of Object.values(renderers)) {
          if (!renderer.hasPreparedFontFamily('Audit Family')) {
            throw Error('an actually prepared family absent from the portable catalog was rejected');
          }
          if (renderer.hasPreparedFontFamily('Missing Audit Family')) {
            throw Error('an unprepared family was incorrectly reported available');
          }
        }
        results.push({ profile, regularAndBoldDiffer: true, bothLoadOrdersMatchDirectFaceControls: true, syntheticItalicPreserved: true });
      } finally {
        for (const renderer of Object.values(renderers)) renderer.dispose();
      }
    }
    return results;
  }); } catch (error) { failures.push(error.message); }
  console.log(JSON.stringify({ original, styles, failures }));
  assert(failures.length === 0, failures.join("; ") || 'document fonts and both style load orders consume the actual supplied outlines');

  // 두 실제 cmap이 서로 다른 문자를 공급한다. 이는 자원 소비 계약이며 한컴 배치의 정답지는 아니다.
  const glyphFallback = await page.evaluate(async () => {
    const { CanvasKitLayerRenderer } = await import('/src/view/canvaskit-renderer.ts');
    const { setHostFontProvider } = await import('/src/core/local-fonts.ts');
    const { collectHostFontRequests } = await import('/src/core/host-font-requests.ts');
    const primary = 'Noto Serif KR', fallback = 'Noto Sans Symbols2';
    const payloads = await Promise.all(['NotoSerifKR-Regular.woff2', 'NotoSansSymbols2-Regular.ttf']
      .map(async file => (await fetch(`fonts/${file}`)).arrayBuffer()));
    await setHostFontProvider({
      getSnapshot: async () => ({ revision: 'ordinary-glyph-fallback', faces: [primary, fallback].map((family, index) => ({
        id: String(index), family, fullName: family, postscriptName: family,
        style: 'Regular', weight: 400, slant: 'normal',
      })) }),
      readFace: async id => ({ bytes: payloads[Number(id)].slice(0), faceIndex: 0 }), subscribe: () => () => {},
    });
    const renderer = await CanvasKitLayerRenderer.create('default', 'software', { requirePreparedFontFamilies: true });
    try {
      const run = (text, family, x, positions) => ({ type: 'textRun', text,
        bbox: { x, y: 10, width: positions.at(-1), height: 40 }, baseline: 32, positions,
        style: { fontFamily: family, fontSize: 30, color: '#000000' } });
      const tree = ops => ({ pageWidth: 130, pageHeight: 60,
        root: { kind: 'leaf', bounds: { x: 0, y: 0, width: 130, height: 60 }, ops } });
      const input = tree([run('가🖨나', `${primary}, ${fallback}`, 10, [0, 30, 60, 90])]);
      await renderer.prepareHostFonts(collectHostFontRequests(input));
      const paint = input => renderer.renderPage(input, document.createElement('canvas'), 1).toDataURL('image/png');
      const actual = paint(input);
      const diagnostics = renderer.diagnostics();
      const expected = paint(tree([run('가', primary, 10, [0, 30]), run('🖨', fallback, 40, [0, 30]), run('나', primary, 70, [0, 30])]));
      if (actual !== expected) throw Error('혼합 런이 독립 글리프 공급 face와 원래 위치를 소비하지 않았다');
      if (!diagnostics.fontSubstitutions.some(row => row.source === 'missingGlyphFamily' && row.resolvedFamily === fallback)) {
        throw Error('실제 그리기의 글리프 대체 face를 관측하지 못했다');
      }
      return { mixedRunMatchesIndependentFaces: true, producerPositionsPreserved: true };
    } finally { renderer.dispose(); await setHostFontProvider(null); }
  });
  assert(glyphFallback.mixedRunMatchesIndependentFaces && glyphFallback.producerPositionsPreserved,
    'ordinary pictographs preserve surrounding Hangul and consume the declared fallback program');

  // #8109: exercise the real replay with a supplied, distinct serif program.
  // A missing primary must retain the document's second candidate. Compare
  // against selecting that exact face directly, not the renderer's default.
  const documentSubstitute = await page.evaluate(async input => {
    const { CanvasKitLayerRenderer } = await import('/src/view/canvaskit-renderer.ts');
    const { setHostFontProvider } = await import('/src/core/local-fonts.ts');
    const { collectHostFontRequests } = await import('/src/core/host-font-requests.ts');
    const bytes = await (await fetch('fonts/NotoSerifKR-Regular.woff2')).arrayBuffer();
    const family = 'Noto Serif KR';
    await setHostFontProvider({
      getSnapshot: async () => ({ revision: 'issue8109-serif', faces: [{
        id: 'serif', family, fullName: family, postscriptName: 'NotoSerifKR-Regular',
        style: 'Regular', weight: 400, slant: 'normal',
      }] }),
      readFace: async () => ({ bytes: bytes.slice(0), faceIndex: 0 }),
      subscribe: () => () => {},
    });
    const renderer = await CanvasKitLayerRenderer.create('default', 'software', { requirePreparedFontFamilies: true });
    try {
      const core = window.__wasm.getWasmModuleExports().HwpDocument.importPdf(new Uint8Array(input), {});
      const source = JSON.parse(core.getPageLayerTreeWithProfile(0, 'print', false, false));
      core.free();
      const withFamily = name => {
        const tree = structuredClone(source), stack = [tree.root];
        while (stack.length) {
          const node = stack.pop();
          if (node.kind === 'group') stack.push(...node.children);
          else if (node.kind === 'clipRect' || node.kind === 'clipPath') stack.push(node.child);
          else for (const op of node.ops) if (op.type === 'textRun' || op.type === 'charOverlap') {
            op.style = { ...op.style, fontFamily: name, bold: false, italic: false };
          }
        }
        return tree;
      };
      const fallbackTree = withFamily(`Missing Document Primary, ${family}`);
      const requests = collectHostFontRequests(fallbackTree);
      if (requests.length !== 1 || requests[0].family !== family) throw Error('document substitute lost before font preparation');
      await renderer.prepareHostFonts(requests);
      const paint = tree => {
        const canvas = document.createElement('canvas');
        return renderer.renderPage(tree, canvas, 1).toDataURL('image/png');
      };
      const direct = paint(withFamily(family));
      if (paint(fallbackTree) !== direct) throw Error('document substitute used another physical face');
      let missingRejected = false;
      try { paint(withFamily('Missing Document Primary')); }
      catch (error) {
        if (!String(error).includes('font family가 준비되지 않았습니다')) throw error;
        missingRejected = true;
      }
      if (!missingRejected) throw Error('a same-name primary without a substitute reused the other chain');
      if (paint(withFamily(`${family}, Missing Document Primary`)) !== direct) {
        throw Error('an available primary stopped taking precedence');
      }
      return { actualReplayMatchesDirectSerifFace: true, availablePrimaryPreserved: true };
    } finally { renderer.dispose(); await setHostFontProvider(null); }
  }, Array.from(fs.readFileSync(new URL('../../pdf/issue7998/body-list.pdf', import.meta.url))));
  assert(documentSubstitute.actualReplayMatchesDirectSerifFace && documentSubstitute.availablePrimaryPreserved,
    'document fallback and available primary consume the same independently supplied serif program');

  // Exercise the actual later Local Font Access refresh callback too. The
  // controlled snapshot gives the supplied serif face an Arial alias; this is
  // a resource-selection contract, not a claim of Arial outline fidelity.
  const latePage = await createPage(browser);
  try {
    await loadApp(latePage, '?renderer=canvaskit&canvaskitSurface=software');
    await latePage.evaluate(async input => {
      const local = await import('/src/core/local-fonts.ts');
      const storageKey = 'rhwp-local-fonts';
      const saved = localStorage.getItem(storageKey);
      const queryDescriptor = Object.getOwnPropertyDescriptor(window, 'queryLocalFonts');
      window.__issue8109Late = { saved, queryDescriptor, storageKey };
      localStorage.removeItem(storageKey);
      await local.loadStoredLocalFonts();
      const wasm = window.__wasm;
      const bytes = new Uint8Array(input);
      const bus = window.__inputHandler.eventBus;
      await new Promise((resolve, reject) => {
        const off = bus.on('open-document-bytes:done', result => {
          if (result.requestId !== 'issue8109-late') return;
          off(); if (result.ok) resolve(); else reject(Error(result.error));
        });
        bus.emit('open-document-bytes', { bytes, fileName: 'issue8109-late.hwp',
          fileHandle: null, skipUnsavedGuard: true, requestId: 'issue8109-late' });
      });
      const info = wasm.getDocumentInfo();
      // fontsUsed는 실제 paint 준비 목록이라 #8191 이후 대체 Arial도 포함한다.
      // 원본에 Arial 별도 선언이 없는 경계는 원본 선언 계획으로 확인한다.
      const { HwpDocument } = wasm.getWasmModuleExports();
      const declarationCore = new HwpDocument(bytes);
      let declarations;
      try { declarations = JSON.parse(declarationCore.getDocumentFontSelectionPlan()); }
      finally { declarationCore.free(); }
      if (declarations.requests.length !== 7 || declarations.requests.some(request =>
        request.source !== 'Helvetica' || request.primary.length !== 1
          || request.primary[0] !== 'Helvetica' || request.substitute.length !== 1
          || request.substitute[0] !== 'Arial')) {
        throw Error('the counterexample must have only seven source Helvetica declarations with Arial substitutes');
      }
      if (info.fontsUsed.length !== 2 || !info.fontsUsed.includes('Helvetica')
        || !info.fontsUsed.includes('Arial')) {
        throw Error('the preparation list must include the source and its actual paint substitute');
      }
      if (!info.fontSubstitutions.some(([source, target]) => source === 'Helvetica' && target === 'Arial')) {
        throw Error('the imported document lost its substitute declaration');
      }
      if (window.__renderBackend !== 'canvaskit') throw Error('the initial real CanvasKit page is missing');
      const before = document.querySelector('#scroll-container canvas').toDataURL();
      const originalBytes = Array.from(wasm.exportHwp());
      const program = await (await fetch('fonts/NotoSerifKR-Regular.woff2')).arrayBuffer();
      const kit = window.__canvasView.pageRenderer.canvaskitRenderer.canvasKit;
      const manager = kit.FontMgr.FromData(program.slice(0));
      const expectedNativeFamily = manager.getFamilyName(0); manager.delete();
      const family = 'Noto Serif KR', fullName = family + ' Regular';
      const face = { family, fullName, postscriptName: 'NotoSerifKR-Regular', style: 'Regular' };
      Object.defineProperty(window, 'queryLocalFonts', { configurable: true, value: async () => [{
        ...face, blob: async () => new Blob([program]),
      }] });
      localStorage.setItem(storageKey, JSON.stringify({ version: 2,
        detectedAt: '2026-10-21T00:00:00.000Z', families: [family], source: 'local-font-access',
        fontRecords: [{ ...face, displayName: family, aliases: [family, fullName, 'Arial'],
          detectionSource: 'local-font-access' }],
      }));
      await local.loadStoredLocalFonts();
      Object.assign(window.__issue8109Late, { before, originalBytes, family, expectedNativeFamily });
      window.__inputHandler.eventBus.emit('local-fonts-changed');
    }, Array.from(fs.readFileSync(new URL('../../tests/fixtures/issue_8109_saved_fonts/only-source-font.hwp', import.meta.url))));
    await latePage.waitForFunction(() => {
      const d = window.__canvasView.getCurrentCanvasKitRenderDiagnostics();
      return d?.localTypefaceCount === 1 && d.localTypefacePendingCount === 0
        && document.querySelector('#scroll-container canvas').toDataURL() !== window.__issue8109Late.before;
    });
    const late = await latePage.evaluate(async () => {
      const { enrichFontDecisionTrace } = await import('/src/core/font-decision-trace.ts');
      const trace = enrichFontDecisionTrace(window.__wasm.getFontDecisionTrace(0, 512), {
        canvasKitEvidence: record => window.__canvasView.getCanvasKitFontDecisionEvidence(0, record),
      });
      const current = window.__wasm.exportHwp(), saved = window.__issue8109Late.originalBytes;
      return { backend: window.__renderBackend, pages: window.__wasm.pageCount,
        resolvedFamilies: [...new Set(trace.records.map(record => record.paint.canvaskit.resolved))],
        requestedFamilies: [...new Set(trace.records.map(record => record.paint.canvaskit.requested))],
        usesSuppliedSerif: trace.records.length > 0
          && trace.records.every(record => record.paint.canvaskit.resolved === window.__issue8109Late.expectedNativeFamily),
        bytesUnchanged: current.length === saved.length && current.every((value, i) => value === saved[i]) };
    });
    assert(late.backend === 'canvaskit' && late.pages === 1 && late.usesSuppliedSerif && late.bytesUnchanged,
      'later local-font refresh consumes the document substitute and preserves the document');
    console.log(JSON.stringify({ lateLocalFontRefresh: late }));
    const preparationWork = await latePage.evaluate(async () => {
      const wasm = window.__wasm, options = window.__canvasView.rendererSession.options;
      const original = wasm.getPageLayerTreeObject;
      let calls = 0;
      wasm.getPageLayerTreeObject = function (...args) { calls++; return original.apply(this, args); };
      try {
        const report = wasm.getCanvasKitDocumentPreflight();
        const incomplete = { ...report, complete: false };
        const rejected = options.transformCanvasKitPreflight(incomplete);
        const incompleteCalls = calls;
        const transformed = options.transformCanvasKitPreflight(report);
        const transformCalls = calls - incompleteCalls;
        const prepared = [];
        await options.prepareCanvasKitDocument({
          prepareLocalFonts: async families => { prepared.push(...families); },
          prepareBundledFonts: async () => {}, hasPreparedFontFamily: () => true,
        }, transformed);
        const sharedCalls = calls;
        options.transformCanvasKitPreflight(wasm.getCanvasKitDocumentPreflight());
        return { incompleteCalls, incompletePreserved: rejected === incomplete,
          transformCalls, sharedCalls, nextDecisionCalls: calls - sharedCalls,
          pages: wasm.pageCount, prepared };
      } finally { wasm.getPageLayerTreeObject = original; }
    });
    assert(preparationWork.incompletePreserved && preparationWork.incompleteCalls === 0,
      'incomplete preflight must fall back without exporting any page trees');
    assert(preparationWork.transformCalls === preparationWork.pages
      && preparationWork.sharedCalls === preparationWork.pages,
      'transform and resource preparation share one pass of document font declarations');
    assert(preparationWork.nextDecisionCalls === preparationWork.pages,
      'a new preflight decision must collect fresh declarations');
    assert(preparationWork.prepared.length > 0, 'sharing declarations must still prepare document fonts');
    console.log(JSON.stringify({ documentFontPreparationWork: preparationWork }));

  } finally {
    await latePage.evaluate(async () => {
      const saved = window.__issue8109Late;
      if (!saved) return;
      if (saved.saved === null) localStorage.removeItem(saved.storageKey);
      else localStorage.setItem(saved.storageKey, saved.saved);
      if (saved.queryDescriptor) Object.defineProperty(window, 'queryLocalFonts', saved.queryDescriptor);
      else delete window.queryLocalFonts;
      const { loadStoredLocalFonts } = await import('/src/core/local-fonts.ts');
      await loadStoredLocalFonts();
    });
    await closePage(latePage);
  }

  const blockedPage = await createPage(browser);
  try {
    await blockedPage.setCacheEnabled(false);
    await blockedPage.setRequestInterception(true);
    blockedPage.on('request', request => {
      // Block just the original document's declared bundled font. Default
      // CanvasKit resources still initialize; this is a preparation failure.
      if (request.url().includes('/HANBatang.woff')) void request.abort();
      else void request.continue();
    });
    await loadApp(blockedPage, '?renderer=canvaskit&canvaskitSurface=software');
    await loadHwpFile(blockedPage, 'group-drawing-02.hwp');
    const unavailable = await blockedPage.evaluate(() => ({
      backend: window.__renderBackend,
      diagnostics: window.__canvasView.getRendererSessionDiagnostics(),
      pages: window.__wasm.pageCount,
    }));
    assert(unavailable.backend === 'canvas2d' && unavailable.pages === 1,
      'failed document font preparation must preserve the page through Canvas2D');
    assert(unavailable.diagnostics.fallbackReason === 'canvaskitResourcePreparationFailed',
      'resource failure must be diagnosed instead of silently painting a wrong default face');
    console.log(JSON.stringify({ unavailable }));
  } finally { await closePage(blockedPage); }
} finally {
  await closePage(page); await closeBrowser(browser);
}
