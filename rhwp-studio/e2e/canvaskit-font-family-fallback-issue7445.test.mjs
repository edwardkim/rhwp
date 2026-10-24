import {
  assert, closeBrowser, closePage, createPage, launchBrowser, loadApp,
} from './helpers.mjs';

// #7445: U+2024 in the court form has no Gulim glyph but has an explicitly
// declared Haansoft Dotum fallback. Public SFNT controls reproduce that coverage
// boundary without redistributing an installed font or pinning document pixels.
const browser = await launchBrowser();
const page = await createPage(browser);
try {
  await loadApp(page, '?renderer=canvaskit&canvaskitSurface=software');
  const results = await page.evaluate(async () => {
    const { CanvasKitLayerRenderer } = await import('/src/view/canvaskit-renderer.ts');
    const renderer = await CanvasKitLayerRenderer.create('default', 'software', {
      requirePreparedFontFamilies: true,
    });
    try {
      await renderer.prepareBundledFonts([
        { url: 'fonts/NotoSansKR-Regular.woff2', aliases: ['Primary Coverage'] },
        { url: 'fonts/Pretendard-Regular.woff2', aliases: ['Declared Fallback'] },
        { url: 'fonts/NanumGothic-Regular.woff2', aliases: ['Earlier Fallback'] },
      ]);
      const paint = (family, text, profile, type = 'textRun') => {
        const bounds = { x: 0, y: 0, width: 160, height: 80 };
        const tree = {
          pageIndex: 0, pageWidth: 160, pageHeight: 80, profile,
          root: { kind: 'leaf', bounds, ops: [{
            type, text,
            bbox: { x: 20, y: 20, width: 24, height: 30 },
            baseline: 24, positions: [0, 24],
            style: { fontFamily: family, fontSize: 24, color: '#000000' },
            ...(type === 'charOverlap' ? { rotation: 0, isVertical: false,
              positionsComplete: true, charOverlap: { borderType: 1, innerCharSize: 0 } } : {}),
          }] },
        };
        const canvas = document.createElement('canvas');
        canvas.width = 160; canvas.height = 80;
        const png = renderer.renderPage(tree, canvas, 1).toDataURL('image/png');
        return { png, diagnostics: renderer.diagnostics() };
      };
      const results = [];
      for (const profile of ['screen', 'print']) {
        const direct = paint('Declared Fallback', '․', profile);
        const mixed = paint('"Primary Coverage", "Declared Fallback"', '․', profile);
        if (mixed.diagnostics.lastUnsupportedOps.length !== 0 || mixed.png !== direct.png) {
          throw Error(`${profile}: declared secondary face did not paint its supplied U+2024 outline: ${JSON.stringify(mixed.diagnostics)}`);
        }
        if (!mixed.diagnostics.fontSubstitutions.some(item => item.source === 'missingGlyphFamily')) {
          throw Error(`${profile}: declared family ownership was not reported`);
        }
        const primary = paint('Primary Coverage', 'A', profile);
        if (paint('Primary Coverage, Declared Fallback', 'A', profile).png !== primary.png) {
          throw Error(`${profile}: a covered primary glyph was replaced`);
        }
        const earlier = paint('Earlier Fallback', '❖', profile);
        if (paint('Primary Coverage, Earlier Fallback, Declared Fallback', '❖', profile).png !== earlier.png) {
          throw Error(`${profile}: declared family order was lost`);
        }
        const absent = paint('Primary Coverage, Never Prepared', '․', profile);
        if (!absent.diagnostics.lastUnsupportedOps.includes('textRun:glyphMapping')) {
          throw Error(`${profile}: missing glyph support was incorrectly accepted`);
        }
        const overlap = paint('Primary Coverage, Declared Fallback', '․', profile, 'charOverlap');
        const overlapDirect = paint('Declared Fallback', '․', profile, 'charOverlap');
        if (overlap.diagnostics.lastUnsupportedOps.length !== 0 || overlap.png !== overlapDirect.png) {
          throw Error(`${profile}: enclosed text did not consume the declared face`);
        }
        results.push({ profile, secondaryOutlineMatchesDirectFace: true,
          primaryOwnershipPreserved: true, declaredOrderPreserved: true,
          unavailableFallbackStillDiagnosed: true, charOverlapDeclaredFaceMatchesDirect: true });
      }
      return results;
    } finally { renderer.dispose(); }
  });
  console.log(JSON.stringify({ results }));
  assert(results.length === 2, 'screen/print must consume the declared family chain');
} finally {
  await closePage(page); await closeBrowser(browser);
}
