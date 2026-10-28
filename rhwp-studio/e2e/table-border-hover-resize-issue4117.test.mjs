/**
 * E2E (#4117): 새 문서에서 표를 만들고 — 셀 선택 모드 클릭 없이 —
 * 열 경계에 마우스를 올리면 리사이즈 커서가 뜨고 드래그가 실제로 동작한다.
 *
 * 수정 전에는 bbox 캐시를 셀 선택 모드 클릭만 채워서, 경계 hover 커서도
 * marker 도 드래그 시작도 전부 불발이었다 (실측: 커서 ''·Δ0.0px). hover 가
 * ensureTableCellBboxCache 로 캐시를 채우되 표 진입당 1회만 엔진을 부른다 —
 * 이동 스톰 60회 중 엔진 호출 수(≤2)를 함께 단정해 task 2010 이 막은
 * "이동마다 표 전체 재계산"이 돌아오지 않게 잠근다.
 */
import { runTest, createNewDocument, clickEditArea, screenshot, assert } from './helpers.mjs';

await runTest('셀 선택 없이 표 경계 hover→리사이즈 드래그 (#4117)', async ({ page }) => {
  await createNewDocument(page);
  await clickEditArea(page);
  await page.evaluate(() => new Promise(r => setTimeout(r, 400)));
  await page.evaluate(() => {
    const btn = [...document.querySelectorAll('button')].find(b => b.textContent?.includes('시작하기'));
    btn?.click();
  });
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));

  const hover = await page.evaluate(async () => {
    const wasm = window.__wasm;
    const ih = window.__inputHandler;
    const nextFrame = () => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)));

    const created = wasm.createTable(0, 0, 0, 3, 3);
    if (!created?.ok) return { error: `createTable 실패: ${JSON.stringify(created)}` };
    window.__canvasView?.loadDocument?.();
    await nextFrame(); await nextFrame();

    // 문서 변경 직후라 캐시는 비어 있어야 한다 — "셀 선택 없이"의 전제 확인
    const cacheEmptyAtStart = !ih.cachedCellBboxes;

    // 경계 좌표 조준: 테스트가 직접 bbox 를 읽어 화면 좌표로 변환 (제품 경로 아님)
    const bboxes = wasm.getTableCellBboxes(0, created.paraIdx, created.controlIdx, 0);
    const c00 = bboxes.find(b => b.row === 0 && b.col === 0);
    if (!c00) return { error: '(0,0) bbox 없음' };
    const sc = ih.container.querySelector('#scroll-content');
    const rect = sc.getBoundingClientRect();
    const zoom = ih.viewportManager.getZoom();
    const pageLeft = ih.virtualScroll.getPageLeftResolved(0, sc.clientWidth);
    const pageOffset = ih.virtualScroll.getPageOffset(0);
    const b = {
      x: rect.left + pageLeft + (c00.x + c00.w) * zoom,
      y: rect.top + pageOffset + (c00.y + c00.h / 2) * zoom,
    };

    const me = (type, x, y, buttons = 0) => {
      const ev = new MouseEvent(type, { button: 0, buttons, clientX: x, clientY: y, bubbles: true });
      Object.defineProperty(ev, 'target', { value: ih.container, configurable: true });
      return ev;
    };

    let engineCalls = 0;
    const orig = wasm.getTableCellBboxes.bind(wasm);
    wasm.getTableCellBboxes = (...args) => { engineCalls++; return orig(...args); };

    ih.onMouseMoveBound(me('mousemove', b.x, b.y, 0));
    await nextFrame();
    const cursorOnBorder = ih.container.style.cursor;
    for (let i = 0; i < 60; i++) {
      ih.onMouseMoveBound(me('mousemove', b.x + ((i % 7) - 3), b.y + ((i % 5) - 2) * 3, 0));
      if (i % 6 === 5) await nextFrame();
    }
    await nextFrame();
    // marker 를 그린 상태에서 멈춰 스크린샷이 잡게 한다
    ih.onMouseMoveBound(me('mousemove', b.x, b.y, 0));
    await nextFrame();
    wasm.getTableCellBboxes = orig;
    return {
      cacheEmptyAtStart, cursorOnBorder,
      cursorAfterStorm: ih.container.style.cursor,
      stormCalls: engineCalls,
      border: b,
      table: { paraIdx: created.paraIdx, controlIdx: created.controlIdx },
    };
  });
  assert(!hover.error, `시나리오 준비: ${hover.error || 'ok'}`);
  assert(hover.cacheEmptyAtStart, '시작 시 bbox 캐시가 비어 있다 (셀 선택 없음의 전제)');
  assert(hover.cursorOnBorder === 'col-resize', `경계 hover 에서 col-resize 커서 (실제: '${hover.cursorOnBorder}')`);
  assert(hover.cursorAfterStorm === 'col-resize', '이동 스톰 후에도 커서 유지');
  assert(hover.stormCalls <= 2, `이동 스톰 60회 중 엔진 호출 ≤2 (실제: ${hover.stormCalls}회)`);

  await screenshot(page, 'table-border-hover-resize-4117-hover-marker');

  const drag = await page.evaluate(async ({ border, table }) => {
    const wasm = window.__wasm;
    const ih = window.__inputHandler;
    const nextFrame = () => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)));
    const me = (type, x, y, buttons = 0) => {
      const ev = new MouseEvent(type, { button: 0, buttons, clientX: x, clientY: y, bubbles: true });
      Object.defineProperty(ev, 'target', { value: ih.container, configurable: true });
      return ev;
    };
    const snapshot = () => wasm.getTableCellBboxes(0, table.paraIdx, table.controlIdx, 0)
      .map(x => ({ row: x.row, col: x.col, w: x.w }));
    const before = snapshot();
    ih.onClickBound(me('mousedown', border.x, border.y, 1));
    const dragging = ih.isResizeDragging === true;
    ih.onMouseMoveBound(me('mousemove', border.x + 20, border.y, 1));
    await nextFrame();
    ih.onMouseMoveBound(me('mousemove', border.x + 40, border.y, 1));
    await nextFrame();
    ih.onMouseUpBound(me('mouseup', border.x + 40, border.y, 0));
    await nextFrame(); await nextFrame();
    const after = snapshot();
    const pick = (rows, r, c) => rows.find(v => v.row === r && v.col === c);
    return {
      dragging,
      dW: pick(after, 0, 0).w - pick(before, 0, 0).w,
      col0Uniform: [0, 1, 2].every(r => Math.abs(pick(after, r, 0).w - pick(after, 0, 0).w) < 0.5),
    };
  }, { border: hover.border, table: hover.table });

  await screenshot(page, 'table-border-hover-resize-4117-after-drag');

  assert(drag.dragging, '셀 선택 없이 mousedown 에서 리사이즈 드래그가 시작된다');
  assert(drag.dW > 10, `드래그로 col0 폭이 실제로 늘었다 (Δ${drag.dW.toFixed(1)}px)`);
  assert(drag.col0Uniform, 'col0 세 행의 폭이 균일하게 이동했다');
});
