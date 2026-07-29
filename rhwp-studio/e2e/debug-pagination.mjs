/**
 * E2E 디버그: 50줄 입력 후 페이지네이션 확인
 */
import { runTest, createNewDocument, clickEditArea } from './helpers.mjs';

runTest('디버그: 페이지네이션', async ({ page }) => {
  await createNewDocument(page);
  await clickEditArea(page);

  const before = await page.evaluate(() => ({
    pageCount: window.__wasm?.pageCount,
    paraCount: window.__wasm?.getParagraphCount(0),
  }));
  console.log('Before:', before);

  for (let i = 0; i < 50; i++) {
    await page.keyboard.type('Line ' + i, { delay: 5 });
    await page.keyboard.press('Enter');
  }
  await page.evaluate(() => new Promise(r => setTimeout(r, 1000)));

  const after = await page.evaluate(() => ({
    pageCount: window.__wasm?.pageCount,
    paraCount: window.__wasm?.getParagraphCount(0),
    canvasCount: document.querySelectorAll('#scroll-container canvas').length,
    scrollH: document.querySelector('#scroll-container')?.scrollHeight,
  }));
  console.log('After:', after);
});
