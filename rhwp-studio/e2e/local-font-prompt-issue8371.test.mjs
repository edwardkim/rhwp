// 관계형: 초기화 후 다음 문서 로드의 재선택·조회 횟수·빈 문서 입력 계약을 실제 UI로 검사한다.
// 주석 갱신: 2026-11-07 20:03 KST. 설치 글꼴·페이지 px 좌표에 의존하지 않는다.
import { assert, clickEditArea, createNewDocument, loadApp, runTest, setTestCase, typeText } from './helpers.mjs';
import path from 'node:path';
import { mkdirSync } from 'node:fs';

runTest('초기화 후 문서 글꼴 사용 선택', async ({ page }) => {
  mkdirSync('../output/issue8371', { recursive: true });
  await page.evaluateOnNewDocument(() => {
    const query = navigator.permissions.query.bind(navigator.permissions);
    navigator.permissions.query = options => options.name === 'local-fonts'
      ? Promise.resolve({ state: 'granted' }) : query(options);
    Object.defineProperty(globalThis, 'queryLocalFonts', {
    configurable: true,
    value: async () => { window.__fontQueries = (window.__fontQueries ?? 0) + 1; return []; },
    });
  });
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
  await page.waitForFunction(() => window.__localFonts.getLocalFontState().stored);
  for (const choice of ['대체 글꼴로 보기', '로컬 글꼴 감지']) {
    setTestCase(`초기화는 삭제만 하고 다음 문서 로드에서 선택: ${choice}`);
    const queries = await page.evaluate(async () => {
      const { OptionsDialog } = await import('/src/ui/options-dialog.ts');
      new OptionsDialog(window.__eventBus).show();
      return window.__fontQueries;
    });
    await page.waitForFunction(() => !document.querySelector('.opt-local-actions button:last-child').disabled);
    await page.evaluate(() => document.querySelector('.opt-local-actions button:last-child').click());
    await page.waitForFunction(() => !window.__localFonts.getLocalFontState().stored
      && !document.querySelector('.opt-local-actions button').disabled);
    await page.evaluate(() => new Promise(resolve => setTimeout(resolve, 200)));
    assert(await page.evaluate(() => ![...document.querySelectorAll('.modal-overlay button')]
      .some(button => button.textContent?.includes('대체 글꼴로 보기'))),
      '초기화 버튼은 로컬/대체 선택창을 다시 띄우지 않는다');
    assert(await page.evaluate(() => window.__fontQueries) === queries,
      '초기화 직후 브라우저 권한으로 자동 재열거하지 않는다');
    await page.screenshot({ path: '../output/issue8371/local-font-reset-only.png' });
    await page.evaluate(() => document.querySelector('.modal-overlay .dialog-close').click());
    await page.evaluate(() => { document.querySelector('#file-input').value = ''; });
    await input.uploadFile(path.resolve('../samples/eq-002.hwp'));
    await page.waitForFunction(() => [...document.querySelectorAll('.modal-overlay button')]
      .some(button => button.textContent?.includes('대체 글꼴로 보기')));
    assert(await page.evaluate(() => window.__fontQueries) === queries,
      '다음 문서 로드도 선택하기 전에는 자동 감지하지 않는다');
    await page.screenshot({ path: '../output/issue8371/local-font-reset-next-load-choice.png' });
    await page.evaluate(text => [...document.querySelector('.modal-overlay:last-child').querySelectorAll('button')]
      .find(button => button.textContent?.includes(text)).click(), choice);
    await page.waitForFunction(() => ![...document.querySelectorAll('.modal-overlay button')]
      .some(button => button.textContent?.includes('대체 글꼴로 보기')));
    if (choice === '대체 글꼴로 보기') {
      assert(await page.evaluate(() => !window.__localFonts.getLocalFontState().stored
        && window.__fontQueries) === queries, '대체 선택 뒤에도 감지는 초기화된 상태를 유지한다');
      await page.evaluate(() => { document.querySelector('#file-input').value = ''; });
      await input.uploadFile(path.resolve('../samples/eq-002.hwp'));
      await page.waitForFunction(() => [...document.querySelectorAll('.modal-overlay button')]
        .some(button => button.textContent?.includes('로컬 글꼴 감지')));
      await page.evaluate(() => [...document.querySelectorAll('.modal-overlay button')]
        .find(button => button.textContent?.includes('로컬 글꼴 감지')).click());
      await page.waitForFunction(() => window.__localFonts.getLocalFontState().stored);
    } else {
      await page.waitForFunction(() => window.__localFonts.getLocalFontState().stored);
      assert(await page.evaluate(() => window.__fontQueries) === queries + 1,
        '재선택한 감지만 한 번 실행한다');
    }
  }
}, { skipLoadApp: true });
