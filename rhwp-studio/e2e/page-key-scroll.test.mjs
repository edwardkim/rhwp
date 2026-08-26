/**
 * E2E 테스트 — PgUp/PgDn 이 화면을 쪽 단위로 스크롤하고 캐럿을 데려간다
 *
 * 검증 항목:
 * 1. 편집기 포커스에서 PgDn/PgUp 이 쪽 단위로 움직이고, 한 걸음이 화면을 넘지 않는다
 *    (종전에는 다음 쪽 머리로 뛰어 확대 상태에서 쪽 중간을 통째로 건너뛰었다)
 * 2. 계속 누르면 모든 쪽 머리에 정확히 착지한다
 * 3. 캐럿이 화면을 따라온다 — 이동 후 방향키를 눌러도 원래 쪽으로 되튀지 않는다
 *    (종전에는 화면만 옮겨 다음 입력 한 번에 이동이 취소된 것처럼 보였다)
 * 4. 툴바 버튼·서식 콤보로 포커스가 나가도 같은 동작이다
 *    (종전에는 편집기 textarea 가 키를 못 받아 무동작이었다)
 * 5. 머리말 편집 같은 하위 모드에서도 키를 삼키지 않되, 캐럿은 그 모드에 남는다
 * 6. Shift+PgDn 은 선택을 확장한다
 * 7. 문서 끝/처음에서는 남은 부분까지 붙이고 멈춘다
 */

import { runTest, loadHwpFile, assert } from './helpers.mjs';

process.env.VITE_URL = process.env.VITE_URL || 'http://localhost:7700';

const SAMPLE = 'biz_plan.hwp';

const readState = () => {
  const container = document.getElementById('scroll-container');
  const handler = window.__inputHandler;
  const position = handler?.cursor?.getPosition?.();
  const rect = handler?.cursor?.getRect?.();
  return {
    top: Math.round(container.scrollTop),
    viewportHeight: container.clientHeight,
    maxScroll: Math.round(container.scrollHeight - container.clientHeight),
    position: position ? JSON.stringify(position) : null,
    caretPageIndex: rect ? rect.pageIndex : null,
    hasSelection: handler?.cursor?.hasSelection?.() ?? false,
  };
};

/** 각 쪽 머리가 시작되는 스크롤 위치(= 그 쪽의 위 여백부터 보이는 자리). */
const readPageTops = () => {
  const vs = window.__canvasView.virtualScroll;
  const tops = [];
  for (let page = 0; page < vs.pageCount; page += vs.pagesPerRow) {
    tops.push(Math.round(vs.getPageOffset(page) - vs.gap));
  }
  return tops;
};

runTest('PgUp/PgDn 쪽 단위 스크롤', async ({ page }) => {
  await loadHwpFile(page, SAMPLE);
  // 첫 실행 스킨 안내가 떠 있으면 모달이 키 전파를 막는다 — 정상 경로로 닫는다.
  await page.evaluate(() => document.querySelector('.modal-overlay .dialog-btn-primary')?.click());
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));

  const state = () => page.evaluate(readState);
  const press = async (key, options = {}) => {
    const before = await state();
    if (options.shift) await page.keyboard.down('Shift');
    await page.keyboard.press(key);
    if (options.shift) await page.keyboard.up('Shift');
    await page.evaluate(() => new Promise(r => setTimeout(r, 250)));
    return { before, after: await state() };
  };
  // 앱이 쓰는 경로로 되돌린다 — DOM scrollTop 에 직접 대입하면 ViewportManager 의 캐시가
  // scroll 이벤트가 돌 때까지 옛 값으로 남아, 다음 키가 옛 위치를 기준으로 계산된다.
  const resetToTop = async () => {
    await page.evaluate(() => {
      window.__inputHandler.viewportManager.setScrollTop(0);
      window.__inputHandler.cursor.moveToDocumentStart();
      window.__inputHandler.focus();
    });
    await page.evaluate(() => new Promise(r => setTimeout(r, 150)));
  };

  const pageTops = await page.evaluate(readPageTops);
  assert(pageTops.length > 1, `전제: 여러 쪽 문서 (${pageTops.length}쪽)`);

  // ── TC1: 편집기 포커스 — 아래로 움직이되 화면을 건너뛰지 않는다 ──
  await resetToTop();
  const down1 = await press('PageDown');
  assert(down1.after.top > down1.before.top,
    `TC1: PgDn 이 아래로 이동 (${down1.before.top} → ${down1.after.top})`);
  assert(down1.after.top - down1.before.top <= down1.before.viewportHeight,
    `TC1: 한 걸음이 화면(${down1.before.viewportHeight})을 넘지 않음 (${down1.after.top - down1.before.top})`);

  const up1 = await press('PageUp');
  assert(up1.after.top < up1.before.top,
    `TC1: PgUp 이 위로 이동 (${up1.before.top} → ${up1.after.top})`);

  // ── TC2: 계속 누르면 모든 쪽 머리에 착지 ──────────────────
  await resetToTop();
  const visited = new Set([0]);
  let steps = 0;
  for (let previous = -1; previous !== (await state()).top && steps < 60; steps++) {
    previous = (await state()).top;
    const moved = await press('PageDown');
    assert(moved.after.top - moved.before.top <= moved.before.viewportHeight,
      `TC2: ${steps + 1}번째 걸음이 화면을 넘지 않음 (${moved.after.top - moved.before.top})`);
    visited.add(moved.after.top);
  }
  const missed = pageTops.filter(top => !visited.has(top));
  assert(missed.length === 0,
    `TC2: 모든 쪽 머리에 착지 (놓친 위치: ${missed.join(', ') || '없음'})`);

  // ── TC3: 캐럿이 화면을 따라온다 ───────────────────────────
  await resetToTop();
  const caretStart = await state();
  const caretMoved = await press('PageDown');
  assert(caretMoved.after.position !== caretStart.position,
    `TC3: PgDn 이 캐럿도 옮김 (${caretStart.position} → ${caretMoved.after.position})`);

  // 캐럿이 화면 밖이면 다음 입력 한 번에 화면이 되튄다 — 그게 종전 증상이었다.
  const afterArrow = await press('ArrowRight');
  assert(afterArrow.after.top === afterArrow.before.top,
    `TC3: 방향키를 눌러도 화면이 되튀지 않음 (${afterArrow.before.top} → ${afterArrow.after.top})`);

  const caretBack = await press('PageUp');
  assert(caretBack.after.position !== afterArrow.after.position,
    `TC3: PgUp 도 캐럿을 옮김 (${caretBack.after.position})`);

  // ── TC4: 편집기 밖(툴바 버튼·서식 콤보) 포커스 ────────────
  await resetToTop();
  const focusBaseline = await press('PageDown');
  await resetToTop();
  await page.evaluate(() => document.querySelector('.tb-btn[data-cmd]')?.focus());
  const toolbarDown = await press('PageDown');
  assert(toolbarDown.after.top === focusBaseline.after.top,
    `TC4: 툴바 포커스에서도 같은 자리로 이동 (${toolbarDown.after.top}, 기준 ${focusBaseline.after.top})`);
  assert(toolbarDown.after.position === focusBaseline.after.position,
    `TC4: 툴바 포커스에서도 캐럿이 따라옴 (${toolbarDown.after.position})`);

  await resetToTop();
  await page.evaluate(() => document.querySelector('select')?.focus());
  const comboDown = await press('PageDown');
  assert(comboDown.after.top === focusBaseline.after.top,
    `TC4: 서식 콤보 포커스에서도 같은 자리로 이동 (${comboDown.after.top})`);

  // ── TC5: 머리말 편집 모드 — 화면만 옮기고 문맥은 유지 ─────
  await resetToTop();
  const inHeaderFooter = await page.evaluate(() => {
    window.__inputHandler.cursor.enterHeaderFooterMode(true, 0, 0);
    return window.__inputHandler.cursor.isInHeaderFooter();
  });
  assert(inHeaderFooter, 'TC5: 머리말 편집 모드 진입');
  const headerDown = await press('PageDown');
  assert(headerDown.after.top === focusBaseline.after.top,
    `TC5: 머리말 편집 모드에서도 화면이 이동 (${headerDown.after.top}, 기준 ${focusBaseline.after.top})`);
  const stillInHeaderFooter = await page.evaluate(
    () => window.__inputHandler.cursor.isInHeaderFooter(),
  );
  assert(stillInHeaderFooter, 'TC5: 머리말 편집 문맥은 그대로 유지');
  await page.evaluate(() => window.__inputHandler.cursor.exitHeaderFooterMode?.());

  // ── TC6: Shift+PgDn 은 선택을 확장한다 ────────────────────
  await resetToTop();
  const shiftDown = await press('PageDown', { shift: true });
  assert(shiftDown.after.hasSelection,
    'TC6: Shift+PgDn 이 선택을 만든다');
  assert(shiftDown.after.top > shiftDown.before.top,
    `TC6: Shift+PgDn 도 화면을 옮긴다 (${shiftDown.before.top} → ${shiftDown.after.top})`);

  // ── TC7: 문서 끝/처음 경계 ────────────────────────────────
  await resetToTop();
  let previous = -1;
  for (let i = 0; i < 60 && previous !== (await state()).top; i++) {
    previous = (await state()).top;
    await press('PageDown');
  }
  const atEnd = await state();
  assert(atEnd.top === atEnd.maxScroll,
    `TC7: 마지막 쪽 아래 끝까지 도달 (${atEnd.top}, 최대 ${atEnd.maxScroll})`);

  previous = -1;
  for (let i = 0; i < 60 && previous !== (await state()).top; i++) {
    previous = (await state()).top;
    await press('PageUp');
  }
  const atStart = await state();
  assert(atStart.top === 0, `TC7: 문서 맨 위까지 복귀 (${atStart.top})`);
});
