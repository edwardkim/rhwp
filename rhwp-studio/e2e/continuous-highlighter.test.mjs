/**
 * 연속 형광펜 모드: 블록 없이 형광펜 색을 고르면 모드에 진입하고, 드래그로 블록을 잡을 때마다
 * 그 색을 칠한다. Esc 또는 형광펜 버튼 재클릭으로 종료하며, 블록이 있으면 기존처럼 즉시 칠한다.
 */
import {
  assert, clickEditArea, createNewDocument, runTest, setTestCase, typeText,
} from './helpers.mjs';

const GREEN = '#00ff00';

async function isOpen(page) {
  return page.$eval('#highlight-dropdown', element => element.classList.contains('open'));
}

async function penState(page) {
  return page.evaluate(() => ({
    mode: document.getElementById('scroll-container').classList.contains('highlight-pen-mode'),
    active: document.getElementById('btn-highlight').classList.contains('active'),
    cursorVar: document.getElementById('scroll-container').style.getPropertyValue('--highlight-pen-cursor'),
  }));
}

async function shadeAt(page, offset) {
  return page.evaluate(o => window.__wasm.getCharPropertiesAt(0, 0, o).shadeColor, offset);
}

async function undoDepth(page) {
  return page.evaluate(() => window.__inputHandler.history.undoStack.length);
}

async function hasSelection(page) {
  return page.evaluate(() => window.__inputHandler.cursor.hasSelection());
}

/** 첫 문단의 글자 위치 offset 을 화면 좌표로 바꾼다 */
async function pointAt(page, offset) {
  const point = await page.evaluate(o => {
    const scrollContent = document.getElementById('scroll-content');
    const canvasView = window.__canvasView;
    const zoom = canvasView.viewportManager.getZoom();
    const virtualScroll = canvasView.virtualScroll;
    const rect = window.__wasm.getCursorRect(0, 0, o);
    const contentRect = scrollContent.getBoundingClientRect();
    const pageOffset = virtualScroll.getPageOffset(rect.pageIndex);
    const pageLeft = (scrollContent.clientWidth - virtualScroll.getPageWidth(rect.pageIndex)) / 2;
    return {
      x: contentRect.left + pageLeft + rect.x * zoom,
      y: contentRect.top + pageOffset + rect.y * zoom + rect.height * zoom / 2,
    };
  }, offset);
  return point;
}

async function dragChars(page, from, to) {
  const start = await pointAt(page, from);
  const end = await pointAt(page, to);
  await page.mouse.move(start.x + 1, start.y);
  await page.mouse.down();
  await page.mouse.move(end.x - 1, end.y, { steps: 8 });
  await page.mouse.up();
  await page.evaluate(() => new Promise(r => setTimeout(r, 150)));
}

async function pickSwatch(page) {
  if (!await isOpen(page)) await page.click('#btn-highlight');
  const swatches = await page.$$('.sb-hl-swatch');
  await swatches[4].click(); // 첫 행 다섯 번째 = #00ff00
}

runTest('연속 형광펜 모드', async ({ page }) => {
  if (await page.$('.skin-onboarding-card')) {
    await page.click('.skin-onboarding-card');
    await page.click('.dialog-btn-primary');
  }
  await createNewDocument(page);
  await clickEditArea(page);
  await typeText(page, 'alpha beta gamma delta');
  const base = await shadeAt(page, 1);

  setTestCase('블록 없이 색을 고르면 연속 형광펜 모드에 진입한다');
  await clickEditArea(page);
  assert(!await hasSelection(page), '시작 시 블록이 없다');
  const depth0 = await undoDepth(page);
  await pickSwatch(page);
  let state = await penState(page);
  assert(state.mode, '본문 컨테이너가 형광펜 모드가 된다');
  assert(state.active, '형광펜 버튼이 눌린 상태로 표시된다');
  assert(state.cursorVar.includes('svg'), '선택한 색의 펜 커서가 설정된다');
  assert(!await isOpen(page), '색 선택 후 팔레트가 닫힌다');
  assert(await undoDepth(page) === depth0, '모드 진입만으로 undo 기록이 생기지 않는다');

  setTestCase('드래그할 때마다 칠하고 모드를 유지한다');
  await dragChars(page, 0, 5);
  assert(await shadeAt(page, 1) === GREEN, '첫 드래그 블록이 칠해진다');
  assert(!await hasSelection(page), '칠한 뒤 블록이 해제된다');
  assert((await penState(page)).mode, '첫 드래그 뒤에도 모드가 유지된다');
  await dragChars(page, 11, 16);
  assert(await shadeAt(page, 13) === GREEN, '두 번째 드래그 블록도 칠해진다');
  assert(await shadeAt(page, 8) === base, '드래그하지 않은 글자는 칠해지지 않는다');
  assert(await undoDepth(page) === depth0 + 2, '드래그마다 undo 한 단계가 기록된다');

  setTestCase('undo 는 드래그 한 번씩 되돌린다');
  await page.evaluate(() => window.__inputHandler.performUndo());
  assert(await shadeAt(page, 13) === base, 'undo 로 두 번째 칠이 되돌려진다');
  assert(await shadeAt(page, 1) === GREEN, '첫 번째 칠은 남는다');

  setTestCase('Esc 로 모드를 종료한다');
  await page.keyboard.press('Escape');
  state = await penState(page);
  assert(!state.mode && !state.active, 'Esc 로 모드와 버튼 눌림이 해제된다');
  assert(state.cursorVar === '', '펜 커서 변수가 제거된다');
  const depth1 = await undoDepth(page);
  await dragChars(page, 17, 22);
  assert(await shadeAt(page, 19) === base, '모드 종료 후 드래그는 칠하지 않는다');
  assert(await undoDepth(page) === depth1, '모드 종료 후 드래그는 undo 기록을 남기지 않는다');

  setTestCase('형광펜 버튼 재클릭으로 모드를 종료한다');
  await clickEditArea(page);
  await pickSwatch(page);
  assert((await penState(page)).mode, '다시 모드에 진입한다');
  await page.click('#btn-highlight');
  state = await penState(page);
  assert(!state.mode && !state.active, '버튼 재클릭으로 모드가 해제된다');
  assert(!await isOpen(page), '모드 종료 클릭은 팔레트를 열지 않는다');

  setTestCase('블록이 있으면 즉시 칠하고 모드에 들어가지 않는다');
  await dragChars(page, 6, 10);
  assert(await hasSelection(page), '드래그로 블록이 잡힌다');
  await pickSwatch(page);
  assert(await shadeAt(page, 8) === GREEN, '선택한 블록이 즉시 칠해진다');
  assert(!(await penState(page)).mode, '블록이 있을 때는 모드에 진입하지 않는다');
});
