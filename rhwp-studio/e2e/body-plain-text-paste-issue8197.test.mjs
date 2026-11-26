import assert from 'node:assert/strict';
import { createNewDocument, loadApp, runTest } from './helpers.mjs';

const input = Array.from({ length: 400 }, (_, index) =>
  'P' + String(index).padStart(3, '0') + ' selection sample');

async function paste(page, text) {
  return page.evaluate(value => window.__pasteIssue8197(value), text);
}

await runTest('Issue 8197: body plain-text paste keeps edits and batches layout', async ({ page }) => {
  await loadApp(page, '?renderer=canvas2d');
  await page.evaluate(() => {
    document.querySelector('.skin-onboarding-card')?.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }));
    [...document.querySelectorAll('button.dialog-btn-primary')].find(button => button.offsetParent !== null)?.click();
    window.__pasteIssue8197 = text => {
      const input = window.__inputHandler;
      const data = new DataTransfer();
      data.setData('text/plain', text);
      input.textarea.dispatchEvent(new ClipboardEvent('paste', {
        clipboardData: data, bubbles: true, cancelable: true,
      }));
    };
    window.__layoutIssue8197 = () => {
      const wasm = window.__wasm;
      return Array.from({ length: wasm.pageCount }, (_, index) =>
        JSON.parse(wasm.doc.getPageTextLayout(index)));
    };
    window.__bodyIssue8197 = () => {
      const wasm = window.__wasm;
      return Array.from({ length: wasm.getParagraphCount(0) }, (_, index) =>
        wasm.getTextRange(0, index, 0, wasm.getParagraphLength(0, index)));
    };
  });

  await createNewDocument(page);
  const counts = await page.evaluate(lines => {
    const wasm = window.__wasm;
    const bus = window.__eventBus;
    const names = ['beginBatch', 'endBatch'];
    const originals = names.map(name => wasm.doc[name]);
    const calls = { beginBatch: 0, endBatch: 0, cursor: 0, mutations: 0 };
    const cursor = wasm.getCursorRect;
    names.forEach((name, index) => {
      wasm.doc[name] = function (...args) {
        calls[name]++;
        return originals[index].apply(this, args);
      };
    });
    wasm.getCursorRect = function (...args) {
      calls.cursor++;
      return cursor.apply(this, args);
    };
    const stop = bus.on('document-mutated', () => { calls.mutations++; });
    try {
      window.__pasteIssue8197(lines.join('\n'));
      return { ...calls, text: window.__bodyIssue8197(), caret: window.__inputHandler.cursor.getPosition() };
    } finally {
      names.forEach((name, index) => { wasm.doc[name] = originals[index]; });
      wasm.getCursorRect = cursor;
      stop();
    }
  }, input);
  assert.deepEqual(counts.text, input);
  assert.deepEqual(counts.caret, { sectionIndex: 0, paragraphIndex: 399, charOffset: 21 });
  assert.deepEqual([counts.beginBatch, counts.endBatch, counts.cursor, counts.mutations], [1, 1, 1, 1]);
  await page.waitForFunction(() => document.querySelector('#scroll-container canvas.document-page-canvas'));
  const batchLayout = await page.evaluate(() => window.__layoutIssue8197());

  // Save and reopen twice with Studio's actual WASM exporters and open parser.
  const reopened = await page.evaluate(() => {
    const wasm = window.__wasm;
    const originals = [['hwp', wasm.exportHwp()], ['hwpx', wasm.exportHwpx()]];
    return originals.flatMap(([format, bytes]) => {
      wasm.loadDocument(bytes, 'paste.' + format);
      const first = window.__bodyIssue8197();
      const resaved = format === 'hwp' ? wasm.exportHwp() : wasm.exportHwpx();
      wasm.loadDocument(resaved, 'paste-resaved.' + format);
      return [first, window.__bodyIssue8197()];
    });
  });
  assert.deepEqual(reopened, [input, input, input, input]);
  await createNewDocument(page);
  const legacyLayout = await page.evaluate(lines => {
    const ih = window.__inputHandler;
    ih.tryPasteBodyParagraphs = () => false;
    try {
      window.__pasteIssue8197(lines.join('\n'));
      return window.__layoutIssue8197();
    } finally { delete ih.tryPasteBodyParagraphs; }
  }, input);
  assert.deepEqual(batchLayout, legacyLayout);
  console.log('400문단 쪽별 텍스트 배치와 HWP/HWPX 반복 저장 통과');

  await createNewDocument(page);
  const historyCase = await page.evaluate(() => {
    const ih = window.__inputHandler;
    const wasm = window.__wasm;
    window.__pasteIssue8197('prefix😀suffix');
    wasm.applyCharFormat(0, 0, 7, 13, JSON.stringify({ bold: true }));
    ih.cursor.moveTo({ sectionIndex: 0, paragraphIndex: 0, charOffset: 7 });
    window.__pasteIssue8197('첫째\r\n\r\n마지막\r\n');
    const pasted = window.__bodyIssue8197();
    const caret = ih.cursor.getPosition();
    const boldSuffix = wasm.getCharPropertiesAt(0, 3, 1).bold;
    ih.performUndo();
    const once = window.__bodyIssue8197();
    for (let index = 0; index < 4; index++) ih.performUndo();
    const undone = window.__bodyIssue8197();
    for (let index = 0; index < 5; index++) ih.performRedo();
    return { pasted, caret, boldSuffix, once, undone, redone: window.__bodyIssue8197() };
  });
  const mixed = ['prefix😀첫째', '', '마지막', 'suffix'];
  assert.deepEqual(historyCase.pasted, mixed);
  assert.deepEqual(historyCase.caret, { sectionIndex: 0, paragraphIndex: 3, charOffset: 0 });
  assert.equal(historyCase.boldSuffix, true);
  assert.deepEqual(historyCase.once, ['prefix😀첫째', '', '마지막suffix']);
  assert.deepEqual(historyCase.undone, ['prefix😀suffix']);
  assert.deepEqual(historyCase.redone, mixed);
  console.log('서식, 캐럿과 Undo/Redo 통과');

  await createNewDocument(page);
  await paste(page, 'First paragraph\nSecond paragraph');
  await page.evaluate(async () => {
    window.__inputHandler.performUndo();
    await document.fonts.ready;
  });
  await page.waitForFunction(() => !window.__canvasView.rendererSelectionPending);
  const recovery = await page.evaluate(async () => {
    const ih = window.__inputHandler;
    const wasm = window.__wasm;
    const before = window.__bodyIssue8197();
    const beforeCaret = ih.cursor.getPosition();
    const split = wasm.splitParagraph;
    const reported = new Promise(resolve => window.addEventListener('rhwp-command-error', event => {
      resolve(event.detail);
    }, { once: true }));
    wasm.splitParagraph = function (...args) {
      split.apply(this, args);
      throw new Error('Issue 8197 injected after a real split');
    };
    try { window.__pasteIssue8197('will\nroll back'); }
    finally { wasm.splitParagraph = split; }
    const report = await Promise.race([reported, new Promise(resolve => setTimeout(() => resolve(null), 5000))]);
    const after = window.__bodyIssue8197();
    const afterCaret = ih.cursor.getPosition();
    const canRedo = ih.canRedo();
    if (report?.code === 'COMMAND_ROLLED_BACK') ih.performRedo();
    return { report, before, beforeCaret, after, afterCaret, canRedo, redone: window.__bodyIssue8197() };
  });
  assert.equal(recovery.report?.code, 'COMMAND_ROLLED_BACK', JSON.stringify(recovery.report));
  assert.deepEqual(recovery.after, recovery.before);
  assert.deepEqual(recovery.afterCaret, recovery.beforeCaret);
  assert.equal(recovery.canRedo, true);
  assert.deepEqual(recovery.redone, ['First paragraph', 'Second paragraph']);

  await createNewDocument(page);
  await paste(page, 'AAAABBBBCCCC');
  const replaced = await page.evaluate(() => {
    const ih = window.__inputHandler;
    ih.cursor.moveTo({ sectionIndex: 0, paragraphIndex: 0, charOffset: 4 });
    ih.cursor.setAnchor();
    ih.cursor.moveTo({ sectionIndex: 0, paragraphIndex: 0, charOffset: 8 });
    window.__pasteIssue8197('X\nY');
    return window.__bodyIssue8197();
  });
  assert.deepEqual(replaced, ['AAAAX', 'YCCCC']);

  await createNewDocument(page);
  const columns = await page.evaluate(() => {
    const wasm = window.__wasm;
    wasm.setColumnDef(0, 2, 0, 1, 1200);
    const batch = wasm.runInBatch;
    let batches = 0;
    wasm.runInBatch = function (...args) { batches++; return batch.apply(this, args); };
    try {
      window.__pasteIssue8197('first\nsecond');
      return { batches, text: window.__bodyIssue8197() };
    } finally { wasm.runInBatch = batch; }
  });
  assert.deepEqual(columns, { batches: 0, text: ['first', 'second'] });
  console.log('400문단 배치·캐럿, HWP/HWPX 재저장, 서식·선택·Undo/Redo 및 배치 중 실패 복구 확인');
}, { skipLoadApp: true });
