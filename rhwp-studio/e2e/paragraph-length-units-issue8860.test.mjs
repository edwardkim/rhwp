import assert from 'node:assert/strict';
import { createNewDocument, moveCursorTo, runTest } from './helpers.mjs';

async function chord(page, modifier, key) {
  await page.keyboard.down(modifier);
  try { await page.keyboard.press(key); } finally { await page.keyboard.up(modifier); }
}

async function paragraphDialog(page, { kind, pt, left, right, alignment } = {}) {
  await page.evaluate(() => window.__inputHandler.focus());
  await chord(page, 'Alt', 'KeyT');
  const indentRadio = 'input[name="ps-first-line"][value="indent"]';
  await page.waitForSelector(indentRadio, { visible: true });
  if (kind) await page.click('input[name="ps-first-line"][value="' + kind + '"]');
  const shown = await page.evaluate(({ indentRadio, pt, left, right, alignment }) => {
    const indent = document.querySelector(indentRadio).parentElement.querySelector('input[type="number"]');
    const panel = indent.closest('.ps-two-col');
    const [leftMargin, rightMargin] = panel.querySelectorAll('input[type="number"]');
    const initial = Number(indent.value);
    for (const [input, value] of [[indent, pt], [leftMargin, left], [rightMargin, right]]) {
      if (value === undefined) continue;
      input.value = String(value);
      input.dispatchEvent(new Event('input', { bubbles: true }));
    }
    const dialog = indent.closest('.dialog') ?? indent.closest('[role="dialog"]') ?? document;
    if (alignment) dialog.querySelector('.sb-al-' + alignment).click();
    const apply = [...dialog.querySelectorAll('button.dialog-btn-primary')].find(b => b.offsetParent !== null);
    if (!apply) throw new Error('문단 모양 설정 버튼을 찾을 수 없습니다.');
    apply.click();
    return initial;
  }, { indentRadio, pt, left, right, alignment });
  await page.waitForSelector(indentRadio, { hidden: true });
  return shown;
}

// 형식별로 서로 다른 문서 인스턴스에 두 번 열어서 화면 표시값과 별도로 저장 단위·본문을 읽는다.
async function readSaved(page, cell) {
  return page.evaluate(cell => {
    const live = window.__wasm.doc;
    function read(doc) {
      const list = cell
        ? JSON.parse(doc.getCursorModel()).lists.find(l => l.sectionIndex === 0 &&
          l.hostListId === 0 && l.hostPara === cell.para && l.controlIndex === cell.control && l.cellIndex === 1).listId
        : 0;
      const { Indentation, LeftMargin, RightMargin } = JSON.parse(doc.getParaShapeSet(list, 0));
      const text = cell ? doc.getTextInCell(0, cell.para, cell.control, 1, 0, 0, 100) : doc.getTextRange(0, 0, 0, 100);
      return [Indentation, LeftMargin, RightMargin, text];
    }
    const rows = [read(live)];
    for (const method of ['exportHwp', 'exportHwpx']) {
      const first = new live.constructor(live[method]());
      try {
        rows.push(read(first));
        const second = new live.constructor(first[method]());
        try { rows.push(read(second)); } finally { second.free(); }
      } finally { first.free(); }
    }
    return rows;
  }, cell);
}

async function expectSaved(page, expected, cell = null) {
  for (const row of await readSaved(page, cell)) assert.deepEqual(row, expected);
}

await runTest('문단 길이의 절대 입력과 CHAR 보존 (#8860)', async ({ page }) => {
  await createNewDocument(page);
  await page.evaluate(async () => {
    const d = window.__wasm.doc;
    d.insertText(0, 0, 0, 'ABC');
    d.applyParaFormat(0, 0, JSON.stringify({ indent: -203, marginLeft: 201, marginRight: 202 }));
    await window.__canvasView.loadDocument();
  });
  await moveCursorTo(page, 0, 0, 0);
  assert.equal(await paragraphDialog(page), 5.1);
  await expectSaved(page, [-203, 201, 202, 'ABC']);
  await paragraphDialog(page, { alignment: 'left' });
  await expectSaved(page, [-203, 201, 202, 'ABC']);
  await paragraphDialog(page, { pt: 5.1 });
  await expectSaved(page, [-1020, 201, 202, 'ABC']);
  await page.evaluate(() => window.__inputHandler.focus());
  await chord(page, 'Control', 'KeyZ');
  await expectSaved(page, [-203, 201, 202, 'ABC']);
  await paragraphDialog(page, { kind: 'hanging', pt: 1.005, left: 1.015, right: 1.005 });
  await expectSaved(page, [-202, 204, 202, 'ABC']);
  await page.evaluate(() => window.__inputHandler.focus());
  await chord(page, 'Control', 'KeyZ');
  await expectSaved(page, [-203, 201, 202, 'ABC']);
  await chord(page, 'Control', 'KeyY');
  await expectSaved(page, [-202, 204, 202, 'ABC']);
  await paragraphDialog(page, { kind: 'normal' });
  await expectSaved(page, [0, 204, 202, 'ABC']);

  await moveCursorTo(page, 0, 0, 2);
  const cursorDistance = await page.evaluate(() => {
    const w = window.__wasm;
    window.__inputHandler.focus();
    return w.getCursorRect(0, 0, 2).x - w.getCursorRect(0, 0, 0).x;
  });
  assert.ok(cursorDistance > 0);
  await chord(page, 'Shift', 'Tab');
  await expectSaved(page, [-2 * Math.round(cursorDistance * 75), 204, 202, 'ABC']);

  await createNewDocument(page);
  await page.evaluate(async () => {
    const d = window.__wasm.doc;
    d.insertText(0, 0, 0, 'source');
    d.splitParagraph(0, 0, 6);
    d.insertText(0, 1, 0, 'target');
    d.applyParaFormat(0, 0, JSON.stringify({ indent: -203, marginLeft: 201, marginRight: 202 }));
    await window.__canvasView.loadDocument();
    window.__inputHandler.focus();
  });
  await chord(page, 'Control', 'Home');
  await chord(page, 'Alt', 'KeyC');
  await chord(page, 'Control', 'End');
  await chord(page, 'Shift', 'Home');
  await chord(page, 'Alt', 'KeyC');
  const copied = await page.evaluate(() => JSON.parse(window.__wasm.doc.getParaShapeSet(0, 1)));
  assert.deepEqual([copied.Indentation, copied.LeftMargin, copied.RightMargin], [-203, 201, 202]);

  await createNewDocument(page);
  const cell = await page.evaluate(async () => {
    const w = window.__wasm;
    const { paraIdx, controlIdx } = w.createTable(0, 0, 0, 1, 2);
    w.doc.insertTextInCell(0, paraIdx, controlIdx, 1, 0, 0, 'cell');
    await window.__canvasView.loadDocument();
    const handler = window.__inputHandler;
    handler.cursor.moveTo({
      sectionIndex: 0, paragraphIndex: 0, charOffset: 0,
      parentParaIndex: paraIdx, controlIndex: controlIdx, cellIndex: 1, cellParaIndex: 0,
      cellPath: [{ controlIndex: controlIdx, cellIndex: 1, cellParaIndex: 0 }],
    });
    handler.updateCaret();
    handler.focus();
    return { para: paraIdx, control: controlIdx };
  });
  await paragraphDialog(page, { kind: 'indent', pt: 1.015 });
  await expectSaved(page, [204, 0, 0, 'cell'], cell);
  await page.evaluate(() => window.__inputHandler.focus());
  await chord(page, 'Shift', 'Tab');
  assert.equal(await page.evaluate(() => window.__inputHandler.cursor.getPosition().cellIndex), 0);
  await expectSaved(page, [204, 0, 0, 'cell'], cell);
});
