/**
 * E2E 테스트 — Home/End 가 줄 처음·끝으로, Ctrl+Home/End 가 문서 처음·끝으로 간다
 *
 * 검증 항목:
 * 1. 편집기 포커스에서 Home/End 가 줄 처음·끝으로 이동한다 (회귀 없음)
 * 2. Shift+Home/End 는 선택을 만든다
 * 3. 툴바 버튼·서식 콤보로 포커스가 나가도 같은 동작이다
 *    (종전에는 편집기 textarea 가 키를 못 받아 네 키 모두 무동작이었다)
 * 4. Ctrl+Home 은 화면을 문서 맨 위에 붙인다
 *    (종전에는 캐럿만 보이면 멈춰 첫 쪽 위 여백이 잘린 채 122px 내려가 있었다)
 * 5. Ctrl+End 는 문서 끝으로 가고 캐럿이 화면 안에 남는다
 * 6. 머리말/꼬리말 편집 모드의 Home/End 는 그 모드 안에서 동작한다
 *    (종전에는 모드 분기가 키를 삼켜 무동작이었다)
 * 7. 각주 편집 모드도 같다
 */

import { runTest, loadHwpFile, assert } from './helpers.mjs';

process.env.VITE_URL = process.env.VITE_URL || 'http://localhost:7700';

const readState = () => {
  const container = document.getElementById('scroll-container');
  const handler = window.__inputHandler;
  const position = handler.cursor.getPosition();
  const rect = handler.cursor.getRect();
  const zoom = handler.viewportManager.getZoom();
  const virtualScroll = handler.virtualScroll;
  let caretVisible = null;
  if (rect) {
    const caretTop = virtualScroll.getPageOffset(rect.pageIndex) + rect.y * zoom - container.scrollTop;
    caretVisible = caretTop >= -1 && caretTop + rect.height * zoom <= container.clientHeight + 1;
  }
  return {
    top: Math.round(container.scrollTop),
    maxScroll: Math.round(container.scrollHeight - container.clientHeight),
    para: position.paragraphIndex,
    charOffset: position.charOffset,
    caretX: rect ? Math.round(rect.x) : null,
    caretVisible,
    hasSelection: handler.cursor.hasSelection(),
  };
};

runTest('Home/End 줄·문서 이동', async ({ page }) => {
  await loadHwpFile(page, 'biz_plan.hwp');
  // 첫 실행 스킨 안내가 떠 있으면 모달이 키 전파를 막는다 — 정상 경로로 닫는다.
  await page.evaluate(() => document.querySelector('.modal-overlay .dialog-btn-primary')?.click());
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));

  const state = () => page.evaluate(readState);
  const press = async (key, modifiers = []) => {
    for (const modifier of modifiers) await page.keyboard.down(modifier);
    await page.keyboard.press(key);
    for (const modifier of [...modifiers].reverse()) await page.keyboard.up(modifier);
    await page.evaluate(() => new Promise(r => setTimeout(r, 300)));
    return state();
  };

  // 글자가 충분히 있는 문단을 찾는다 — 빈 문단에서는 Home/End 가 제자리라 판정이 안 선다.
  const target = await page.evaluate(() => {
    const cursor = window.__inputHandler.cursor;
    for (let para = 0; para < 60; para++) {
      try {
        cursor.moveTo({ sectionIndex: 0, paragraphIndex: para, charOffset: 0 });
        const start = cursor.getRect();
        cursor.moveToLineEnd();
        const end = cursor.getRect();
        if (start && end && end.x - start.x > 40) {
          return { para, lineStartX: Math.round(start.x), lineEndX: Math.round(end.x) };
        }
      } catch { /* 다음 문단 */ }
    }
    return null;
  });
  assert(target !== null, '전제: 글자가 있는 문단을 찾음');

  /** 대상 문단의 줄 중간에 캐럿을 놓고 화면을 맨 위로 되돌린다. */
  const seat = async () => {
    await page.evaluate((paragraphIndex) => {
      window.__inputHandler.viewportManager.setScrollTop(0);
      window.__inputHandler.cursor.clearSelection();
      window.__inputHandler.cursor.moveTo({ sectionIndex: 0, paragraphIndex, charOffset: 1 });
      window.__inputHandler.focus();
    }, target.para);
    await page.evaluate(() => new Promise(r => setTimeout(r, 200)));
    return state();
  };

  // ── TC1: 편집기 포커스 — 줄 처음·끝 ──────────────────────
  await seat();
  const end1 = await press('End');
  assert(end1.caretX >= target.lineEndX - 1,
    `TC1: End 가 줄 끝으로 (x=${end1.caretX}, 줄 끝 ${target.lineEndX})`);
  const home1 = await press('Home');
  assert(home1.caretX <= target.lineStartX + 1 && home1.charOffset === 0,
    `TC1: Home 이 줄 처음으로 (x=${home1.caretX}, offset=${home1.charOffset})`);

  // ── TC2: Shift 조합은 선택을 만든다 ───────────────────────
  await seat();
  const shiftEnd = await press('End', ['Shift']);
  assert(shiftEnd.hasSelection, 'TC2: Shift+End 가 선택을 만든다');
  await seat();
  const shiftHome = await press('Home', ['Shift']);
  assert(shiftHome.hasSelection, 'TC2: Shift+Home 이 선택을 만든다');

  // ── TC3: 편집기 밖(툴바 버튼·서식 콤보) 포커스 ────────────
  await seat();
  await page.evaluate(() => document.querySelector('.tb-btn[data-cmd]')?.focus());
  const toolbarEnd = await press('End');
  assert(toolbarEnd.caretX === end1.caretX,
    `TC3: 툴바 포커스에서도 End 가 줄 끝으로 (x=${toolbarEnd.caretX}, 기준 ${end1.caretX})`);
  const toolbarHome = await press('Home');
  assert(toolbarHome.charOffset === 0,
    `TC3: 툴바 포커스에서도 Home 이 줄 처음으로 (offset=${toolbarHome.charOffset})`);

  await seat();
  await page.evaluate(() => document.querySelector('select')?.focus());
  const comboEnd = await press('End');
  assert(comboEnd.caretX === end1.caretX,
    `TC3: 서식 콤보 포커스에서도 End 가 줄 끝으로 (x=${comboEnd.caretX}, 기준 ${end1.caretX})`);

  // ── TC4/TC5: Ctrl+End / Ctrl+Home ─────────────────────────
  await seat();
  const ctrlEnd = await press('End', ['Control']);
  assert(ctrlEnd.para > target.para,
    `TC5: Ctrl+End 가 문서 끝으로 (문단 ${target.para} → ${ctrlEnd.para})`);
  assert(ctrlEnd.caretVisible, `TC5: Ctrl+End 뒤 캐럿이 화면 안 (top=${ctrlEnd.top})`);

  const ctrlHome = await press('Home', ['Control']);
  assert(ctrlHome.para === 0 && ctrlHome.charOffset === 0,
    `TC4: Ctrl+Home 이 문서 처음으로 (문단 ${ctrlHome.para}:${ctrlHome.charOffset})`);
  assert(ctrlHome.top === 0,
    `TC4: Ctrl+Home 이 화면을 문서 맨 위에 붙임 (top=${ctrlHome.top})`);

  // 편집기 밖 포커스에서도 같다
  await seat();
  await page.evaluate(() => document.querySelector('.tb-btn[data-cmd]')?.focus());
  const toolbarCtrlEnd = await press('End', ['Control']);
  assert(toolbarCtrlEnd.para === ctrlEnd.para,
    `TC3: 툴바 포커스에서도 Ctrl+End 동작 (문단 ${toolbarCtrlEnd.para}, 기준 ${ctrlEnd.para})`);
  const toolbarCtrlHome = await press('Home', ['Control']);
  assert(toolbarCtrlHome.top === 0 && toolbarCtrlHome.para === 0,
    `TC3: 툴바 포커스에서도 Ctrl+Home 동작 (top=${toolbarCtrlHome.top}, 문단 ${toolbarCtrlHome.para})`);

  // ── TC6: 머리말 편집 모드 ─────────────────────────────────
  const headerState = () => page.evaluate(() => ({
    inHeaderFooter: window.__inputHandler.cursor.isInHeaderFooter(),
    charOffset: window.__inputHandler.cursor.hfCharOffset,
  }));
  const enteredHeader = await page.evaluate(() => {
    const handler = window.__inputHandler;
    handler.cursor.enterHeaderFooterMode(true, 0, 0);
    handler.cursor.setHfCursorPosition(0, 0);
    handler.focus();
    return handler.cursor.isInHeaderFooter();
  });
  assert(enteredHeader, 'TC6: 머리말 편집 모드 진입');
  await page.evaluate(() => new Promise(r => setTimeout(r, 200)));

  // 번들 샘플의 머리말은 모두 비어 있어 Home/End 가 제자리다 — 글자를 먼저 넣는다.
  await page.keyboard.type('HEADER', { delay: 30 });
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));
  const headerTyped = await headerState();
  assert(headerTyped.charOffset > 0,
    `TC6: 머리말에 글자 입력 (offset=${headerTyped.charOffset})`);

  await press('Home');
  const headerAfterHome = await headerState();
  assert(headerAfterHome.inHeaderFooter, 'TC6: Home 뒤에도 머리말 편집 문맥 유지');
  assert(headerAfterHome.charOffset === 0,
    `TC6: Home 이 머리말 줄 처음으로 (offset=${headerAfterHome.charOffset})`);

  await press('End');
  const headerAfterEnd = await headerState();
  assert(headerAfterEnd.charOffset === headerTyped.charOffset,
    `TC6: End 가 머리말 줄 끝으로 (offset=${headerAfterEnd.charOffset}, 기준 ${headerTyped.charOffset})`);
  await page.evaluate(() => window.__inputHandler.cursor.exitHeaderFooterMode?.());

  // ── TC7: 각주 편집 모드 ───────────────────────────────────
  await loadHwpFile(page, 'footnote-01.hwp');
  const footnoteState = () => page.evaluate(() => ({
    inFootnote: window.__inputHandler.cursor.isInFootnote(),
    charOffset: window.__inputHandler.cursor.fnCharOffset,
  }));
  const enteredFootnote = await page.evaluate(() => {
    const handler = window.__inputHandler;
    handler.cursor.enterFootnoteMode(0, 3, 0, 0, 0);
    handler.eventBus.emit('footnoteModeChanged', true);
    handler.cursor.setFnCursorPosition(0, 0);
    handler.active = true;
    handler.focus();
    handler.updateCaret();
    return handler.cursor.isInFootnote();
  });
  assert(enteredFootnote, 'TC7: 각주 편집 모드 진입');
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));

  await press('End');
  const footnoteAfterEnd = await footnoteState();
  assert(footnoteAfterEnd.inFootnote, 'TC7: End 뒤에도 각주 편집 문맥 유지');
  assert(footnoteAfterEnd.charOffset > 0,
    `TC7: End 가 각주 줄 끝으로 (offset=${footnoteAfterEnd.charOffset})`);

  await press('Home');
  const footnoteAfterHome = await footnoteState();
  assert(footnoteAfterHome.charOffset === 0,
    `TC7: Home 이 각주 줄 처음으로 (offset=${footnoteAfterHome.charOffset})`);
});
