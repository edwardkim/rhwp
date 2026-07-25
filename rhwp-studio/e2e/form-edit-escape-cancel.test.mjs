/**
 * E2E 회귀: 편집 양식 필드에서 Escape는 취소여야 한다 (#2375).
 *
 * Escape가 input을 지우면 blur가 뒤따른다. 따라서 Escape 처리에서 commit 가드를 먼저
 * 올리지 않으면 변경한 텍스트가 저장되고 undo 기록도 생긴다. 실제 form-01.hwp의 Edit
 * 필드와 브라우저 keydown/blur 순서로 그 계약을 검증한다.
 */
import { assert, loadHwpFile, runTest, setTestCase } from './helpers.mjs';

const wait = (page, ms) => page.evaluate((delay) => new Promise((resolve) => setTimeout(resolve, delay)), ms);

runTest('양식 편집 필드 Escape 취소 (#2375)', async ({ page }) => {
  setTestCase('Edit 값 변경 후 Escape → blur에도 저장·undo 기록 없음');

  const { pageCount } = await loadHwpFile(page, 'form-01.hwp');
  assert(pageCount >= 1, `form-01.hwp 로드 성공 (${pageCount}페이지)`);

  const before = await page.evaluate(() => {
    const tree = JSON.parse(window.__wasm.doc.getPageRenderTree(0));
    const stack = [tree];
    while (stack.length) {
      const node = stack.pop();
      if (node?.type === 'Form' && node.bbox) {
        const hit = window.__wasm.getFormObjectAt(
          0,
          node.bbox.x + node.bbox.w / 2,
          node.bbox.y + node.bbox.h / 2,
        );
        if (hit?.formType === 'Edit') {
          return {
            hit,
            text: hit.text ?? '',
            undoDepth: window.__inputHandler?.history?.undoStack?.length ?? -1,
          };
        }
      }
      stack.push(...(node?.children || []));
    }
    return null;
  });

  assert(before?.hit, 'form-01.hwp의 Edit 양식 필드를 찾음');
  if (!before?.hit) throw new Error('Edit 양식 필드가 없어 Escape 취소를 검증할 수 없음');

  const opened = await page.evaluate((hit) => {
    const handler = window.__inputHandler;
    if (!handler?.handleFormObjectClick) return false;
    handler.handleFormObjectClick(hit, 0, 1);
    return true;
  }, before.hit);
  assert(opened, '실제 InputHandler 경로로 Edit overlay를 열음');
  await page.waitForSelector('input.form-edit-input', { timeout: 3_000 });

  await page.keyboard.type('ESCAPE_MUST_CANCEL');
  await page.keyboard.press('Escape');
  await wait(page, 150); // remove() 뒤 blur가 같은 취소 가드를 통과하는지 확인.

  const after = await page.evaluate((hit) => {
    const current = window.__wasm.getFormObjectAt(
      0,
      hit.bbox.x + hit.bbox.w / 2,
      hit.bbox.y + hit.bbox.h / 2,
    );
    return {
      overlayPresent: Boolean(document.querySelector('input.form-edit-input')),
      text: current?.text ?? '',
      undoDepth: window.__inputHandler?.history?.undoStack?.length ?? -1,
    };
  }, before.hit);

  assert(!after.overlayPresent, 'Escape 후 Edit overlay가 닫힘');
  assert(after.text === before.text, `Escape 뒤 값 불변 (${JSON.stringify(after.text)})`);
  assert(after.undoDepth === before.undoDepth, `Escape 뒤 undo 기록 없음 (${after.undoDepth})`);
});
