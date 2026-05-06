/**
 * E2E 테스트: #598 본문 각주 삭제 확인창/취소/Undo
 */
import { runTest, loadHwpFile, screenshot, assert } from './helpers.mjs';

async function moveCursor(page, sectionIndex, paragraphIndex, charOffset) {
  await page.evaluate((sec, para, offset) => {
    const handler = window.__inputHandler;
    handler?.cursor?.moveTo?.({ sectionIndex: sec, paragraphIndex: para, charOffset: offset });
    if (handler) handler.active = true;
    handler?.focus?.();
    handler?.updateCaret?.();
  }, sectionIndex, paragraphIndex, charOffset);
  await page.evaluate(() => new Promise(r => setTimeout(r, 100)));
}

async function footnoteState(page) {
  return await page.evaluate(() => {
    const w = window.__wasm;
    const info = (sec, para, ctrl) => {
      try { return w.getFootnoteInfo(sec, para, ctrl); }
      catch { return null; }
    };
    return {
      markerP3: w.getControlTextPositions(0, 3),
      markerP7: w.getControlTextPositions(0, 7),
      fnP3: info(0, 3, 0),
      fnP7: info(0, 7, 0),
    };
  });
}

async function cursorState(page) {
  return await page.evaluate(() => {
    const cursor = window.__inputHandler?.cursor;
    const pos = cursor?.getPosition?.();
    return {
      position: pos ? {
        sectionIndex: pos.sectionIndex,
        paragraphIndex: pos.paragraphIndex,
        charOffset: pos.charOffset,
      } : null,
      inFootnote: cursor?.isInFootnote?.() ?? false,
      fnSectionIdx: cursor?.fnSectionIdx,
      fnParaIdx: cursor?.fnParaIdx,
      fnControlIdx: cursor?.fnControlIdx,
      fnInnerParaIdx: cursor?.fnInnerParaIdx,
      fnCharOffset: cursor?.fnCharOffset,
      fnFootnoteIndex: cursor?.fnFootnoteIndex,
    };
  });
}

async function exitFootnoteMode(page) {
  await page.evaluate(() => {
    const handler = window.__inputHandler;
    handler?.cursor?.exitFootnoteMode?.();
    handler?.eventBus?.emit?.('footnoteModeChanged', false);
    handler?.focus?.();
    handler?.updateCaret?.();
  });
  await page.evaluate(() => new Promise(r => setTimeout(r, 100)));
}

async function clickPagePoint(page, pageIndex, pageX, pageY) {
  const point = await page.evaluate((idx, x, y) => {
    const handler = window.__inputHandler;
    const scrollContent = document.querySelector('#scroll-content');
    if (!handler || !scrollContent) throw new Error('input handler 또는 scroll-content 없음');
    const zoom = handler.viewportManager?.getZoom?.() ?? 1;
    const rect = scrollContent.getBoundingClientRect();
    const pageOffset = handler.virtualScroll?.getPageOffset?.(idx) ?? 0;
    const pageWidth = handler.virtualScroll?.getPageWidth?.(idx) ?? 0;
    const pageLeft = (scrollContent.clientWidth - pageWidth) / 2;
    return {
      x: rect.left + pageLeft + x * zoom,
      y: rect.top + pageOffset + y * zoom,
    };
  }, pageIndex, pageX, pageY);
  await page.mouse.click(point.x, point.y);
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));
}

async function dialogText(page) {
  return await page.$eval('.modal-overlay .dialog-wrap', el => el.textContent || '');
}

async function clickDialogButton(page, label) {
  await page.evaluate((text) => {
    const buttons = Array.from(document.querySelectorAll('.modal-overlay .dialog-btn'));
    const button = buttons.find(btn => (btn.textContent || '').trim() === text);
    if (!button) throw new Error(`${text} 버튼을 찾을 수 없습니다`);
    button.click();
  }, label);
  await page.waitForFunction(() => !document.querySelector('.modal-overlay'), { timeout: 3000 });
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));
}

runTest('본문 각주 삭제 확인창/취소/Undo', async ({ page }) => {
  await loadHwpFile(page, 'footnote-01.hwp');

  const initial = await footnoteState(page);
  assert(JSON.stringify(initial.markerP3) === '[7]', '초기 첫 번째 본문 각주 마커 위치 확인');
  assert(initial.fnP3?.number === 1, '초기 첫 번째 각주 번호 확인');
  assert(initial.fnP7?.number === 2, '초기 두 번째 각주 번호 확인');

  console.log('\n[1] 좌/우 방향키 각주 마커 1칸 이동 확인...');
  await moveCursor(page, 0, 3, 7);
  await page.keyboard.press('ArrowRight');
  await page.evaluate(() => new Promise(r => setTimeout(r, 200)));
  const afterArrowRight = await cursorState(page);
  assert(afterArrowRight.position?.charOffset === 8, 'ArrowRight가 각주 마커 오른쪽으로 1칸 이동');

  await page.keyboard.press('ArrowLeft');
  await page.evaluate(() => new Promise(r => setTimeout(r, 200)));
  const afterArrowLeft = await cursorState(page);
  assert(afterArrowLeft.position?.charOffset === 7, 'ArrowLeft가 각주 마커 왼쪽으로 1칸 이동');

  console.log('\n[2] 본문 각주 마커 클릭 후 각주 편집 모드 진입 확인...');
  await clickPagePoint(page, 0, 264, 380);
  const afterMarkerClick = await cursorState(page);
  assert(afterMarkerClick.inFootnote === true, '본문 각주 마커 클릭 후 각주 편집 모드 진입');
  assert(afterMarkerClick.fnParaIdx === 3, '본문 각주 마커 클릭 후 원본 문단 인덱스 연결');
  assert(afterMarkerClick.fnControlIdx === 0, '본문 각주 마커 클릭 후 원본 control index 연결');
  assert(afterMarkerClick.fnFootnoteIndex === 0, '본문 각주 마커 클릭 후 첫 번째 각주 영역 연결');
  await exitFootnoteMode(page);

  console.log('\n[3] 각주 앞 Backspace 일반 텍스트 삭제/Undo 확인...');
  await moveCursor(page, 0, 3, 7);
  await page.keyboard.press('Backspace');
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));
  const dialogAfterPlainBackspace = await page.$('.modal-overlay .dialog-wrap');
  assert(dialogAfterPlainBackspace === null, '각주 앞 Backspace는 각주 삭제 확인창을 표시하지 않음');

  const afterPlainBackspace = await footnoteState(page);
  assert(JSON.stringify(afterPlainBackspace.markerP3) === '[6]', '각주 앞 Backspace 후 marker anchor가 이전 위치로 따라감');
  assert(afterPlainBackspace.fnP3?.number === 1, '각주 앞 Backspace 후 각주 본문 유지');

  await page.keyboard.down('Control');
  await page.keyboard.press('KeyZ');
  await page.keyboard.up('Control');
  await page.evaluate(() => new Promise(r => setTimeout(r, 500)));

  const afterPlainBackspaceUndo = await footnoteState(page);
  assert(JSON.stringify(afterPlainBackspaceUndo.markerP3) === '[7]', '각주 앞 Backspace Undo 후 marker anchor 복원');
  assert(afterPlainBackspaceUndo.fnP3?.number === 1, '각주 앞 Backspace Undo 후 각주 본문 유지');

  console.log('\n[4] Delete 취소 확인...');
  await moveCursor(page, 0, 3, 7);
  await page.keyboard.press('Delete');
  await page.waitForSelector('.modal-overlay .dialog-wrap', { timeout: 3000 });
  const cancelDialog = await dialogText(page);
  assert(cancelDialog.includes('각주를 삭제하시겠습니까?'), 'Delete 경로 확인창 메시지 표시');
  await screenshot(page, 'footnote-delete-confirm-delete');
  await clickDialogButton(page, '취소');

  const afterCancel = await footnoteState(page);
  assert(JSON.stringify(afterCancel.markerP3) === '[7]', '취소 후 첫 번째 각주 마커 유지');
  assert(afterCancel.fnP3?.number === 1, '취소 후 첫 번째 각주 유지');
  assert(afterCancel.fnP7?.number === 2, '취소 후 두 번째 각주 번호 유지');

  console.log('\n[5] Backspace 확인 후 삭제...');
  await moveCursor(page, 0, 3, 8);
  await page.keyboard.press('Backspace');
  await page.waitForSelector('.modal-overlay .dialog-wrap', { timeout: 3000 });
  const confirmDialog = await dialogText(page);
  assert(confirmDialog.includes('각주를 삭제하시겠습니까?'), 'Backspace 경로 동일 확인창 메시지 표시');
  await clickDialogButton(page, '확인');

  const afterDelete = await footnoteState(page);
  assert(JSON.stringify(afterDelete.markerP3) === '[]', '확인 후 첫 번째 각주 마커 삭제');
  assert(afterDelete.fnP3 === null, '확인 후 첫 번째 각주 본문 삭제');
  assert(afterDelete.fnP7?.number === 1, '확인 후 두 번째 각주가 1번으로 재번호화');

  console.log('\n[6] Ctrl+Z 복원...');
  await page.keyboard.down('Control');
  await page.keyboard.press('KeyZ');
  await page.keyboard.up('Control');
  await page.evaluate(() => new Promise(r => setTimeout(r, 500)));

  const afterUndo = await footnoteState(page);
  assert(JSON.stringify(afterUndo.markerP3) === '[7]', 'Undo 후 첫 번째 각주 마커 복원');
  assert(afterUndo.fnP3?.number === 1, 'Undo 후 첫 번째 각주 본문 복원');
  assert(afterUndo.fnP7?.number === 2, 'Undo 후 두 번째 각주 번호 복원');
  await screenshot(page, 'footnote-delete-confirm-undo');
});
