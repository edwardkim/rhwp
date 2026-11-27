import assert from 'node:assert/strict';
import { loadApp, runTest, setTestCase } from './helpers.mjs';

const STYLE = '#style-name';
const LIST = '#style-picker-list';
const SAMPLE = 'hwp3-sample16-hwp5-2024.hwp';

async function clickArrow(page) {
  const control = await page.$(STYLE);
  const { x, y, width, height } = await control.boundingBox();
  await page.mouse.click(x + width - 8, y + height / 2);
}

function state(page) {
  return page.evaluate(() => {
    const cursor = window.__inputHandler.getCursorPosition();
    return {
      selected: document.querySelector('#style-name').value,
      selection: window.__inputHandler.getSelection(),
      style: window.__wasm.getStyleAt(cursor.sectionIndex, cursor.paragraphIndex).id,
      undoDepth: window.__inputHandler.history.undoStack.length,
    };
  });
}

await runTest('스타일 콤보 열림 유지 및 재클릭 닫기 (#9452)', async ({ page }) => {
  await loadApp(page, `/?url=/samples/${SAMPLE}`);
  await page.waitForFunction(() => document.querySelector('#style-name').options.length > 10
    || !!document.querySelector('.modal-overlay .dialog-footer .dialog-btn:not(.dialog-btn-primary)'));
  const fontChoice = await page.$('.modal-overlay .dialog-footer .dialog-btn:not(.dialog-btn-primary)');
  if (fontChoice) await fontChoice.click();
  await page.waitForFunction(() => window.__inputHandler.isActive()
    && document.querySelector('#style-name').options.length > 10, { timeout: 30000 });

  const before = await state(page);
  const count = await page.$eval(STYLE, element => element.options.length);
  console.log(`  ${SAMPLE}: ${count}개 스타일, 최초 선택 ${before.selected}`);

  setTestCase('마우스를 떼고 기다린 뒤 같은 화살표로 닫기');
  await clickArrow(page);
  // 재현 조건은 클릭이 끝나고 추가 입력 없이 기다려도 열린 상태다.
  await new Promise(resolve => setTimeout(resolve, 600));
  assert.equal(await page.$eval(STYLE, element => element.getAttribute('aria-expanded')), 'true');
  assert.equal(await page.$$eval(`${LIST} [role="option"]`, options => options.length), count);
  assert.deepEqual(await state(page), before);
  const box = await (await page.$(LIST)).boundingBox();
  const scrollBefore = await page.$eval(LIST, list => list.scrollTop);
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.wheel({ deltaY: scrollBefore ? -120 : 120 });
  await page.waitForFunction(previous => document.querySelector('#style-picker-list')?.scrollTop !== previous,
    { timeout: 5000 }, scrollBefore);
  assert.equal(await page.$eval(STYLE, element => element.getAttribute('aria-expanded')), 'true');
  await clickArrow(page);
  assert.equal(await page.$(LIST), null);
  assert.deepEqual(await state(page), before);
  console.log('  PASS: 열림 유지, 스크롤, 재클릭 닫기 및 문서 상태 유지');

  setTestCase('닫기 입력 및 키보드 탐색');
  await page.keyboard.press('Enter');
  await page.keyboard.press('End');
  assert.equal(await page.$eval(STYLE, element => element.getAttribute('aria-activedescendant')),
    `style-picker-option-${count - 1}`);
  await page.keyboard.press('Escape');
  assert.equal(await page.$(LIST), null);
  assert.deepEqual(await state(page), before);
  await clickArrow(page);
  await page.click('#font-lang');
  assert.equal(await page.$(LIST), null);
  await page.keyboard.press('Escape');
  assert.deepEqual(await state(page), before);
  console.log('  PASS: 키보드 탐색, Escape 및 목록 밖 클릭으로 닫기');

  setTestCase('다른 스타일을 한 번 적용하고 닫기');
  const alternative = await page.$eval(STYLE, element => {
    const index = element.selectedIndex === 0 ? 1 : 0;
    return { index, value: element.options[index].value };
  });
  await clickArrow(page);
  await page.click(`#style-picker-option-${alternative.index}`);
  assert.equal(await page.$(LIST), null);
  const applied = await state(page);
  assert.equal(applied.selected, alternative.value);
  assert.equal(applied.style, Number(alternative.value));
  assert.equal(applied.undoDepth, before.undoDepth + 1);
  console.log('  PASS: 선택 스타일 적용, 목록 닫기 및 실행 취소 한 건');
}, { skipLoadApp: true });
