import { assert, loadApp, runTest, setTestCase } from './helpers.mjs';

runTest('로컬 글꼴 수 재감지와 재열기', async ({ page }) => {
  await page.evaluateOnNewDocument(() => Object.defineProperty(globalThis, 'queryLocalFonts', {
    configurable: true,
    value: async () => [
      { family: 'Example', fullName: 'Example Regular', postscriptName: 'Example-Regular', style: 'Regular' },
      { family: 'Example', fullName: 'Example Bold', postscriptName: 'Example-Bold', style: 'Bold' },
    ],
  }));
  await loadApp(page);
  setTestCase('한 family의 두 face를 재감지하고 설정을 다시 연다');
  await page.evaluate(async () => {
    const { OptionsDialog } = await import('/src/ui/options-dialog.ts');
    window.__fontCountDialog = new OptionsDialog();
    window.__fontCountDialog.show();
    document.querySelector('.opt-local-actions button').click();
  });
  await page.waitForFunction(() => !document.querySelector('.opt-local-actions button').disabled);
  const first = await page.$eval('.opt-local-status', element => element.textContent);
  await page.evaluate(() => {
    window.__fontCountDialog.hide();
    window.__fontCountDialog = new window.__fontCountDialog.constructor();
    window.__fontCountDialog.show();
  });
  await page.waitForFunction(() => document.querySelector('.opt-local-status')?.textContent?.includes('2개 face'));
  const reopened = await page.$eval('.opt-local-status', element => element.textContent);
  assert(first.includes('2개 face') && reopened.includes('2개 face'), '재감지와 재열기의 face 수가 같다');
}, { skipLoadApp: true });
