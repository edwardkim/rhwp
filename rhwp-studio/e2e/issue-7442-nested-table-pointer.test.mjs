/** #7442: 실제 포인터 → 중첩 표 hover/resize/Undo/선택. */
import { runTest, loadApp, waitForCanvas, screenshot } from './helpers.mjs';
import assert from 'node:assert/strict';

const fixture = 'basic/issue1994_behindtext_table_20200830.hwp';
const nestedPath = [
  { controlIndex: 0, cellIndex: 10, cellParaIndex: 0 },
  { controlIndex: 0, cellIndex: 0, cellParaIndex: 0 },
];
const settle = page => page.evaluate(() => new Promise(resolve =>
  requestAnimationFrame(() => requestAnimationFrame(resolve))));

runTest('#7442 중첩 표 실제 포인터 조작', async ({ page }) => {
  await loadApp(page, `/?url=${encodeURIComponent(`/samples/${fixture}`)}`);
  await page.waitForFunction(() => window.__wasm?.fileName?.includes('issue1994'));
  await waitForCanvas(page);
  await settle(page);
  assert.equal(await page.evaluate(() => !!window.__inputHandler.cachedCellBboxes), false, 'cold cache에서 시작');

  // 조회는 좌표 조준과 결과 관측만 담당한다. 입력은 브라우저 mouse/keyboard로 보낸다.
  const snapshot = () => page.evaluate(path => ({
    inner: window.__wasm.getTableCellBboxesByPath(0, 5, JSON.stringify(path)),
    outer: window.__wasm.getTableCellBboxes(0, 5, 0, 0),
    outerProperties: window.__wasm.getTableProperties(0, 5, 0),
    outerCells: Array.from({ length: window.__wasm.getTableDimensions(0, 5, 0).cellCount }, (_, i) =>
      window.__wasm.getCellOwnProperties(0, 5, 0, i)),
  }), nestedPath);
  const point = (x, y) => page.evaluate(({ x, y }) => {
    const ih = window.__inputHandler;
    const sc = ih.container.querySelector('#scroll-content');
    const rect = sc.getBoundingClientRect();
    const zoom = ih.viewportManager.getZoom();
    return {
      x: rect.left + ih.virtualScroll.getPageLeftResolved(0, sc.clientWidth) + x * zoom,
      y: rect.top + ih.virtualScroll.getPageOffset(0) + y * zoom,
    };
  }, { x, y });
  const before = await snapshot();
  // 실제 음성 대조: 영광송 행의 '국악찬송 7장' 셀을 F5로 선택한 뒤
  // 오른쪽 경계를 움직인다. 일반 포인터 resize는 PR 이전에도 정상이므로
  // 선택 모드의 별도 입력 분기를 반드시 먼저 검사한다.
  const selectedCell = before.inner.find(c => c.pageIndex === 0 && c.cellIdx === 3);
  assert.ok(selectedCell, '영광송 행의 국악찬송 셀');
  const text = await point(selectedCell.x + selectedCell.w / 2, selectedCell.y + selectedCell.h / 2);
  await page.mouse.click(text.x, text.y);
  await page.keyboard.press('F5');
  await settle(page);
  assert.equal(await page.evaluate(() => window.__inputHandler.cursor.isInCellSelectionMode()), true);
  const selectedEdge = await point(selectedCell.x + selectedCell.w - 0.2, selectedCell.y + selectedCell.h / 2);
  await page.mouse.move(selectedEdge.x, selectedEdge.y);
  await page.mouse.down();
  await page.mouse.move(selectedEdge.x + 30, selectedEdge.y, { steps: 12 });
  await page.mouse.up();
  await settle(page);
  const selectedResize = await snapshot();
  const resizedCell = selectedResize.inner.find(c => c.pageIndex === 0 && c.cellIdx === 3);
  assert.ok(resizedCell.w > selectedCell.w + 20, 'F5 선택 상태에서도 안쪽 셀 오른쪽 경계가 이동');
  assert.deepEqual(selectedResize.outerProperties, before.outerProperties, 'F5 resize 바깥 표 모델 보존');
  assert.deepEqual(selectedResize.outerCells, before.outerCells, 'F5 resize 바깥 셀 모델 보존');
  await screenshot(page, 'issue-7442-f5-selected-resize');
  await page.keyboard.down('Control');
  await page.keyboard.press('z');
  await page.keyboard.up('Control');
  await settle(page);
  assert.deepEqual((await snapshot()).inner, before.inner, 'F5 resize Undo');
  await page.keyboard.press('Escape');
  await page.keyboard.press('Escape');
  // fixture의 빈 영역: 이전 core가 바깥 셀을 반환한 독립 재현 좌표.
  assert.equal(await page.evaluate(() => window.__wasm.hitTest(0, 59.9, 406).cellPath?.length), 2,
    '빈 영역 hit도 안쪽 표에 속한다');
  const cell = before.inner.find(c => c.pageIndex === 0 && c.col === 0 && c.row === 0);
  assert.ok(cell, '안쪽 첫 셀');
  const border = await point(cell.x + cell.w - 0.2, cell.y + cell.h / 2);
  await page.mouse.move(border.x, border.y);
  await settle(page);
  assert.equal(await page.evaluate(() => window.__inputHandler.container.style.cursor), 'col-resize');
  await screenshot(page, 'issue-7442-inner-hover');
  await page.mouse.down();
  await page.mouse.move(border.x + 4, border.y, { steps: 8 });
  await page.mouse.up();
  await settle(page);
  const after = await snapshot();
  console.log('outer rendered widths', JSON.stringify({ before: before.outer.map(c => c.w), after: after.outer.map(c => c.w) }));
  assert.ok(after.inner.some((c, i) => c.w !== before.inner[i].w), '안쪽 폭 변경');
  // 셀 안 내용의 재조판은 부모의 paint bbox를 바꿀 수 있다. 편집 대상 보존은
  // bbox가 아니라 문서 모델의 실제 표·셀 속성으로 확인한다. 위 로그에 geometry도 남긴다.
  assert.deepEqual(after.outerProperties, before.outerProperties, '바깥 표 모델 보존');
  assert.deepEqual(after.outerCells, before.outerCells, '바깥 셀 모델 보존');
  await page.keyboard.down('Control');
  await page.keyboard.press('z');
  await page.keyboard.up('Control');
  await settle(page);
  assert.deepEqual((await snapshot()).inner, before.inner, 'Undo로 안쪽 geometry 복원');

  // 좁은 칸의 중앙도 리사이즈 hit tolerance 안에 들어갈 수 있으므로
  // 블록 선택은 경계에서 충분히 떨어진 두 넓은 칸의 내부에서 시작한다.
  const selectionCell = before.inner.find(c => c.pageIndex === 0 && c.row === 0 && c.w > 40 &&
    before.inner.some(n => n.row === c.row && n.col === c.col + c.colSpan && n.w > 40));
  assert.ok(selectionCell, '블록 선택용 넓은 셀');
  const adjacent = before.inner.find(c => c.pageIndex === 0 && c.row === selectionCell.row && c.col === selectionCell.col + selectionCell.colSpan);
  assert.ok(adjacent, '안쪽 이웃 셀');
  const a = await point(selectionCell.x + selectionCell.w / 2, selectionCell.y + selectionCell.h / 2);
  const b = await point(adjacent.x + adjacent.w / 2, adjacent.y + adjacent.h / 2);
  await page.mouse.move(a.x, a.y);
  await page.mouse.down();
  await page.mouse.move(b.x, b.y, { steps: 12 });
  await page.mouse.up();
  await settle(page);
  const selection = await page.evaluate(() => ({
    active: window.__inputHandler.cursor.isInCellSelectionMode(),
    range: window.__inputHandler.cursor.getSelectedCellRange(),
  }));
  assert.equal(selection.active, true, '안쪽 셀 블록 선택 모드');
  assert.notEqual(selection.range.startCol, selection.range.endCol, '다른 셀까지 블록 선택');

  await page.keyboard.press('Escape');
  await page.keyboard.press('Escape');
  const right = Math.max(...before.inner.filter(c => c.pageIndex === 0).map(c => c.x + c.w));
  const edge = await point(right - 0.2, cell.y + cell.h / 2);
  await page.mouse.move(edge.x, edge.y);
  await settle(page);
  await page.mouse.click(edge.x, edge.y);
  await settle(page);
  const ref = await page.evaluate(() => window.__inputHandler.cursor.getSelectedTableRef());
  assert.equal(ref?.cellPath?.length, 2, '이동 없는 외곽선 클릭도 중첩 개체 선택');
  assert.deepEqual(ref.cellPath[0], nestedPath[0]);
  await screenshot(page, 'issue-7442-inner-object-selection');
});
