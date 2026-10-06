import assert from 'node:assert/strict';
import { runTest, createNewDocument } from './helpers.mjs';

runTest('Issue 6638: plain text keeps cell paragraph boundaries', async ({ page }) => {
  await createNewDocument(page);
  const target = await page.evaluate(async () => {
    const wasm = window.__wasm;
    const result = wasm.createTable(0, 0, 0, 1, 1);
    if (!result.ok) throw new Error('createTable failed');
    await window.__canvasView.loadDocument();
    const input = window.__inputHandler;
    input.cursor.moveTo({
      sectionIndex: 0, paragraphIndex: 0, charOffset: 0,
      parentParaIndex: result.paraIdx, controlIndex: result.controlIdx,
      cellIndex: 0, cellParaIndex: 0,
      cellPath: [{ controlIndex: result.controlIdx, cellIndex: 0, cellParaIndex: 0 }],
    });
    input.updateCaret();
    input.focus();
    return { para: result.paraIdx, control: result.controlIdx };
  });
  await page.evaluate(() => {
    const data = new DataTransfer();
    data.setData('text/plain', '첫째\n\n셋째\n');
    window.__inputHandler.textarea.dispatchEvent(new ClipboardEvent('paste', {
      clipboardData: data, bubbles: true, cancelable: true,
    }));
  });
  const readState = () => page.evaluate(({ para, control }) => {
    const wasm = window.__wasm;
    const count = wasm.getCellParagraphCount(0, para, control, 0);
    return {
      count,
      texts: Array.from({ length: count }, (_, index) =>
        wasm.getTextInCell(0, para, control, 0, index, 0,
          wasm.getCellParagraphLength(0, para, control, 0, index))),
      cursor: window.__inputHandler.cursor.getPosition(),
    };
  }, target);
  const state = await readState();
  console.log(JSON.stringify(state));
  assert.deepEqual(state.texts, ['첫째', '', '셋째', '']);
  assert.equal(state.cursor.cellParaIndex, 3);
  assert.equal(state.cursor.charOffset, 0);
  const depth = await page.evaluate(() => window.__inputHandler.history.undoStack.length);
  assert.ok(depth > 0 && depth <= 5);
  for (let index = 0; index < depth; index++) {
    await page.evaluate(() => window.__inputHandler.performUndo());
  }
  assert.deepEqual((await readState()).texts, ['']);
  for (let index = 0; index < depth; index++) {
    await page.evaluate(() => window.__inputHandler.performRedo());
  }
  assert.deepEqual((await readState()).texts, state.texts);
});
