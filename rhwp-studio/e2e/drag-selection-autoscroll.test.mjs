/**
 * E2E 테스트: 텍스트 드래그 선택 edge 자동 스크롤
 */
import {
  runTest, createNewDocument, screenshot, assert, getPageCount, getParagraphCount,
} from './helpers.mjs';

runTest('텍스트 드래그 선택 edge 자동 스크롤 테스트', async ({ page }) => {
  console.log('[1] 새 문서 생성 및 긴 문단 입력...');
  await createNewDocument(page);
  assert(await getPageCount(page) >= 1, `새 문서 페이지 수: ${await getPageCount(page)}`);

  const lines = Array.from(
    { length: 70 },
    (_, i) => `drag selection auto scroll line ${String(i + 1).padStart(2, '0')}`,
  );

  const canvas = await page.$('#scroll-container canvas');
  const box = await canvas.boundingBox();
  await page.mouse.click(box.x + 120, box.y + 140);
  await page.keyboard.type(lines.join('\n'), { delay: 0 });
  await page.evaluate(() => new Promise(r => setTimeout(r, 1500)));

  const paraCount = await getParagraphCount(page);
  console.log(`  문단 수: ${paraCount}`);
  assert(paraCount >= 60, '드래그 자동 스크롤 검증용 긴 문서가 생성되어야 함');
  await screenshot(page, 'drag-autoscroll-before');

  console.log('\n[2] 첫 줄에서 하단 edge로 드래그...');
  const dragStart = await page.evaluate(() => {
    const container = document.getElementById('scroll-container');
    const scrollContent = document.getElementById('scroll-content');
    const canvasView = window.__canvasView;
    const wasm = window.__wasm;
    if (!container || !scrollContent || !canvasView || !wasm) {
      return { error: '검증 대상 DOM 또는 전역 객체가 없습니다' };
    }

    container.scrollTop = 0;
    const zoom = canvasView.getZoom?.() ?? 1;
    const virtualScroll = canvasView.virtualScroll;
    const rect = wasm.getCursorRect(0, 0, 0);
    const contentRect = scrollContent.getBoundingClientRect();
    const containerRect = container.getBoundingClientRect();
    const pageOffset = virtualScroll.getPageOffset(rect.pageIndex);
    const pageWidth = virtualScroll.getPageWidth(rect.pageIndex);
    const pageLeft = (scrollContent.clientWidth - pageWidth) / 2;

    return {
      startX: contentRect.left + pageLeft + rect.x * zoom + 3,
      startY: contentRect.top + pageOffset + rect.y * zoom + rect.height * zoom / 2,
      bottomY: containerRect.bottom - 8,
      beforeScrollTop: container.scrollTop,
    };
  });
  if (dragStart.error) throw new Error(dragStart.error);

  await page.mouse.move(dragStart.startX, dragStart.startY);
  await page.mouse.down();
  await page.mouse.move(dragStart.startX + 180, dragStart.bottomY, { steps: 24 });
  await page.evaluate(() => new Promise(r => setTimeout(r, 1200)));
  await page.mouse.up();
  await page.evaluate(() => new Promise(r => setTimeout(r, 500)));

  const result = await page.evaluate(() => {
    const container = document.getElementById('scroll-container');
    const ih = window.__inputHandler;
    const highlights = Array.from(document.querySelectorAll('.selection-layer > div'))
      .filter((el) => getComputedStyle(el).display !== 'none');
    const selection = ih?.cursor?.getSelectionOrdered?.() ?? null;
    return {
      scrollTop: container?.scrollTop ?? 0,
      hasSelection: ih?.hasSelection?.() ?? false,
      selection,
      cursor: ih?.getCursorPosition?.() ?? null,
      highlightCount: highlights.length,
    };
  });
  console.log(`  결과: ${JSON.stringify(result)}`);
  await screenshot(page, 'drag-autoscroll-after');

  assert(result.scrollTop > dragStart.beforeScrollTop + 80, `edge 드래그 중 스크롤이 내려가야 함 (${dragStart.beforeScrollTop} → ${result.scrollTop})`);
  assert(result.hasSelection, 'edge 드래그 후 선택 상태가 유지되어야 함');
  assert(result.highlightCount > 0, `선택 하이라이트가 표시되어야 함 (${result.highlightCount})`);
  assert(result.selection?.end?.paragraphIndex >= 20, `선택 focus가 아래쪽 문단까지 확장되어야 함 (${result.selection?.end?.paragraphIndex})`);
}, { skipLoadApp: false });
