/**
 * #4121 머리말/꼬리말 선택 Stage 2 E2E.
 *
 * 같은 HF 정의를 쓰는 화면 밖 페이지로 스크롤했을 때 선택 overlay가 새 visible page에
 * 투영되고, 다시 돌아왔을 때도 논리 범위가 유지되는지 검증한다.
 */
import { runTest, loadHwpFile, assert } from './helpers.mjs';

process.env.VITE_URL = process.env.VITE_URL || 'http://localhost:7700';

const settle = (page, ms = 400) => page.evaluate(
  (delay) => new Promise(resolve => setTimeout(resolve, delay)),
  ms,
);

runTest('#4121 HF 선택 반복 페이지 scroll-in 투영', async ({ page }) => {
  const { pageCount } = await loadHwpFile(page, 'biz_plan.hwp');
  assert(pageCount >= 2, `전제: 반복 HF 검증에 두 쪽 이상 필요 (actual=${pageCount})`);
  await page.evaluate(() => document.querySelector('.modal-overlay .dialog-btn-primary')?.click());

  const setup = await page.evaluate(() => {
    const handler = window.__inputHandler;
    const wasm = handler.wasm;
    const startPage = 0;
    const target = wasm.getHeaderFooterEditTarget(startPage, true);
    let repeatPage = -1;
    for (let pageNum = 1; pageNum < wasm.pageCount; pageNum++) {
      const candidate = wasm.getHeaderFooterEditTarget(pageNum, true);
      if (
        candidate.sectionIndex === target.sectionIndex
        && candidate.applyTo === target.applyTo
      ) {
        repeatPage = pageNum;
        break;
      }
    }
    if (repeatPage < 0) return { error: '같은 머리말 정의를 쓰는 반복 페이지 없음' };

    // 샘플의 기존 머리말에는 필드/인라인 컨트롤이 있을 수 있다. 이 E2E는 선택 투영만
    // 판정하므로 같은 target을 빈 정의로 재생성해 텍스트 offset 축을 결정적으로 만든다.
    const existing = JSON.parse(wasm.getHeaderFooter(target.sectionIndex, true, target.applyTo));
    if (existing.exists) wasm.deleteHeaderFooter(target.sectionIndex, true, target.applyTo);
    wasm.createHeaderFooter(target.sectionIndex, true, target.applyTo);
    handler.cursor.enterHeaderFooterMode(
      true,
      target.sectionIndex,
      target.applyTo,
      startPage,
    );
    handler.cursor.setHfCursorPosition(0, 0, startPage);
    handler.eventBus.emit('headerFooterModeChanged', 'header');
    handler.viewportManager.setZoom(0.7);
    handler.focus();
    return { startPage, repeatPage, target };
  });
  assert(!setup.error, setup.error || 'HF setup');
  await settle(page);

  const initialLength = await page.evaluate(() => {
    const handler = window.__inputHandler;
    return JSON.parse(handler.wasm.getHeaderFooterParaInfo(
      handler.cursor.hfSectionIdx,
      true,
      handler.cursor.hfApplyTo,
      0,
    )).charCount;
  });
  if (initialLength === 0) {
    await page.keyboard.type('HEADER SELECT', { delay: 15 });
    await settle(page);
  }

  const dragPoints = await page.evaluate(() => {
    const handler = window.__inputHandler;
    const cursor = handler.cursor;
    const wasm = handler.wasm;
    const info = JSON.parse(wasm.getHeaderFooterParaInfo(
      cursor.hfSectionIdx,
      true,
      cursor.hfApplyTo,
      0,
    ));
    const pageNum = cursor.hfPreferredPage;
    const start = wasm.getCursorRectInHeaderFooter(
      cursor.hfSectionIdx, true, cursor.hfApplyTo, 0, 1, pageNum,
    );
    const end = wasm.getCursorRectInHeaderFooter(
      cursor.hfSectionIdx, true, cursor.hfApplyTo, 0, info.charCount - 1, pageNum,
    );
    const sc = handler.container.querySelector('#scroll-content');
    const bounds = sc.getBoundingClientRect();
    const zoom = handler.viewportManager.getZoom();
    const left = handler.virtualScroll.getPageLeftResolved(pageNum, sc.clientWidth);
    const top = handler.virtualScroll.getPageOffset(pageNum);
    const point = rect => ({
      x: bounds.left + left + rect.x * zoom,
      y: bounds.top + top + (rect.y + rect.height * 0.5) * zoom,
    });
    return { start: point(start), end: point(end), charCount: info.charCount };
  });
  assert(dragPoints.charCount >= 4, `전제: 드래그할 HF 텍스트가 충분함 (${dragPoints.charCount})`);
  await page.mouse.move(dragPoints.start.x, dragPoints.start.y);
  await page.mouse.down();
  await page.mouse.move(dragPoints.end.x, dragPoints.end.y, { steps: 8 });
  await page.mouse.up();
  await settle(page);
  const mouseSelection = await page.evaluate(() =>
    window.__inputHandler.cursor.getHeaderFooterSelectionOrdered());
  assert(
    mouseSelection !== null
      && (mouseSelection.start.paraIdx !== mouseSelection.end.paraIdx
        || mouseSelection.start.charOffset !== mouseSelection.end.charOffset),
    '실제 마우스 드래그가 HF 선택을 만든다',
  );

  await page.keyboard.press('Escape');
  await settle(page, 200);
  const afterEscape = await page.evaluate(() => ({
    inHeaderFooter: window.__inputHandler.cursor.isInHeaderFooter(),
    selection: window.__inputHandler.cursor.getHeaderFooterSelectionOrdered(),
  }));
  assert(afterEscape.inHeaderFooter, '선택이 있는 Esc는 HF 모드를 유지한다');
  assert(afterEscape.selection === null, '선택이 있는 Esc는 선택만 해제한다');

  await page.keyboard.down('Shift');
  await page.mouse.click(dragPoints.start.x, dragPoints.start.y);
  await page.keyboard.up('Shift');
  await settle(page, 200);
  const shiftClickSelection = await page.evaluate(() =>
    window.__inputHandler.cursor.getHeaderFooterSelectionOrdered());
  assert(shiftClickSelection !== null, '실제 Shift+클릭이 기존 HF 캐럿에서 선택을 확장한다');
  await page.keyboard.press('Escape');
  await settle(page, 100);

  await page.keyboard.press('Home');
  await page.keyboard.down('Shift');
  await page.keyboard.press('End');
  await page.keyboard.up('Shift');
  await settle(page);

  const selected = await page.evaluate(() => {
    const cursor = window.__inputHandler.cursor;
    return {
      selection: cursor.getHeaderFooterSelectionOrdered(),
      activeHighlights: Array.from(document.querySelectorAll('.selection-highlight'))
        .filter(el => el.style.display !== 'none').length,
    };
  });
  assert(selected.selection !== null, 'Shift+End가 HF 논리 선택을 만든다');
  assert(selected.activeHighlights > 0, '시작 페이지에 HF 선택 overlay가 보인다');

  const repeatProjection = await page.evaluate((repeatPage) => {
    const handler = window.__inputHandler;
    const vs = handler.virtualScroll;
    handler.viewportManager.setScrollTop(vs.getPageOffset(repeatPage));
    return { pageTop: vs.getPageOffset(repeatPage), pageHeight: vs.getPageHeight(repeatPage) };
  }, setup.repeatPage);
  await settle(page, 600);

  const afterScroll = await page.evaluate(({ pageTop, pageHeight }) => {
    const handler = window.__inputHandler;
    const tops = Array.from(document.querySelectorAll('.selection-highlight'))
      .filter(el => el.style.display !== 'none')
      .map(el => Number.parseFloat(el.style.top));
    return {
      selection: handler.cursor.getHeaderFooterSelectionOrdered(),
      projected: tops.some(top => top >= pageTop && top <= pageTop + pageHeight),
      tops,
    };
  }, repeatProjection);
  assert(afterScroll.selection !== null, 'scroll-in 뒤에도 HF 논리 선택이 유지된다');
  assert(afterScroll.projected, `새 visible 반복 페이지에 overlay가 투영된다 (${afterScroll.tops})`);

  await page.evaluate((startPage) => {
    const handler = window.__inputHandler;
    handler.viewportManager.setScrollTop(handler.virtualScroll.getPageOffset(startPage));
  }, setup.startPage);
  await settle(page, 600);
  const returned = await page.evaluate((startPage) => {
    const handler = window.__inputHandler;
    const top = handler.virtualScroll.getPageOffset(startPage);
    const bottom = top + handler.virtualScroll.getPageHeight(startPage);
    return Array.from(document.querySelectorAll('.selection-highlight'))
      .filter(el => el.style.display !== 'none')
      .some(el => {
        const value = Number.parseFloat(el.style.top);
        return value >= top && value <= bottom;
      });
  }, setup.startPage);
  assert(returned, '시작 페이지로 돌아오면 같은 선택 overlay가 다시 보인다');
});
