import { assert, clickEditArea, createNewDocument, loadApp, runTest, setTestCase, typeText } from './helpers.mjs';
import path from 'node:path';

runTest('초기화 후 문서 글꼴 사용 선택', async ({ page }) => {
  await page.evaluateOnNewDocument(() => Object.defineProperty(globalThis, 'queryLocalFonts', {
    configurable: true,
    value: async () => { window.__fontQueries = (window.__fontQueries ?? 0) + 1; return []; },
  }));
  await loadApp(page);
  await page.evaluate(async () => {
    const fonts = await import('/src/core/local-fonts.ts');
    await fonts.clearStoredLocalFonts();
  });
  setTestCase('빈 문서는 글꼴 선택창 없이 첫 입력과 문단 나누기가 가능하다');
  await createNewDocument(page);
  const blankDialog = await page.evaluate(() => [...document.querySelectorAll('.modal-overlay button')]
    .some(button => button.textContent?.includes('대체 글꼴로 보기')));
  assert(!blankDialog, '새 빈 문서에는 원본 글꼴 선택창을 띄우지 않는다');
  await clickEditArea(page);
  await typeText(page, 'first');
  await page.keyboard.press('Enter');
  await typeText(page, 'second');
  const blank = await page.evaluate(() => ({
    count: window.__wasm.getParagraphCount(0),
    first: window.__wasm.getParagraphLength(0, 0),
    second: window.__wasm.getParagraphLength(0, 1),
  }));
  assert(blank.count === 2 && blank.first >= 5 && blank.second >= 6,
    `첫 입력과 문단 나누기가 보존된다: ${JSON.stringify(blank)}`);
  await createNewDocument(page);
  setTestCase('문서를 열면 로컬 감지 또는 대체 글꼴 사용을 묻는다');
  const input = await page.$('#file-input');
  await input.uploadFile(path.resolve('../samples/eq-002.hwp'));
  await page.waitForFunction(() => [...document.querySelectorAll('.modal-overlay button')]
    .some(button => button.textContent?.includes('대체 글꼴로 보기')), { timeout: 30000 });
  await page.screenshot({ path: '../output/issue8371/local-font-choice.png' });
  const before = await page.evaluate(() => window.__fontQueries ?? 0);
  await page.evaluate(() => [...document.querySelectorAll('.modal-overlay button')]
    .find(button => button.textContent?.includes('대체 글꼴로 보기')).click());
  await page.waitForFunction(() => ![...document.querySelectorAll('.modal-overlay button')]
    .some(button => button.textContent?.includes('대체 글꼴로 보기')));
  const after = await page.evaluate(async () => {
    const fonts = await import('/src/core/local-fonts.ts');
    return { queries: window.__fontQueries ?? 0, stored: fonts.getLocalFontState().stored };
  });
  assert(before === 0 && after.queries === 0, '대체 글꼴 선택 시 로컬 글꼴 열거를 호출하지 않는다');
  assert(after.stored === false, '대체 글꼴 선택으로 로컬 글꼴 감지를 승인하지 않는다');
  await page.evaluate(() => { document.querySelector('#file-input').value = ''; });
  await input.uploadFile(path.resolve('../samples/eq-002.hwp'));
  await page.waitForFunction(() => [...document.querySelectorAll('.modal-overlay button')]
    .some(button => button.textContent?.includes('로컬 글꼴 감지')), { timeout: 30000 });
  await page.evaluate(() => [...document.querySelectorAll('.modal-overlay button')]
    .find(button => button.textContent?.includes('로컬 글꼴 감지')).click());
  await page.waitForFunction(() => (window.__fontQueries ?? 0) > 0);
  assert(await page.evaluate(() => window.__fontQueries) > 0, '사용자가 감지를 선택한 뒤에만 열거한다');
}, { skipLoadApp: true });
