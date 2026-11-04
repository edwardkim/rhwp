import { assert, loadApp, runTest, setTestCase } from './helpers.mjs';

runTest('로컬 글꼴 목록 검색', async ({ page }) => {
  await page.evaluateOnNewDocument(() => {
    Object.defineProperty(globalThis, 'queryLocalFonts', {
      configurable: true,
      value: async () => [
        { family: 'Strawberry', fullName: '딸기', postscriptName: 'Strawberry-Regular', style: 'Regular' },
        { family: '(1) YangPyeong L', fullName: '(1) 양평군체 L Light', postscriptName: '1-YangPyeong-L', style: 'Light' },
        { family: 'Yangju Byeolsan', fullName: '[양주]별산체 Regular', postscriptName: 'YangjuByeolsan', style: 'Regular' },
      ],
    });
  });
  await loadApp(page);
  setTestCase('감지된 face와 별칭 표시 및 검색');
  const result = await page.evaluate(async () => {
    const local = await import('/src/core/local-fonts.ts');
    const { OptionsDialog } = await import('/src/ui/options-dialog.ts');
    await local.detectLocalFonts({ force: true, includeRegistered: true });
    new OptionsDialog().show();
    const buttons = [...document.querySelectorAll('.opt-local-actions button')];
    buttons[1].click();
    await new Promise(resolve => setTimeout(resolve, 0));
    const overlay = [...document.querySelectorAll('.modal-overlay')].at(-1);
    const text = overlay.textContent;
    const search = overlay.querySelector('input[type=search]');
    search.value = '딸기';
    search.dispatchEvent(new Event('input', { bubbles: true }));
    const matched = overlay.querySelectorAll('tbody tr').length;
    search.value = '없는 글꼴';
    search.dispatchEvent(new Event('input', { bubbles: true }));
    return { buttonCount: buttons.length, text, matched,
      missing: overlay.querySelectorAll('tbody tr').length };
  });
  assert(result.buttonCount === 3, '감지와 초기화 사이에 목록 버튼이 있다');
  assert(result.text.includes('딸기') && result.text.includes('Strawberry'), '한글·영문 이름 표시');
  assert(result.text.includes('Strawberry-Regular'), 'PostScript face 표시');
  assert(result.matched === 1 && result.missing === 0, '이름으로 검색한다');
  await page.evaluate(() => {
    const search = document.querySelector('.local-font-list-search');
    search.value = '';
    search.dispatchEvent(new Event('input', { bubbles: true }));
  });
  await page.screenshot({ path: '../output/issue8371/local-font-list-popup.png' });
}, { skipLoadApp: true });
