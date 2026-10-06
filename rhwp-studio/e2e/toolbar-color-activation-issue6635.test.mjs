/**
 * #6635: 실제 Tab/Enter/Space와 마우스로 색상 버튼을 활성화한다.
 * native picker의 OS 창 표시는 검사하지 않는다. input으로 전달되는 click을 관찰하고,
 * 색상 적용 검증에서만 picker의 input 이벤트를 대신 전달한다.
 */
import {
  assert, clickEditArea, createNewDocument, runTest, setTestCase, typeText,
} from './helpers.mjs';

const OTHER = '#highlight-palette button:nth-of-type(2)';
const NONE = '#highlight-palette button:first-of-type';
const PICKER = '#highlight-palette input[type="color"]';

async function tabTo(page, selector) {
  await page.click('#font-size');
  for (let i = 0; i < 30; i++) {
    await page.keyboard.press('Tab');
    if (await page.$eval(selector, element => element === document.activeElement)) return;
  }
  throw new Error(`Tab으로 버튼에 도달하지 못함: ${selector}`);
}

async function isOpen(page) {
  return page.$eval('#highlight-dropdown', element => element.classList.contains('open'));
}

async function openPalette(page) {
  if (!await isOpen(page)) await page.click('#btn-highlight');
}

async function selection(page) {
  return page.evaluate(() => JSON.stringify(window.__inputHandler.cursor.getSelection()));
}

async function properties(page) {
  return page.evaluate(() => window.__wasm.getCharPropertiesAt(0, 0, 1));
}

async function selectText(page) {
  await clickEditArea(page);
  await page.keyboard.down('Control');
  await page.keyboard.press('KeyA');
  await page.keyboard.up('Control');
}

runTest('색상 버튼 표준 활성화와 선택 보존 (#6635)', async ({ page }) => {
  // 처음 실행하는 브라우저에서도 실제 버튼으로 스킨 안내를 마친다.
  if (await page.$('.skin-onboarding-card')) {
    await page.click('.skin-onboarding-card');
    await page.click('.dialog-btn-primary');
  }
  await createNewDocument(page);
  await clickEditArea(page);
  await typeText(page, 'color selection');
  await selectText(page);
  const selected = await selection(page);
  assert(selected !== 'null', '본문 텍스트 선택이 존재한다');
  const before = await properties(page);
  const undoDepth = await page.evaluate(() => window.__inputHandler.history.undoStack.length);

  setTestCase('숨겨진 색상 input을 건너뛰는 양방향 Tab 이동');
  await tabTo(page, '#btn-text-color');
  await page.keyboard.press('Tab');
  assert(await page.$eval('#btn-highlight', button => button === document.activeElement),
    '글자색에서 Tab 한 번으로 형광펜 버튼에 도달한다');
  await tabTo(page, '#btn-highlight');
  await page.keyboard.down('Shift');
  await page.keyboard.press('Tab');
  await page.keyboard.up('Shift');
  assert(await page.$eval('#btn-text-color', button => button === document.activeElement),
    '형광펜에서 Shift+Tab 한 번으로 글자색 버튼에 도달한다');

  for (const [target, tabs] of [['형광펜 버튼', 0], ['색 없음', 1], ['다른 색', 2]]) {
    setTestCase(`${target}: Escape로 닫기와 포커스·선택 보존`);
    await tabTo(page, '#btn-highlight');
    await page.keyboard.press('Enter');
    assert(await isOpen(page), `${target}: 키보드로 팔레트가 열린다`);
    for (let i = 0; i < tabs; i++) await page.keyboard.press('Tab');
    const selector = tabs === 0 ? '#btn-highlight' : tabs === 1 ? NONE : OTHER;
    assert(await page.$eval(selector, button => button === document.activeElement),
      `${target}: Escape를 누를 요소에 포커스가 있다`);
    await page.keyboard.press('Escape');
    assert(!await isOpen(page), `${target}: Escape로 팔레트가 닫힌다`);
    assert(await page.$eval('#btn-highlight', button => button === document.activeElement),
      `${target}: 형광펜 버튼으로 포커스가 복원된다`);
    assert(await selection(page) === selected, `${target}: 선택 영역이 보존된다`);
    assert(JSON.stringify(await properties(page)) === JSON.stringify(before),
      `${target}: 닫기만으로 서식이 바뀌지 않는다`);
    assert(await page.evaluate(() => window.__inputHandler.history.undoStack.length) === undoDepth,
      `${target}: 닫기만으로 undo 기록이 생기지 않는다`);
    // 수정 전 실패를 확인할 때도 다음 사례의 시작 상태를 일정하게 유지한다.
    if (await isOpen(page)) await page.click('#btn-highlight');
  }

  await page.evaluate(() => {
    window.__colorPickerClicks = { text: 0, highlight: 0 };
    document.querySelector('#text-color-picker').addEventListener('click', () => {
      window.__colorPickerClicks.text++;
    });
    document.querySelector('#highlight-palette input[type="color"]').addEventListener('click', () => {
      window.__colorPickerClicks.highlight++;
    });
  });

  for (const key of ['Enter', 'Space']) {
    setTestCase(`${key}: Tab으로 이동한 세 버튼의 표준 활성화`);
    await tabTo(page, '#btn-text-color');
    const beforeText = await page.evaluate(() => window.__colorPickerClicks.text);
    await page.keyboard.press(key);
    assert(await page.evaluate(() => window.__colorPickerClicks.text) === beforeText + 1,
      `${key}: 글자색 input이 정확히 한 번 활성화된다`);
    await page.keyboard.press('Escape');

    await tabTo(page, '#btn-highlight');
    await page.keyboard.press(key);
    assert(await isOpen(page), `${key}: 형광펜 팔레트가 열린다`);
    await page.keyboard.press(key);
    assert(!await isOpen(page), `${key}: 다시 누르면 팔레트가 닫힌다`);

    await tabTo(page, '#btn-highlight');
    await openPalette(page);
    await page.keyboard.press('Tab');
    await page.keyboard.press('Tab');
    assert(await page.$eval(OTHER, element => element === document.activeElement), '다른 색 버튼에 포커스');
    const beforeOther = await page.evaluate(() => window.__colorPickerClicks.highlight);
    await page.keyboard.press(key);
    assert(await page.evaluate(() => window.__colorPickerClicks.highlight) === beforeOther + 1,
      `${key}: 다른 색 input이 정확히 한 번 활성화된다`);
    await page.keyboard.press('Escape');
    assert(await selection(page) === selected, `${key}: 선택 영역이 보존된다`);
    await page.click('#btn-highlight');
  }

  setTestCase('마우스 활성화와 유효한 색상 input 구조');
  for (const [button, counter] of [['#btn-text-color', 'text'], [OTHER, 'highlight']]) {
    if (counter === 'highlight') await openPalette(page);
    const before = await page.evaluate(name => window.__colorPickerClicks[name], counter);
    await page.click(button);
    assert(await page.evaluate(name => window.__colorPickerClicks[name], counter) === before + 1,
      `마우스: ${counter} input이 정확히 한 번 활성화된다`);
    await page.keyboard.press('Escape');
    assert(await selection(page) === selected, `마우스: ${counter} 선택 영역 보존`);
  }
  assert(await page.$eval(PICKER, input => !input.closest('button')), 'color input은 button에 중첩되지 않는다');
  assert(JSON.stringify(await properties(page)) === JSON.stringify(before), '색을 고르지 않으면 서식이 바뀌지 않는다');
  assert(await page.evaluate(() => window.__inputHandler.history.undoStack.length) === undoDepth,
    '열기와 취소만으로 undo 기록이 생기지 않는다');

  setTestCase('선택한 글자에 색 적용, 색 없음 키보드 활성화와 undo');
  await page.$eval(PICKER, input => {
    input.value = '#00ff00';
    input.dispatchEvent(new Event('input', { bubbles: true }));
  });
  assert((await properties(page)).shadeColor === '#00ff00', '선택한 글자에 사용자 지정 형광펜 색이 적용된다');
  assert(!await isOpen(page), '색 적용 후 팔레트가 닫힌다');
  assert(await page.evaluate(() => window.__inputHandler.history.undoStack.length) === undoDepth + 1,
    '색 적용은 undo 한 단계다');
  assert(await page.evaluate(() => document.activeElement === window.__inputHandler.textarea),
    '색 적용 후 편집기 입력 포커스가 복원된다');
  assert(await selection(page) === selected, '색 적용 후 선택 범위가 보존된다');

  for (const activation of ['Enter', 'Space', 'mouse']) {
    // 본문 서식 undo는 기존 계약상 선택을 해제한다. 다음 적용 전에 다시 선택한다.
    await selectText(page);
    await tabTo(page, '#btn-highlight');
    await openPalette(page);
    await page.keyboard.press('Tab');
    assert(await page.$eval(NONE, element => element === document.activeElement), '색 없음 버튼에 포커스');
    if (activation === 'mouse') await page.click(NONE);
    else await page.keyboard.press(activation);
    assert((await properties(page)).shadeColor === '#ffffff', `${activation}: 색 없음이 적용된다`);
    assert(!await isOpen(page), `${activation}: 색 없음 적용 후 팔레트가 닫힌다`);
    await page.evaluate(() => window.__inputHandler.performUndo());
    assert((await properties(page)).shadeColor === '#00ff00', `${activation}: undo로 형광펜 색이 복원된다`);
  }
  await page.evaluate(() => window.__inputHandler.performUndo());
  assert((await properties(page)).shadeColor === before.shadeColor, 'undo로 원래 서식이 복원된다');

  setTestCase('글자색 적용과 undo');
  await selectText(page);
  await page.$eval('#text-color-picker', input => {
    input.value = '#ff0000';
    input.dispatchEvent(new Event('input', { bubbles: true }));
  });
  assert((await properties(page)).textColor === '#ff0000', '선택한 글자에 글자색이 적용된다');
  assert(await selection(page) === selected, '글자색 적용 후 선택 범위가 보존된다');
  await page.evaluate(() => window.__inputHandler.performUndo());
  assert((await properties(page)).textColor === before.textColor, 'undo로 원래 글자색이 복원된다');
});
