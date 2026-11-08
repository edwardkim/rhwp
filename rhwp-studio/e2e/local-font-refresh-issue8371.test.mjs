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
});
