// 관계형: 표시 이름이 같은 두 face도 고유 PostScript 식별자로 준비하고 실제 윤곽을 소비한다.
// 실물 문서의 px 고정 검사가 아닌 합성 자원 계약이다. 저장소의 공개 글꼴만 사용한다.
import { assert, closeBrowser, closePage, createPage, launchBrowser, loadApp } from './helpers.mjs';

const browser = await launchBrowser();
const page = await createPage(browser);
try {
  await loadApp(page, '?renderer=canvaskit&canvaskitSurface=software');
  const result = await page.evaluate(async () => {
    const local = await import('/src/core/local-fonts.ts');
    const { CanvasKitLayerRenderer } = await import('/src/view/canvaskit-renderer.ts');
    await local.setHostFontProvider(null);
    const sources = [
      { family: 'Noto Serif KR', postscriptName: 'NotoSerifKR-Regular', file: 'NotoSerifKR-Regular.woff2' },
      { family: 'Noto Sans Symbols2', postscriptName: 'NotoSansSymbols2-Regular', file: 'NotoSansSymbols2-Regular.ttf' },
    ];
    const bytes = await Promise.all(sources.map(async source => (await fetch(`fonts/${source.file}`)).arrayBuffer()));
    // 서로 다른 실제 프로그램에 같은 표시 이름을 부여한 Local Font Access 대조군이다.
    globalThis.queryLocalFonts = async options => sources.flatMap((source, index) =>
      options?.postscriptNames && !options.postscriptNames.includes(source.postscriptName) ? [] : [{
        family: source.family, fullName: '000 Shared Display Name', postscriptName: source.postscriptName,
        style: 'Regular', blob: async () => new Blob([bytes[index]]),
      }]);
    await local.detectLocalFonts({ force: true, includeRegistered: true });
    if (local.resolveLocalFont('000 Shared Display Name') !== null) throw Error('모호한 표시 이름을 임의로 선택했다');
    const renderer = await CanvasKitLayerRenderer.create('default', 'software', { requirePreparedFontFamilies: true });
    const direct = await CanvasKitLayerRenderer.create('default', 'software', { requirePreparedFontFamilies: true });
    try {
      const count = await renderer.prepareLocalFonts(sources.map(source => source.family));
      if (count !== 2) throw Error(`고유 face 두 개가 준비되어야 한다: actual=${count}`);
      for (const source of sources) if (!renderer.hasPreparedFontFamily(source.family)) throw Error(`준비 누락: ${source.family}`);
      await direct.prepareBundledFonts(sources.map(source => ({ url: `fonts/${source.file}`, aliases: [source.family] })));
      const run = (text, fontFamily, x) => ({ type: 'textRun', text, bbox: { x, y: 10, width: 30, height: 40 },
        baseline: 32, positions: [0, 30], style: { fontFamily, fontSize: 30, color: '#000000' } });
      const tree = { pageWidth: 100, pageHeight: 60, root: { kind: 'leaf', bounds: { x: 0, y: 0, width: 100, height: 60 },
        ops: [run('가', sources[0].family, 10), run('🖨', sources[1].family, 50)] } };
      const paint = target => target.renderPage(tree, document.createElement('canvas'), 1).toDataURL('image/png');
      if (paint(renderer) !== paint(direct)) throw Error('준비한 두 face의 윤곽이 독립 직접 공급과 다르다');
      return { preparedFaces: count, sameDisplayNameRemainsAmbiguous: true, actualReplayMatchesDirectPrograms: true };
    } finally { renderer.dispose(); direct.dispose(); }
  });
  console.log(JSON.stringify(result));
  assert(result.preparedFaces === 2 && result.actualReplayMatchesDirectPrograms, '표시 이름이 같아도 실제 두 face를 유지한다');
} finally { await closePage(page); await closeBrowser(browser); }
