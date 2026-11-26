// 관계형 검사. 인라인 표 뒤에 입력한 한글과 보조 평면 문자는 같은 글자와 캐럿으로 취소·복원된다.
// 96 DPI 한컴 Print와 두 렌더러의 전쪽 실루엣은 mydocs/pr/assets/semanticist21-9337-9354-9377/focus-pages.tsv에 기록했다.
import { fileURLToPath } from 'node:url';
import { runTest, createNewDocument, assert } from './helpers.mjs';

async function openArea(page, fixture, isHeader) {
  await createNewDocument(page);
  const name = fixture + '.hwpx';
  const source = new URL('../../tests/fixtures/issue9354/' + name, import.meta.url);
  const input = await page.$('#file-input');
  await input.uploadFile(fileURLToPath(source));
  await page.waitForFunction((name) => {
    const fallback = [...document.querySelectorAll('.modal-overlay button')].find(button =>
      ['대체 글꼴로 보기', 'View with substitute fonts'].includes(button.textContent?.trim()));
    fallback?.click();
    return window.__wasm.fileName === name && window.__inputHandler.isActive()
      && !document.documentElement.classList.contains('rhwp-busy') && !window.__documentState.isDirty();
  }, { timeout: 30000 }, name);
  await page.evaluate((isHeader) => {
    const handler = window.__inputHandler;
    handler.cursor.enterHeaderFooterMode(isHeader, 0, 0, 0);
    handler.cursor.setHfCursorPosition(0, 3);
    handler.eventBus.emit('headerFooterModeChanged', {
      mode: isHeader ? 'header' : 'footer', sectionIdx: 0, applyTo: 0, previewPage: 0,
    });
    handler.focus();
  }, isHeader);
}

async function assertArea(page, isHeader, text, offset, step) {
  const actual = await page.evaluate((isHeader) => {
    const handler = window.__inputHandler;
    const paragraph = JSON.parse(handler.wasm.getHeaderFooterParaInfo(0, isHeader, 0, 0));
    return { text: paragraph.text, offset: handler.cursor.hfCharOffset };
  }, isHeader);
  assert(actual.text === text && actual.offset === offset,
    step + ' (' + JSON.stringify(actual) + ')');
}

async function typeComposedHangul(page) {
  const client = await page.createCDPSession();
  try {
    for (const text of ['ㅎ', '하', '한']) {
      await client.send('Input.imeSetComposition', { text, selectionStart: 1, selectionEnd: 1 });
    }
    await client.send('Input.insertText', { text: '한' });
  } finally {
    await client.detach();
  }
}

runTest('#9354 머리말·꼬리말의 끝 표 뒤 입력과 실행 취소', async ({ page }) => {
  for (const [fixture, isHeader] of [
    ['trailing-header', true], ['trailing-footer', false],
    ['short-header', true], ['short-footer', false],
  ]) {
    for (const input of ['한😀', '한']) {
      await openArea(page, fixture, isHeader);
      const label = fixture + ' ' + input;
      await assertArea(page, isHeader, '앞뒤', 3, label + ' 편집 전');
      if (input === '한') await typeComposedHangul(page);
      else await page.keyboard.sendCharacter(input);
      const end = 2 + [...input].length;
      await assertArea(page, isHeader, '앞뒤' + input, end, label + ' 입력');
      await page.keyboard.down('Control');
      await page.keyboard.press('z');
      await assertArea(page, isHeader, '앞뒤', 3, label + ' 실행 취소');
      await page.keyboard.down('Shift');
      await page.keyboard.press('z');
      await page.keyboard.up('Shift');
      await page.keyboard.up('Control');
      await assertArea(page, isHeader, '앞뒤' + input, end, label + ' 다시 실행');
    }
  }
});
