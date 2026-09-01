/**
 * E2E 테스트 (#6557): 병합 셀이 섞인 표에서 셀 선택 후 열 경계 마우스 드래그.
 *
 * 3x2 표에서 col0 rows0-1 을 세로 병합하고, 셀(2,1)에서 F5 → 범위 확장으로
 * rows1..2 x cols0..1 을 선택한 뒤 col0|col1 경계를 +48px 드래그한다.
 * 결함 3층이 겹친 시나리오였다:
 *  - finishResizeDrag 선택 필터가 시작 좌표 비교라 병합 셀 누락
 *  - 오른쪽 이웃 보상이 병합 셀 시작 행만 쓸어 행별 폭 합 불일치
 *  - engine 이 균일 결과에도 local_resize 행을 마킹해 base grid 에서
 *    col0 폭 소스가 사라짐 (병합 셀 폭 279.7px → 24px 붕괴 실측)
 * 세 층이 모두 고쳐졌을 때만 경계가 전 행에서 같은 x 로 이동한다.
 */
import { runTest, createNewDocument, clickEditArea, screenshot, assert } from './helpers.mjs';

await runTest('병합 셀 경계 드래그 — 하위 행 선택에서도 경계가 함께 움직인다 (#6557)', async ({ page }) => {
  await createNewDocument(page);
  await clickEditArea(page);
  await page.evaluate(() => new Promise(r => setTimeout(r, 400)));

  // 첫 실행 화면 스킨 다이얼로그가 캔버스를 가리면 닫는다
  await page.evaluate(() => {
    const btn = [...document.querySelectorAll('button')].find(b => b.textContent?.includes('시작하기'));
    btn?.click();
  });
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));

  const setup = await page.evaluate(async () => {
    const wasm = window.__wasm;
    const nextFrame = () => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)));
    const created = wasm.createTable(0, 0, 0, 3, 2);
    if (!created?.ok) return { error: `createTable 실패: ${JSON.stringify(created)}` };
    const merged = wasm.mergeTableCells(0, created.paraIdx, created.controlIdx, 0, 0, 1, 0);
    window.__canvasView?.loadDocument?.();
    await nextFrame(); await nextFrame();
    const bboxes = wasm.getTableCellBboxes(0, created.paraIdx, created.controlIdx, 0) || [];
    const cell21 = bboxes.find(b => b.row === 2 && b.col === 1);
    return { created, merged, hasMerged: bboxes.some(b => b.rowSpan === 2 && b.col === 0), cell21Idx: cell21?.cellIdx ?? null };
  });
  assert(!setup.error, `표 생성/병합: ${setup.error || 'ok'}`);
  assert(setup.merged?.ok && setup.hasMerged, '세로 병합 셀(rowSpan=2, col0) 준비됨');
  assert(setup.cell21Idx !== null, '셀(2,1) 존재');

  const sel = await page.evaluate(async ({ paraIdx, controlIdx, cell21Idx }) => {
    const ih = window.__inputHandler;
    const nextFrame = () => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)));
    ih.cursor.moveToCellByIndex(0, paraIdx, controlIdx, undefined, cell21Idx, 'start');
    await nextFrame();
    if (!ih.cursor.isInCell()) return { error: 'moveToCellByIndex 후 커서가 셀 밖' };
    if (!ih.cursor.enterCellSelectionMode()) return { error: 'enterCellSelectionMode 실패' };
    ih.cursor.advanceCellSelectionPhase();
    ih.cursor.expandCellSelection(-1, -1);
    ih.updateCellSelection();
    await nextFrame();
    return { range: ih.cursor.getSelectedCellRange() };
  }, { paraIdx: setup.created.paraIdx, controlIdx: setup.created.controlIdx, cell21Idx: setup.cell21Idx });
  assert(!sel.error, `셀 선택: ${sel.error || 'ok'}`);
  assert(
    sel.range && sel.range.startRow === 1 && sel.range.endRow === 2 && sel.range.startCol === 0 && sel.range.endCol === 1,
    `선택 범위 rows1..2 x cols0..1 (실제: ${JSON.stringify(sel.range)})`,
  );

  const drag = await page.evaluate(async ({ paraIdx, controlIdx }) => {
    const wasm = window.__wasm;
    const ih = window.__inputHandler;
    const nextFrame = () => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)));
    const me = (type, x, y, buttons = 0) => {
      const ev = new MouseEvent(type, { button: 0, buttons, clientX: x, clientY: y, bubbles: true });
      Object.defineProperty(ev, 'target', { value: ih.container, configurable: true });
      return ev;
    };
    const sc = ih.container.querySelector('#scroll-content');
    const rect = sc.getBoundingClientRect();
    const snapshot = () => (wasm.getTableCellBboxes(0, paraIdx, controlIdx, 0) || [])
      .map(b => ({ cellIdx: b.cellIdx, row: b.row, col: b.col, rowSpan: b.rowSpan, x: b.x, y: b.y, w: b.w, h: b.h }))
      .sort((a, b) => a.cellIdx - b.cellIdx);
    const before = snapshot();
    const merged = before.find(b => b.rowSpan === 2 && b.col === 0);
    const row2c0 = before.find(b => b.row === 2 && b.col === 0);
    if (!merged || !row2c0) return { error: '병합 셀/row2col0 bbox 없음' };

    // 좌표는 렌더 배율·스크롤에 좌우되므로 실제 hitTest 로 프로브해 찾는다
    const hit = (x, y) => ih.hitTestCellRowCol(me('mousemove', x, y, 0));
    const estX = rect.left + row2c0.x + row2c0.w / 2;
    const estY = rect.top + row2c0.y + row2c0.h / 2;
    let yRow2 = null;
    for (let dy = 0; dy <= 40 && yRow2 === null; dy += 2) {
      for (const s of dy === 0 ? [0] : [-1, 1]) {
        const h = hit(estX, estY + s * dy);
        if (h && h.row === 2 && h.col === 0) { yRow2 = estY + s * dy; break; }
      }
    }
    if (yRow2 === null) return { error: 'row2 y 프로브 실패' };
    let lastC0 = null; let firstC1 = null;
    for (let x = estX; x <= estX + row2c0.w + 60; x += 1) {
      const h = hit(x, yRow2);
      if (h && h.row === 2 && h.col === 0) lastC0 = x;
      if (h && h.col === 1) { firstC1 = x; break; }
    }
    if (lastC0 === null || firstC1 === null) return { error: '경계 x 프로브 실패' };
    const bx = (lastC0 + firstC1) / 2;

    ih.onMouseMoveBound(me('mousemove', bx, yRow2, 0));
    await nextFrame(); await nextFrame();
    ih.onClickBound(me('mousedown', bx, yRow2, 1));
    const dragging = ih.isResizeDragging === true;
    ih.onMouseMoveBound(me('mousemove', bx + 24, yRow2, 1));
    await nextFrame();
    ih.onMouseMoveBound(me('mousemove', bx + 48, yRow2, 1));
    await nextFrame();
    ih.onMouseUpBound(me('mouseup', bx + 48, yRow2, 0));
    await nextFrame(); await nextFrame();
    return { dragging, before, after: snapshot() };
  }, { paraIdx: setup.created.paraIdx, controlIdx: setup.created.controlIdx });
  assert(!drag.error, `드래그: ${drag.error || 'ok'}`);
  assert(drag.dragging, 'mousedown 에서 리사이즈 드래그 시작됨');

  await screenshot(page, 'merged-cell-boundary-drag-after');

  const pick = (rows, pred) => rows.find(pred);
  const bMerged = pick(drag.before, b => b.rowSpan === 2);
  const aMerged = pick(drag.after, b => b.rowSpan === 2);
  const bRow2 = pick(drag.before, b => b.row === 2 && b.col === 0);
  const aRow2 = pick(drag.after, b => b.row === 2 && b.col === 0);
  const dMerged = aMerged.w - bMerged.w;
  const dRow2 = aRow2.w - bRow2.w;

  assert(dRow2 > 20, `row2 col0 폭이 드래그만큼 늘었다 (+${dRow2.toFixed(1)}px)`);
  assert(
    Math.abs(dMerged - dRow2) < 1.0,
    `병합 셀도 같은 delta 를 받았다 (병합 ${dMerged.toFixed(1)}px vs row2 ${dRow2.toFixed(1)}px) — ` +
      '어긋나면 하위 행 선택에서 병합 셀이 누락된 것',
  );
  assert(
    Math.abs((aMerged.x + aMerged.w) - (aRow2.x + aRow2.w)) < 1.0,
    `col0|col1 경계가 전 행에서 정렬됐다 (병합 ${(aMerged.x + aMerged.w).toFixed(1)} vs row2 ${(aRow2.x + aRow2.w).toFixed(1)})`,
  );
});
