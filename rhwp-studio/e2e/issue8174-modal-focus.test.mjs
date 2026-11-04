/** #8174: password input must stay out of the document, including delayed editor focus. */
import { runTest, createNewDocument, assert } from './helpers.mjs';

await runTest('Issue #8174 암호 모달의 입력 소유와 중첩 모달 보존', async ({ page }) => {
  await createNewDocument(page);
  const point = await page.evaluate(() => {
    const handler = window.__inputHandler;
    const cursor = window.__wasm.getCursorRect(0, 0, 0);
    const rect = handler.formBboxToOverlayRect({ x: cursor.x, y: cursor.y, w: 10, h: cursor.height }, 0);
    const content = document.querySelector('#scroll-content').getBoundingClientRect();
    return { x: content.left + rect.left + 3, y: content.top + rect.top + 3 };
  });
  await page.mouse.click(point.x, point.y);
  const before = await page.evaluate(async () => {
    const { showHwpSavePasswordDialog } = await import('/src/ui/hwp-password-dialog.ts');
    window.__focusDialogResult = showHwpSavePasswordDialog('focus-test.hwp');
    return window.__wasm.doc.getTextFileUnicode();
  });
  await page.waitForFunction(() => document.activeElement?.id === 'hwp-save-password-input');
  await page.evaluate(() => window.__inputHandler.focus());
  await page.keyboard.type('S04_focus_marker');
  const state = await page.evaluate(before => ({
    ownsFocus: document.activeElement?.id === 'hwp-save-password-input',
    documentUnchanged: window.__wasm.doc.getTextFileUnicode() === before,
  }), before);
  assert(state.ownsFocus, '늦은 편집기 focus 호출이 암호 입력의 포커스를 빼앗지 않는다');
  assert(state.documentUnchanged, '암호 입력용 키가 문서 내용에 들어가지 않는다');
  await page.keyboard.press('Escape');
  await page.waitForSelector('#hwp-save-password-input', { hidden: true });

  await page.evaluate(async () => {
    const { ModalDialog } = await import('/src/ui/dialog.ts');
    class InputDialog extends ModalDialog {
      constructor(id) { super('Nested focus', 300); this.id = id; }
      createBody() {
        const body = document.createElement('div');
        const input = document.createElement('input'); input.id = this.id; body.append(input);
        return body;
      }
      onConfirm() { return true; }
    }
    window.__parentFocusDialog = new InputDialog('parent-focus');
    window.__childFocusDialog = new InputDialog('child-focus');
    window.__parentFocusDialog.show(); window.__childFocusDialog.show();
  });
  await page.type('#child-focus', 'child input');
  assert(await page.$eval('#child-focus', input => input.value === 'child input'), '위쪽 자식 모달의 입력을 허용한다');
  await page.evaluate(() => window.__childFocusDialog.hide());
  await page.type('#parent-focus', 'parent input');
  assert(await page.$eval('#parent-focus', input => input.value === 'parent input'), '자식을 닫으면 부모 모달의 입력을 허용한다');
  await page.evaluate(() => window.__parentFocusDialog.hide());
});
