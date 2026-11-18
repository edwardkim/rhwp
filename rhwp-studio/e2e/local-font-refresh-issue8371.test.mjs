// 관계형: 재감지 버튼의 갱신 범위·원본 face 분리·이전 CSS 별칭 해제를 검사한다.
// Native/fresh WASM 일치율: 비해당. 사설 글꼴·실물 문서의 px 위치를 고정하지 않는다.
// 주석 갱신: 2026-11-18 21:39:28 KST.
import { assert, runTest, setTestCase, waitForCanvas } from './helpers.mjs';
import path from 'node:path';

runTest('문서 글꼴 변경 시에만 재조판', async ({ page }) => {
  const input = await page.$('#file-input');
  await input.uploadFile(path.resolve('../samples/eq-002.hwp'));
  await page.waitForFunction(() => window.__wasm?.pageCount > 0, { timeout: 30000 });
  await waitForCanvas(page, 30000);
  setTestCase('문서와 무관한 글꼴 및 문서 글꼴 감지');
  const font = await page.evaluate(async () => {
    const bridge = window.__wasm;
    const used = bridge.getDocumentInfo().fontsUsed[0];
    if (!used) throw new Error('문서 글꼴이 없습니다');
    window.__refreshCount = 0;
    const refresh = bridge.refreshLayout.bind(bridge);
    bridge.refreshLayout = () => { window.__refreshCount++; return refresh(); };
    window.__testFontRecords = [];
    Object.defineProperty(globalThis, 'queryLocalFonts', {
      configurable: true, value: async () => window.__testFontRecords,
    });
    const { OptionsDialog } = await import('/src/ui/options-dialog.ts');
    new OptionsDialog(window.__eventBus).show();
    return used;
  });
  const detect = async (family) => {
    await page.evaluate(name => {
      window.__testFontRecords = [{ family: name, fullName: name,
        postscriptName: `${name}-Regular`, style: 'Regular' }];
      document.querySelector('.opt-local-actions button').click();
    }, family);
    await page.waitForFunction(() => !document.querySelector('.opt-local-actions button').disabled);
    await new Promise(resolve => setTimeout(resolve, 500));
    return page.evaluate(() => window.__refreshCount);
  };
  const unrelated = await detect('Unrelated Issue8371 Font');
  const related = await detect(font);
  assert(unrelated === 0, '문서와 무관한 글꼴은 재조판하지 않는다');
  assert(related === 1, '문서에서 쓰는 글꼴이 변경되면 재조판한다');

  setTestCase('HY·한양 재감지와 이전 CSS 별칭 해제');
  const names = ['중고딕', '신명조', '견고딕', '견명조', '그래픽', '궁서']
    .flatMap(suffix => [`HY${suffix}`, `한양${suffix}`]);
  await page.evaluate(async families => {
    const { registerLoadedLocalFontFace } = await import('/src/core/font-loader.ts');
    const response = await fetch('/fonts/NotoSansKR-Regular.woff2');
    if (!response.ok) throw new Error('공개 검사 글꼴을 읽을 수 없습니다');
    const stale = new FontFace('이전 감지 별칭', await response.arrayBuffer());
    await stale.load(); registerLoadedLocalFontFace(stale);
    window.__staleLocalFontFace = stale;
    if (!document.fonts.has(stale)) throw new Error('이전 별칭 재현 실패');
    window.__testFontRecords = families.map(family => ({ family, fullName: family,
      postscriptName: `${family}-Original`, style: 'Regular' }));
    document.querySelector('.opt-local-actions button').click();
  }, names);
  await page.waitForFunction(() => !document.querySelector('.opt-local-actions button').disabled);
  const distinct = await page.evaluate(families => {
    // 개발 서버의 HMR 쿼리가 붙은 실제 앱 인스턴스를 사용한다.
    // 새 bare import를 만들면 별도 모듈의 빈 캐시를 검사하게 된다.
    return {
      staleRemoved: !document.fonts.has(window.__staleLocalFontFace),
      selections: families.map(name => window.__localFonts.resolveLocalFont(name)?.postscriptName),
    };
  }, names);
  console.log('  재감지 결과:', JSON.stringify(distinct));
  assert(distinct.staleRemoved, '재감지 시 이전 목록의 표시용 CSS 별칭도 해제한다');
  assert(distinct.selections.every((selected, i) => selected === `${names[i]}-Original`),
    'HY와 한양을 별개 face로 수집하고 조회한다');
});
