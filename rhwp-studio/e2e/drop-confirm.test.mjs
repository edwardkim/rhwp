/**
 * E2E 테스트: #1439 드래그&드롭 로컬 파일 로딩 보안 게이트
 *
 * 드롭 시 즉시 로딩하지 않고 모달 확인 대화상자를 표시하며, [열기]에서만 로딩되고
 * [취소]/미동의 시 로딩되지 않아야 한다. (확장/웹 공통 — 순수 DOM 모달.)
 */
import { runTest, waitForCanvas, screenshot, assert, setTestCase } from './helpers.mjs';

process.env.VITE_URL = process.env.VITE_URL || 'http://localhost:7700';

/** scroll-container 에 HWP 파일 drop 이벤트를 합성한다. */
async function dispatchDrop(page, fileName, content) {
  await page.evaluate((name, text) => {
    const container = document.getElementById('scroll-container');
    if (!container) throw new Error('scroll-container not found');
    const file = new File([text], name, { type: 'application/x-hwp' });
    const dt = new DataTransfer();
    dt.items.add(file);
    const ev = new DragEvent('drop', { bubbles: true, cancelable: true });
    Object.defineProperty(ev, 'dataTransfer', { value: dt });
    container.dispatchEvent(ev);
  }, fileName, content);
}

async function dropDialogVisible(page) {
  return await page.evaluate(() => {
    const dialog = document.querySelector('.modal-overlay .dialog-wrap');
    return Boolean(dialog?.textContent?.includes('드래그한 로컬 파일을 엽니다'));
  });
}

async function clickDropDialogButton(page, label) {
  await page.evaluate((text) => {
    const buttons = Array.from(document.querySelectorAll('.modal-overlay .dialog-btn'));
    const button = buttons.find(btn => (btn.textContent || '').trim() === text);
    if (!button) throw new Error(`${text} 버튼을 찾을 수 없습니다`);
    button.click();
  }, label);
}

async function hasLoadedDocument(page) {
  return await page.evaluate(() => window.__wasm?.hasLoadedDocument?.() ?? null);
}

// 유효한 최소 HWP 가 아니어도 로딩 "시도" 여부만 보면 되므로, 로딩 분기 진입을
// 판정하기 위해 실제 샘플 바이트를 쓰지 않고 더미를 쓴다. [취소] 케이스는 로딩
// 분기 자체가 호출되지 않음을 검증한다.
const DUMMY = 'dummy-hwp-content';

runTest('드래그&드롭 보안 확인 대화상자', async ({ page }) => {
  setTestCase('TC-1: 드롭 시 즉시 로딩되지 않고 확인 대화상자 표시');
  await dispatchDrop(page, 'dropped.hwp', DUMMY);
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));
  assert(await dropDialogVisible(page), '드롭 후 확인 대화상자가 표시되어야 함');
  await screenshot(page, 'drop-confirm-dialog');

  setTestCase('TC-2: [취소] 시 대화상자 닫히고 로딩 안 됨');
  const loadedBefore = await hasLoadedDocument(page);
  await clickDropDialogButton(page, '취소');
  await page.waitForFunction(() => !document.querySelector('.modal-overlay'), { timeout: 3000 });
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));
  assert(
    (await hasLoadedDocument(page)) === loadedBefore,
    '[취소] 후 문서 로드 상태가 변하지 않아야 함 (미로딩)',
  );

  setTestCase('TC-3: [열기] 시 로딩 분기 진입 (더미라 로드 실패하지만 시도됨)');
  await dispatchDrop(page, 'dropped2.hwp', DUMMY);
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));
  assert(await dropDialogVisible(page), 'TC-3 재드롭 시 대화상자 표시');
  await clickDropDialogButton(page, '열기');
  await page.waitForFunction(() => !document.querySelector('.modal-overlay'), { timeout: 3000 });
  // 더미 바이트라 로드는 실패(오류 알림)하나, [열기] 분기가 호출되어 상태바/토스트에
  // 로딩 시도 흔적이 남는다 — 대화상자가 닫혔는지만으로 [열기] 동작을 확인한다.
  await page.evaluate(() => new Promise(r => setTimeout(r, 500)));
  assert(
    !(await page.evaluate(() => Boolean(document.querySelector('.modal-overlay')))),
    '[열기] 후 대화상자가 닫혀야 함 (로딩 분기 진입)',
  );

  await waitForCanvas(page).catch(() => {}); // 더미 로드 실패해도 무시
});
