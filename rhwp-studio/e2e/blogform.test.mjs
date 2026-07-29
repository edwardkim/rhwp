/**
 * E2E 테스트: BlogForm_BookReview.hwp 누름틀 안내문
 *
 * 시나리오: 문서 로드 → 누름틀 셀 클릭 → 필드 정보 확인
 */
import {
  runTest, loadHwpFile, screenshot, assert,
} from './helpers.mjs';

runTest('BlogForm_BookReview.hwp 누름틀 안내문 테스트', async ({ page }) => {
  // 1. 문서 로드
  console.log('[1] BlogForm_BookReview.hwp 파일 로드...');
  const { pageCount } = await loadHwpFile(page, 'basic/BlogForm_BookReview.hwp');
  console.log(`  페이지 수: ${pageCount}`);
  await screenshot(page, 'blogform-01-before-click');

  // 2. 누름틀 셀 클릭
  console.log('[2] 누름틀 셀 클릭...');
  const canvas = await page.$('#scroll-container canvas');
  const box = await canvas?.boundingBox();
  if (!box) {
    console.error('  canvas boundingBox is null');
    process.exitCode = 1;
    return;
  }
  console.log(`  canvas box: ${JSON.stringify(box)}`);
  const clickX = box.x + box.width * 0.65;
  const clickY = box.y + box.height * 0.12;
  console.log(`  클릭 좌표: (${clickX.toFixed(0)}, ${clickY.toFixed(0)})`);
  await page.mouse.click(clickX, clickY);
  await page.evaluate(() => new Promise(r => setTimeout(r, 1000)));
  await screenshot(page, 'blogform-02-after-click');

  // 3. 필드 정보 확인
  const fieldInfo = await page.evaluate(() => {
    const ih = window.__inputHandler;
    if (!ih) return { error: 'no inputHandler' };
    const pos = ih.getCursorPosition?.();
    const inField = ih.isInField?.() ?? false;
    return { inField, pos: { sec: pos?.sectionIndex, para: pos?.paragraphIndex, char: pos?.charOffset,
      parentPara: pos?.parentParagraphIndex, ctrlIdx: pos?.controlIndex, cellIdx: pos?.cellIndex, cellPara: pos?.cellParagraphIndex } };
  });
  console.log(`  필드 정보: ${JSON.stringify(fieldInfo)}`);

  // 4. 상태 표시줄 확인
  const statusText = await page.evaluate(() =>
    document.querySelector('#status-bar')?.textContent ?? ''
  );
  console.log(`  상태 표시줄: ${statusText}`);
});
