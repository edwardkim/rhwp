/**
 * E2E 테스트: #886 저장되지 않은 변경사항 보호 모달
 */
import { runTest, createNewDocument, typeText, screenshot, assert, setTestCase } from './helpers.mjs';

async function requestNewDocument(page) {
  await page.evaluate(() => window.__eventBus?.emit('create-new-document'));
}

async function modalText(page) {
  return await page.$eval('.modal-overlay .dialog-wrap', el => el.textContent || '');
}

async function hasUnsavedModal(page) {
  return await page.evaluate(() => {
    const dialog = document.querySelector('.modal-overlay .dialog-wrap');
    return Boolean(dialog?.textContent?.includes('저장하지 않은 변경사항'));
  });
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

async function currentText(page) {
  return await page.evaluate(() => {
    try {
      return window.__wasm?.getTextRange(0, 0, 0, 200) ?? '';
    } catch {
      return '';
    }
  });
}

runTest('저장되지 않은 변경사항 보호 모달', async ({ page }) => {
  setTestCase('TC-1: dirty 상태에서 새 문서 시도 시 모달 표시');
  await createNewDocument(page);
  await typeText(page, 'UNSAVED_GUARD_TEST');

  await requestNewDocument(page);
  await page.waitForSelector('.modal-overlay .dialog-wrap', { timeout: 3000 });

  const firstModal = await modalText(page);
  assert(firstModal.includes('저장하지 않은 변경사항'), 'dirty 상태에서 저장 확인 모달 표시');
  assert(firstModal.includes('저장') && firstModal.includes('저장 안 함') && firstModal.includes('취소'), '저장/저장 안 함/취소 버튼 표시');
  await screenshot(page, 'unsaved-guard-modal');

  setTestCase('TC-2: 취소 선택 시 기존 문서 유지');
  await clickDialogButton(page, '취소');
  assert(!await hasUnsavedModal(page), '취소 후 모달 닫힘');
  assert((await currentText(page)).includes('UNSAVED_GUARD_TEST'), '취소 후 기존 문서 내용 유지');

  setTestCase('TC-3: 저장 안 함 선택 시 새 문서 전환');
  await requestNewDocument(page);
  await page.waitForSelector('.modal-overlay .dialog-wrap', { timeout: 3000 });
  await clickDialogButton(page, '저장 안 함');
  assert(!await hasUnsavedModal(page), '저장 안 함 후 모달 닫힘');
  assert(!(await currentText(page)).includes('UNSAVED_GUARD_TEST'), '저장 안 함 후 새 문서로 전환');
  await screenshot(page, 'unsaved-guard-discard');
});
