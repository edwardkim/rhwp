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
