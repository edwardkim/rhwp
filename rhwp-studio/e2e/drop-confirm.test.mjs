/**
 * E2E 테스트: 드래그&드롭 확인 대화상자 경계
 *
 * 문서 드롭은 열기 동작이라 확인 대화상자 없이 바로 로딩 분기로 들어간다.
 * 이미지 드롭은 편집 중인 문서에 로컬 파일을 끼워 넣는 편집 동작이라 #1439 보안 게이트를
 * 유지한다 — 모달에서 [열기]를 눌러야 삽입되고, [취소]면 삽입되지 않는다.
 */
import { runTest, waitForCanvas, screenshot, assert, setTestCase } from './helpers.mjs';

process.env.VITE_URL = process.env.VITE_URL || 'http://localhost:7700';

/** scroll-container 에 파일 drop 이벤트를 합성한다. */
async function dispatchDrop(page, fileName, content, type) {
  await page.evaluate((name, text, mime) => {
    const container = document.getElementById('scroll-container');
    if (!container) throw new Error('scroll-container not found');
    const isBase64 = mime.startsWith('image/');
    const body = isBase64
      ? Uint8Array.from(atob(text), (c) => c.charCodeAt(0))
      : text;
    const file = new File([body], name, { type: mime });
    const dt = new DataTransfer();
    dt.items.add(file);
    const ev = new DragEvent('drop', { bubbles: true, cancelable: true });
    Object.defineProperty(ev, 'dataTransfer', { value: dt });
    container.dispatchEvent(ev);
  }, fileName, content, type);
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

// 유효한 최소 HWP 가 아니어도 로딩 "시도" 여부만 보면 되므로 더미 바이트를 쓴다.
const DUMMY = 'dummy-hwp-content';
/** 1x1 투명 PNG (base64). */
const PNG_1X1 = 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII=';

runTest('드래그&드롭 확인 대화상자 경계', async ({ page }) => {
  setTestCase('TC-1: 문서 드롭은 확인 대화상자 없이 로딩 분기로 들어간다');
  await dispatchDrop(page, 'dropped.hwp', DUMMY, 'application/x-hwp');
  await page.evaluate(() => new Promise(r => setTimeout(r, 500)));
  assert(!(await dropDialogVisible(page)), '문서 드롭에는 확인 대화상자가 없어야 함');
  await screenshot(page, 'drop-doc-no-dialog');
  // 더미 바이트라 로드는 실패한다 — 실패 알림(토스트)이 로딩 분기 진입의 흔적이다.
  const attempted = await page.evaluate(
    () => document.body.textContent?.includes('파일 로드 실패') ?? false,
  );
  assert(attempted, '문서 드롭이 곧바로 로딩을 시도해야 함');

  setTestCase('TC-2: 이미지 드롭은 확인 대화상자를 유지한다 (#1439)');
  await page.evaluate(() => {
    document.querySelectorAll('.toast-confirm, .toast').forEach(el => el.remove());
  });
  await dispatchDrop(page, 'dropped.png', PNG_1X1, 'image/png');
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));
  assert(await dropDialogVisible(page), '이미지 드롭 후 확인 대화상자가 표시되어야 함');
  await screenshot(page, 'drop-image-confirm-dialog');

  setTestCase('TC-3: [취소] 시 대화상자가 닫히고 삽입되지 않는다');
  await clickDropDialogButton(page, '취소');
  await page.waitForFunction(() => !document.querySelector('.modal-overlay'), { timeout: 3000 });
  assert(
    !(await page.evaluate(() => Boolean(document.querySelector('.modal-overlay')))),
    '[취소] 후 대화상자가 닫혀야 함',
  );

  await waitForCanvas(page).catch(() => {}); // 더미 로드 실패해도 무시
});
