/**
 * E2E 테스트 (#6557): 가로 병합 셀(colSpan=2)이 있는 표에서 행 경계 마우스 드래그.
 *
 * 2x3 표에서 row0 cols0-1 을 가로 병합하고 전 셀 높이를 키운 뒤, 셀 선택
 * 진입(경계 mousedown 이 bbox 캐시를 채우는 유일한 경로) 상태에서 row0|row1
 * 경계를 아래로 드래그한다. 행 경계는 셀 선택과 무관하게 일반 보상 분기를
 * 타는데, 아래 이웃 보상이 병합 셀의 시작 열 이웃만 쓸면 (1,1) 이 -delta 를
 * 받지 못해 드래그가 절반만 먹고 표 전체 높이가 불어난다 (실측 194.2px →
 * 208.2px). 걸친 모든 열의 이웃이 보상을 받아야 표 높이가 보존된다.
 */
import { runTest, createNewDocument, clickEditArea, screenshot, assert } from './helpers.mjs';

await runTest('가로 병합 행 경계 드래그 — 걸친 모든 열이 보상을 받는다 (#6557)', async ({ page }) => {
  await createNewDocument(page);
  await clickEditArea(page);
  await page.evaluate(() => new Promise(r => setTimeout(r, 400)));
  await page.evaluate(() => {
    const btn = [...document.querySelectorAll('button')].find(b => b.textContent?.includes('시작하기'));
    btn?.click();
  });
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));

  const setup = await page.evaluate(async () => {
    const wasm = window.__wasm;
    const nextFrame = () => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)));
    const created = wasm.createTable(0, 0, 0, 2, 3);
    if (!created?.ok) return { error: `createTable 실패: ${JSON.stringify(created)}` };
    const merged = wasm.mergeTableCells(0, created.paraIdx, created.controlIdx, 0, 0, 0, 1);
    window.__canvasView?.loadDocument?.();
    await nextFrame(); await nextFrame();
    // 빈 표는 행 높이가 텍스트 최소값이라 delta 가 짓눌린다 — 전 셀을 먼저 키운다
    const grow = (wasm.getTableCellBboxes(0, created.paraIdx, created.controlIdx, 0) || [])
      .map(b => ({ cellIdx: b.cellIdx, heightDelta: 6000 }));
    wasm.resizeTableCells(0, created.paraIdx, created.controlIdx, grow);
    window.__canvasView?.loadDocument?.();
    await nextFrame(); await nextFrame();
    const bboxes = wasm.getTableCellBboxes(0, created.paraIdx, created.controlIdx, 0) || [];
    const c12 = bboxes.find(b => b.row === 1 && b.col === 2);
    return { created, merged, hasMerged: bboxes.some(b => b.colSpan === 2 && b.row === 0), c12Idx: c12?.cellIdx ?? null };
  });
  assert(!setup.error, `표 생성/병합: ${setup.error || 'ok'}`);
  assert(setup.merged?.ok && setup.hasMerged, '가로 병합 셀(colSpan=2, row0) 준비됨');
  assert(setup.c12Idx !== null, '셀(1,2) 존재');

  const sel = await page.evaluate(async ({ paraIdx, controlIdx, c12Idx }) => {
    const ih = window.__inputHandler;
    const nextFrame = () => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)));
    ih.cursor.moveToCellByIndex(0, paraIdx, controlIdx, undefined, c12Idx, 'start');
    await nextFrame();
    if (!ih.cursor.enterCellSelectionMode()) return { error: 'enterCellSelectionMode 실패' };
    ih.updateCellSelection();
    await nextFrame();
    return { inCellSel: ih.cursor.isInCellSelectionMode() };
  }, { paraIdx: setup.created.paraIdx, controlIdx: setup.created.controlIdx, c12Idx: setup.c12Idx });
  assert(!sel.error && sel.inCellSel, `셀 선택 진입: ${sel.error || 'ok'}`);

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
      .map(b => ({ cellIdx: b.cellIdx, row: b.row, col: b.col, colSpan: b.colSpan, y: b.y, h: b.h }))
      .sort((a, b) => a.cellIdx - b.cellIdx);
    const before = snapshot();

    // 좌표는 렌더 배율·스크롤에 좌우되므로 실제 hitTest 로 프로브해 찾는다
    const hit = (x, y) => ih.hitTestCellRowCol(me('mousemove', x, y, 0));
    const c02full = (wasm.getTableCellBboxes(0, paraIdx, controlIdx, 0) || []).find(b => b.row === 0 && b.col === 2);
    if (!c02full) return { error: '(0,2) bbox 없음' };
    const colX = rect.left + c02full.x + c02full.w / 2;
    const estY = rect.top + c02full.y + c02full.h / 2;
    let lastR0 = null; let firstR1 = null;
    for (let y = estY - 20; y <= estY + c02full.h + 60; y += 1) {
      const h = hit(colX, y);
      if (h && h.row === 0 && h.col === 2) lastR0 = y;
      if (h && h.row === 1) { firstR1 = y; break; }
    }
    if (lastR0 === null || firstR1 === null) return { error: '행 경계 프로브 실패' };
    const by = (lastR0 + firstR1) / 2;

    ih.onMouseMoveBound(me('mousemove', colX, by, 0));
    await nextFrame(); await nextFrame();
    ih.onClickBound(me('mousedown', colX, by, 1));
    const dragging = ih.isResizeDragging === true;
    ih.onMouseMoveBound(me('mousemove', colX, by + 20, 1));
    await nextFrame();
    ih.onMouseMoveBound(me('mousemove', colX, by + 40, 1));
    await nextFrame();
    ih.onMouseUpBound(me('mouseup', colX, by + 40, 0));
    await nextFrame(); await nextFrame();
    return { dragging, before, after: snapshot() };
  }, { paraIdx: setup.created.paraIdx, controlIdx: setup.created.controlIdx });
  assert(!drag.error, `드래그: ${drag.error || 'ok'}`);
  assert(drag.dragging, 'mousedown 에서 리사이즈 드래그 시작됨');

  await screenshot(page, 'merged-cell-row-boundary-drag-after');

  const pick = (rows, r, c) => rows.find(b => b.row === r && b.col === c);
  const totalBefore = pick(drag.before, 0, 2).h + pick(drag.before, 1, 2).h;
  const totalAfter = pick(drag.after, 0, 2).h + pick(drag.after, 1, 2).h;
  assert(
    Math.abs(totalBefore - totalAfter) < 1.0,
    `보상 드래그는 표 전체 높이를 보존한다 (${totalBefore.toFixed(1)} → ${totalAfter.toFixed(1)}) — ` +
      '불어나면 병합 셀이 걸친 열의 이웃이 보상에서 빠진 것',
  );
  const row1 = [0, 1, 2].map(c => pick(drag.after, 1, c));
  assert(row1.every(b => b && Math.abs(b.h - row1[0].h) < 0.5), `row1 세 열 높이 균일 (${row1.map(b => b.h.toFixed(1)).join('/')})`);
  assert(row1.every(b => b && Math.abs(b.y - row1[0].y) < 0.5), 'row1 세 열 시작 y 정렬');
  const dRow0 = pick(drag.after, 0, 2).h - pick(drag.before, 0, 2).h;
  assert(dRow0 > 10, `행 경계가 실제로 이동했다 (+${dRow0.toFixed(1)}px)`);
});
