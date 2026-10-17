import { runTest, setTestCase, assert } from './helpers.mjs';

runTest('#7983 교차 구역 선택 변경은 문서·선택·히스토리를 보존한다', async ({ page }) => {
  await page.evaluate(() => {
    document.querySelector('.skin-onboarding-card')?.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }));
    [...document.querySelectorAll('button.dialog-btn-primary')]
      .find(e => e.offsetParent !== null && e.textContent?.trim() === '시작하기')?.click();
  });
  const source = await page.evaluate(async () => {
    const response = await fetch('/samples/hml/aligns.hml');
    if (!response.ok) throw new Error(`fixture HTTP ${response.status}`);
    const xml = new DOMParser().parseFromString(await response.text(), 'application/xml');
    xml.querySelector('HEAD').setAttribute('SecCnt', '2');
    const section = xml.querySelector('BODY > SECTION');
    for (const child of [...section.children].slice(1)) child.remove();
    for (const character of section.querySelectorAll('CHAR')) character.textContent = '';
    const second = section.cloneNode(true);
    second.setAttribute('Id', '1');
    section.parentNode.appendChild(second);
    return new XMLSerializer().serializeToString(xml);
  });
  const read = () => page.evaluate(() => {
    const w = window.__wasm;
    const ih = window.__inputHandler;
    return {
      text: [w.getTextRange(0, 0, 0, 99), w.getTextRange(1, 0, 0, 99)],
      selection: ih.cursor.getSelectionOrdered(), undo: ih.history.undoStack.length,
      redo: ih.history.redoStack.length,
    };
  });
  const setup = async (endSection = 1) => {
    await page.evaluate(async ({ source, endSection }) => {
      const w = window.__wasm;
      w.loadDocument(new TextEncoder().encode(source), 'two-sections.hml');
      w.insertText(0, 0, 0, 'AAAA'); w.insertText(1, 0, 0, 'BBBB');
      await window.__canvasView.loadDocument();
      const ih = window.__inputHandler;
      ih.cursor.exitHeaderFooterMode(); ih.cursor.exitFootnoteMode();
      ih.history.clear(w);
      if (!ih.cursor.selectRange(
        { sectionIndex: 0, paragraphIndex: 0, charOffset: 1 },
        { sectionIndex: endSection, paragraphIndex: 0, charOffset: 3 },
      )) throw new Error('selection setup rejected');
      ih.active = true; ih.textarea.focus();
    }, { source, endSection });
  };

  for (const action of ['Delete', 'Backspace', 'Enter', 'Tab', 'input', 'paste', 'cut', 'composition']) {
    setTestCase(`#7983 ${action}`);
    await setup();
    const before = await read();
    if (action === 'input') await page.keyboard.type('X');
    else if (action === 'paste' || action === 'cut') {
      await page.evaluate(action => {
        const data = new DataTransfer(); data.setData('text/plain', 'X');
        window.__inputHandler.textarea.dispatchEvent(new ClipboardEvent(action, { clipboardData: data, bubbles: true, cancelable: true }));
      }, action);
    } else if (action === 'composition') {
      await page.evaluate(() => window.__inputHandler.textarea.dispatchEvent(new CompositionEvent('compositionstart', { bubbles: true })));
    } else await page.keyboard.press(action);
    assert(JSON.stringify(await read()) === JSON.stringify(before), `${action}: 문서·선택·undo/redo 보존`);
    assert(await page.evaluate(() => !!document.querySelector('#rhwp-toast-container')), '지원 범위 안내 표시');
  }
  setTestCase('#7983 same-section control');
  await setup(0);
  await page.keyboard.press('Delete');
  assert(JSON.stringify((await read()).text) === JSON.stringify(['AA', 'BBBB']), '동일 구역 삭제 유지');
  await page.keyboard.down('Control'); await page.keyboard.press('KeyZ'); await page.keyboard.up('Control');
  assert(JSON.stringify((await read()).text) === JSON.stringify(['AAAA', 'BBBB']), '동일 구역 undo 유지');

  setTestCase('#7983 form-mode Tab stays navigation');
  await setup(0);
  await page.evaluate(() => {
    const ih = window.__inputHandler;
    ih.editMode = 'form';
    window.__formTabCalls = [];
    const navigate = ih.moveToAdjacentFormField.bind(ih);
    ih.moveToAdjacentFormField = delta => { window.__formTabCalls.push(delta); return navigate(delta); };
  });
  await page.keyboard.press('Tab');
  await page.evaluate(() => window.__inputHandler.textarea.focus());
  await page.keyboard.down('Shift'); await page.keyboard.press('Tab'); await page.keyboard.up('Shift');
  assert(JSON.stringify(await page.evaluate(() => window.__formTabCalls)) === JSON.stringify([1, -1]),
    '같은 구역의 편집 불가 선택에서도 Tab/Shift+Tab은 필드 탐색');
});
