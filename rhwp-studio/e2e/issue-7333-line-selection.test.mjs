/**
 * #7333 8쪽의 넓은 도형 위에 그린 화살표를 실제 마우스로 선택한다.
 *
 * 단순 bbox hit-test는 화살표를 감싼 도형을 먼저 고를 수 있다. 이 검증은 실클릭 뒤
 * 선택 참조가 line인지와 클릭이 z-order 변경/Undo 항목을 만들지 않는지를 함께 확인한다.
 */
import assert from 'node:assert/strict';
import { loadHwpFile, runTest, screenshot } from './helpers.mjs';

const FIXTURE = 'issue7333/aaaaaa.hwp';
const PAGE_INDEX = 7;
const PARA_INDEX = 141;
const CONTROL_INDEX = 1;

runTest('#7333 8쪽 화살표 실클릭 객체 선택', async ({ page }) => {
  const loaded = await loadHwpFile(page, FIXTURE);
  assert.equal(loaded.pageCount, 50, '50쪽 원본을 로드한다');

  const target = await page.evaluate(({ pageIndex, paraIndex, controlIndex }) => {
    const wasm = window.__wasm;
    const input = window.__inputHandler;
    const layout = wasm.getPageControlLayout(pageIndex);
    const line = layout.controls.find((control) =>
      control.type === 'line' && control.paraIdx === paraIndex && control.controlIdx === controlIndex);
    if (!line) return { error: '8쪽 화살표 line control을 찾지 못함' };
    const pageX = (line.x1 + line.x2) / 2;
    const pageY = (line.y1 + line.y2) / 2;
    const directHit = input.findPictureAtClick(pageIndex, pageX, pageY);
    const content = document.querySelector('#scroll-content');
    const viewport = content?.parentElement;
    if (!content || !viewport) return { error: 'scroll content를 찾지 못함' };
    const zoom = input.viewportManager.getZoom();
    const pageLeft = input.virtualScroll.getPageLeftResolved(pageIndex, content.clientWidth);
    const contentX = pageLeft + pageX * zoom;
    const contentY = input.virtualScroll.getPageOffset(pageIndex) + pageY * zoom;
    viewport.scrollTop = Math.max(0, contentY - 300);
    const operationProbe = { calls: 0, original: input.executeOperation };
    input.__issue7333LineClickProbe = operationProbe;
    input.executeOperation = function (...args) {
      operationProbe.calls += 1;
      return operationProbe.original.apply(this, args);
    };
    return { line, directHit, contentX, contentY };
  }, { pageIndex: PAGE_INDEX, paraIndex: PARA_INDEX, controlIndex: CONTROL_INDEX });

  assert.equal(target.error, undefined, target.error);
  assert.equal(target.directHit?.type, 'line', '화살표 중앙 hit-test가 line을 반환한다');
  await page.evaluate(() => new Promise((resolve) => setTimeout(resolve, 250)));

  const clickPoint = await page.evaluate(({ contentX, contentY }) => {
    const content = document.querySelector('#scroll-content');
    const rect = content.getBoundingClientRect();
    return { x: rect.left + contentX, y: rect.top + contentY };
  }, target);
  await page.mouse.click(clickPoint.x, clickPoint.y);
  await page.evaluate(() => new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve))));

  const result = await page.evaluate(() => {
    const input = window.__inputHandler;
    const probe = input.__issue7333LineClickProbe;
    const selected = input.cursor.getSelectedPictureRef?.();
    input.executeOperation = probe.original;
    delete input.__issue7333LineClickProbe;
    return {
      selected,
      objectSelection: input.cursor.isInPictureObjectSelection(),
      operationCalls: probe.calls,
    };
  });
  await screenshot(page, 'issue7333-page8-line-selected');

  assert.equal(result.objectSelection, true, '실클릭 뒤 객체 선택 모드다');
  assert.equal(result.selected?.type, 'line', '실클릭 뒤 선택 대상은 화살표 line이다');
  assert.equal(result.selected?.ppi, PARA_INDEX, '실클릭 뒤 원래 문단을 보존한다');
  assert.equal(result.selected?.ci, CONTROL_INDEX, '실클릭 뒤 원래 control을 보존한다');
  assert.equal(result.selected?.pageIndex, PAGE_INDEX, '실클릭 뒤 8쪽 소유를 보존한다');
  assert.equal(result.operationCalls, 0, '단순 클릭은 z-order 변경이나 Undo 항목을 만들지 않는다');

  const moveState = await page.evaluate(() => {
    const input = window.__inputHandler;
    const ref = input.cursor.getSelectedPictureRef();
    const bbox = input.findPictureBbox(ref);
    const props = input.getObjectProperties(ref);
    const content = document.querySelector('#scroll-content');
    const rect = content.getBoundingClientRect();
    const zoom = input.viewportManager.getZoom();
    const pageLeft = input.virtualScroll.getPageLeftResolved(bbox.pageIndex, content.clientWidth);
    return {
      ref,
      before: { horzOffset: props.horzOffset, vertOffset: props.vertOffset },
      x: rect.left + pageLeft + ((bbox.x1 + bbox.x2) / 2) * zoom,
      y: rect.top + input.virtualScroll.getPageOffset(bbox.pageIndex) + ((bbox.y1 + bbox.y2) / 2) * zoom,
    };
  });
  await page.mouse.move(moveState.x, moveState.y);
  await page.mouse.down();
  await page.mouse.move(moveState.x + 24, moveState.y + 18, { steps: 4 });
  await page.mouse.up();
  await page.evaluate(() => new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve))));
  const afterMove = await page.evaluate((ref) => {
    const input = window.__inputHandler;
    const props = input.getObjectProperties(ref);
    return { horzOffset: props.horzOffset, vertOffset: props.vertOffset };
  }, moveState.ref);
  assert.notDeepEqual(afterMove, moveState.before, '실제 드래그가 화살표 위치를 바꾼다');

  await page.keyboard.down('Control');
  await page.keyboard.press('z');
  await page.keyboard.up('Control');
  await page.evaluate(() => new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve))));
  const afterUndo = await page.evaluate((ref) => {
    const input = window.__inputHandler;
    const props = input.getObjectProperties(ref);
    return { horzOffset: props.horzOffset, vertOffset: props.vertOffset };
  }, moveState.ref);
  assert.deepEqual(afterUndo, moveState.before, '실제 Ctrl+Z가 화살표 위치를 원래대로 되돌린다');

  await page.mouse.click(clickPoint.x, clickPoint.y);
  await page.mouse.move(moveState.x, moveState.y);
  await page.mouse.down();
  await page.mouse.move(moveState.x + 18, moveState.y + 12, { steps: 3 });
  await page.mouse.up();
  await page.evaluate(() => new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve))));
  const afterMetaMove = await page.evaluate((ref) => {
    const props = window.__inputHandler.getObjectProperties(ref);
    return { horzOffset: props.horzOffset, vertOffset: props.vertOffset };
  }, moveState.ref);
  assert.notDeepEqual(afterMetaMove, moveState.before, '두 번째 실제 드래그도 위치를 바꾼다');

  await page.keyboard.down('Meta');
  await page.keyboard.press('KeyZ');
  await page.keyboard.up('Meta');
  await page.evaluate(() => new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve))));
  const afterMetaUndo = await page.evaluate((ref) => {
    const props = window.__inputHandler.getObjectProperties(ref);
    return { horzOffset: props.horzOffset, vertOffset: props.vertOffset };
  }, moveState.ref);
  assert.deepEqual(afterMetaUndo, moveState.before, '실제 Command+Z도 화살표 위치를 원래대로 되돌린다');
});
