/**
 * E2E 테스트: 외부 클립보드가 rhwp-studio 내부 클립보드보다 우선되어야 함 (Task 871)
 *
 * macOS headless Chrome에서는 키보드 단축키 Ctrl/Cmd+C가 copy 이벤트를
 * 안정적으로 발생시키지 않는다. 그래서 복사 단계는 document.execCommand('copy')로
 * 앱의 onCopy 경로를 통과시키고, 붙여넣기 단계는 ClipboardEvent를 직접 주입해
 * 내부 marker 유무에 따른 우선순위만 검증한다.
 */
import {
  runTest, createNewDocument, clickEditArea, typeText,
  assert, getParaText,
} from './helpers.mjs';

async function wait(page, ms) {
  await page.evaluate((value) => new Promise(r => setTimeout(r, value)), ms);
}

async function selectAll(page) {
  await page.keyboard.down('Control');
  await page.keyboard.press('a');
  await page.keyboard.up('Control');
  await wait(page, 300);
}

async function prepareInternalClipboard(page, expectedText) {
  await selectAll(page);
  const copyOk = await page.evaluate(() => document.execCommand('copy'));
  assert(copyOk, 'document.execCommand("copy") 이벤트 발생');
  await wait(page, 200);

  const state = await page.evaluate(() => ({
    hasInternalClipboard: window.__wasm?.hasInternalClipboard?.() ?? false,
    clipboardText: window.__wasm?.getClipboardText?.() ?? '',
    token: window.__inputHandler?.rhwpClipboardToken ?? null,
  }));
  assert(state.hasInternalClipboard, 'rhwp-studio 내부 클립보드 생성');
  assert(state.clipboardText.includes(expectedText), `내부 클립보드 텍스트 확인: ${state.clipboardText}`);
  assert(!!state.token, 'rhwp-studio clipboard marker token 생성');

  await page.keyboard.press('End');
  await wait(page, 200);
  return state;
}

function rhwpHtml(token, text) {
  return `<html><body>
<!--rhwp-studio-clipboard:${token}-->
<!--StartFragment-->
<p>${text}</p>
<!--EndFragment-->
</body></html>`;
}

function externalHtml(text) {
  return `<html><body>
<!--StartFragment-->
<p>${text}</p>
<!--EndFragment-->
</body></html>`;
}

async function dispatchPaste(page, { text = '', html = '' }) {
  await page.evaluate(({ text: plainText, html: htmlText }) => {
    const data = new DataTransfer();
    if (plainText) data.setData('text/plain', plainText);
    if (htmlText) data.setData('text/html', htmlText);
    const event = new ClipboardEvent('paste', {
      clipboardData: data,
      bubbles: true,
      cancelable: true,
    });
    const target = document.activeElement || document.body;
    target.dispatchEvent(event);
  }, { text, html });
  await wait(page, 500);
}

async function expectFirstParagraph(page, expectedText) {
  const text = await getParaText(page, 0, 0, 200);
  assert(text.includes(expectedText), `문단 텍스트 확인: ${text}`);
  return text;
}

async function createDocumentWithInternalClipboard(page, text) {
  await createNewDocument(page);
  await clickEditArea(page);
  await typeText(page, text);
  await expectFirstParagraph(page, text);
  return await prepareInternalClipboard(page, text);
}

runTest('Task 871: 클립보드 우선순위', async ({ page }) => {
  const internalState = await createDocumentWithInternalClipboard(page, 'abcdefg');
  await dispatchPaste(page, {
    text: 'abcdefg',
    html: rhwpHtml(internalState.token, 'abcdefg'),
  });
  await expectFirstParagraph(page, 'abcdefgabcdefg');
  console.log('  [1] rhwp marker가 있는 붙여넣기는 내부 클립보드를 사용');

  await createDocumentWithInternalClipboard(page, 'abcdefg');
  await dispatchPaste(page, { text: 'OUTSIDE' });
  const externalText = await expectFirstParagraph(page, 'abcdefgOUTSIDE');
  assert(!externalText.includes('abcdefgabcdefg'), '외부 plain text가 오래된 내부 클립보드보다 우선');
  console.log('  [2] 외부 plain text는 내부 클립보드를 덮어씀');

  await createDocumentWithInternalClipboard(page, 'base');
  await dispatchPaste(page, {
    text: 'HTML',
    html: externalHtml('HTML'),
  });
  const htmlText = await expectFirstParagraph(page, 'baseHTML');
  assert(!htmlText.includes('basebase'), 'marker 없는 외부 HTML이 내부 클립보드를 사용하지 않음');
  console.log('  [3] marker 없는 외부 HTML은 pasteHtml 경로를 사용');
});
