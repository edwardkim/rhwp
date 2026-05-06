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

  console.log('\n[1] 각주 앞 Backspace 일반 텍스트 삭제/Undo 확인...');
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

  console.log('\n[2] Delete 취소 확인...');
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

  console.log('\n[3] Backspace 확인 후 삭제...');
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

  console.log('\n[4] Ctrl+Z 복원...');
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
