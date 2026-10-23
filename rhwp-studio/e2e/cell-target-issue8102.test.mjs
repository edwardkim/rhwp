import assert from 'node:assert/strict';
import { mkdirSync, writeFileSync } from 'node:fs';
import { runTest, createNewDocument, screenshot, setTestCase } from './helpers.mjs';
const output = '../output/pr-review/issue-8102';
mkdirSync(output, { recursive: true });
const settle = page => page.evaluate(() => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r))));
const chord = async (page, key, shift = false) => {
  await page.keyboard.down('Control');
  if (shift) await page.keyboard.down('Shift');
  try { await page.keyboard.press(key); } finally {
    if (shift) await page.keyboard.up('Shift');
    await page.keyboard.up('Control');
  }
};
const selectDelete = async page => {
  await chord(page, 'a'); await page.keyboard.press('Backspace'); await settle(page);
};
const readCell = (page, pos, cellIndex = 0) => page.evaluate(({ pos, cellIndex }) => {
  const w = window.__wasm;
  const n = w.getCellParagraphCount(0, pos.parentParaIndex, pos.controlIndex, cellIndex);
  return Array.from({ length: n }, (_, p) => w.getTextInCell(0, pos.parentParaIndex, pos.controlIndex, cellIndex, p, 0,
    w.getCellParagraphLength(0, pos.parentParaIndex, pos.controlIndex, cellIndex, p)));
}, { pos, cellIndex });
await runTest('#8102 셀 진입·선택 삭제·undo/redo', async ({ page }) => {
  const rows = [], errors = [];
  page.on('pageerror', e => errors.push(String(e)));
  for (const mode of ['created', 'clicked', 'keyboard', 'undo-entry', 'legacy-flat', 'multi-paragraph']) {
    setTestCase(mode);
    try {
      await createNewDocument(page);
      await page.evaluate(async () => {
        const ih = window.__inputHandler;
        ih.cursor.clearSelection(); ih.cursor.exitCellSelectionMode();
        ih.cursor.moveTo({ sectionIndex: 0, paragraphIndex: 0, charOffset: 0 });
        ih.activateWithCaretPosition(); ih.focus();
        const { tableCommands } = await import('/src/command/commands/table.ts');
        tableCommands.find(c => c.id === 'table:create').execute({ getInputHandler: () => ih });
      });
      const grid = await page.waitForSelector('[data-row="1"][data-col="1"]', { visible: true });
      await grid.hover(); await grid.click(); await settle(page);
      const pos = await page.evaluate(() => window.__inputHandler.cursor.getPosition());
      assert.equal(pos.cellIndex, 0);
      await page.evaluate(pos => window.__wasm.insertTextInCell(0, pos.parentParaIndex, pos.controlIndex, 1, 0, 0, 'NEIGHBOR'), pos);
      const expected = mode === 'multi-paragraph' ? ['alpha', 'beta', 'gamma'] : ['alpha'];
      await page.evaluate(text => {
        const data = new DataTransfer(); data.setData('text/plain', text);
        window.__inputHandler.textarea.dispatchEvent(new ClipboardEvent('paste', {
          clipboardData: data, bubbles: true, cancelable: true,
        }));
      }, expected.join('\n'));
      await settle(page);
      assert.deepEqual(await readCell(page, pos), expected);
      if (mode === 'clicked') {
        const point = await page.evaluate(pos => {
          const w = window.__wasm, cv = window.__canvasView;
          const r = w.getCursorRectInCell(0, pos.parentParaIndex, pos.controlIndex, 0, 0, 2);
          const vs = cv.virtualScroll, zoom = cv.viewportManager.getZoom();
          const content = document.getElementById('scroll-content'), box = content.getBoundingClientRect();
          return { x: box.left + vs.getPageLeftResolved(0, content.clientWidth) + r.x * zoom,
            y: box.top + vs.getPageOffset(0) + (r.y + r.height / 2) * zoom };
        }, pos);
        await page.mouse.click(point.x, point.y); await settle(page);
        const p = await page.evaluate(() => window.__inputHandler.cursor.getPosition());
        assert.equal(p.cellIndex, 0); assert.equal(p.cellPath?.length, 1);
      } else if (mode === 'keyboard') {
        await page.keyboard.press('Tab'); await page.keyboard.down('Shift'); await page.keyboard.press('Tab'); await page.keyboard.up('Shift');
        assert.equal(await page.evaluate(() => window.__inputHandler.cursor.getPosition().cellIndex), 0);
      } else if (mode === 'undo-entry') {
        await selectDelete(page);
        assert.deepEqual(await readCell(page, pos), ['']);
        await chord(page, 'z'); await settle(page);
        assert.deepEqual(await readCell(page, pos), expected);
      } else if (mode === 'legacy-flat') {
        await page.evaluate(() => {
          const ih = window.__inputHandler, p = ih.cursor.getPosition();
          delete p.cellPath; ih.cursor.moveTo(p); ih.updateCaret(); ih.focus();
        });
      }
      const before = await page.evaluate(() => window.__inputHandler.cursor.getPosition());
      await selectDelete(page);
      const after = await readCell(page, pos);
      assert.deepEqual(after, [''], `${mode}: 선택 삭제`);
      assert.deepEqual(await readCell(page, pos, 1), ['NEIGHBOR']);
      await chord(page, 'z'); await settle(page);
      assert.deepEqual(await readCell(page, pos), expected);
      assert.equal(await page.evaluate(() => window.__inputHandler.cursor.getPosition().cellIndex), 0, 'undo 셀 진입');
      await chord(page, 'z', true); await settle(page);
      assert.deepEqual(await readCell(page, pos), ['']);
      assert.deepEqual(await readCell(page, pos, 1), ['NEIGHBOR']);
      await chord(page, 'z'); await settle(page); await selectDelete(page);
      assert.deepEqual(await readCell(page, pos), [''], 'undo 재진입 후 다시 삭제');
      if (mode === 'created') await screenshot(page, 'issue8102-created-deleted');
      rows.push({ mode, before, after, undo: expected, redo: [''], neighbor: 'NEIGHBOR', result: 'PASS' });
    } catch (e) { rows.push({ mode, result: 'FAIL', error: String(e) }); }
  }
  setTestCase('중첩 셀·invalid target');
  const contracts = await page.evaluate(async () => {
    const { WasmBridge } = await import('/src/core/wasm-bridge.ts');
    const { CursorState } = await import('/src/engine/cursor.ts');
    const { DeleteSelectionCommand } = await import('/src/engine/command.ts');
    const { CommandHistory } = await import('/src/engine/history.ts');
    const bridge = new WasmBridge(); await bridge.initialize();
    try {
      const cell = content => `<TABLE RowCount="1" ColCount="2"><SHAPEOBJECT><SIZE Width="16000" Height="4000"/></SHAPEOBJECT><ROW><CELL ColAddr="0" RowAddr="0" Width="8000" Height="4000"><PARALIST>${content}</PARALIST></CELL><CELL ColAddr="1" RowAddr="0" Width="8000" Height="4000"><PARALIST><P><TEXT><CHAR>NEIGHBOR</CHAR></TEXT></P></PARALIST></CELL></ROW></TABLE>`;
      const paras = ['alpha', 'beta', 'gamma'].map(t => `<P><TEXT><CHAR>${t}</CHAR></TEXT></P>`).join('');
      const hml = `<HWPML Version="2.91"><HEAD/><BODY><SECTION><P><TEXT>${cell(`<P><TEXT><CHAR>OUTER</CHAR>${cell(paras)}</TEXT></P>`)}</TEXT></P></SECTION></BODY><TAIL/></HWPML>`;
      bridge.loadDocument(new TextEncoder().encode(hml), 'cell-target.hml');
      const target = { sectionIndex: 0, paragraphIndex: 0, charOffset: 0, parentParaIndex: 0,
        controlIndex: 0, cellIndex: 0, cellParaIndex: 0,
        cellPath: [{ controlIndex: 0, cellIndex: 0, cellParaIndex: 0 }, { controlIndex: 0, cellIndex: 0, cellParaIndex: 0 }] };
      const read = path => {
        const n = bridge.getCellParagraphCountByPath(0, 0, JSON.stringify(path));
        return Array.from({ length: n }, (_, p) => bridge.getTextInCellByPath(0, 0,
          JSON.stringify(path.map((e, i) => i === path.length - 1 ? { ...e, cellParaIndex: p } : e)), 0, 100));
      };
      const cursor = new CursorState(bridge); cursor.moveTo(target);
      if (!cursor.selectAllInCell()) throw new Error('중첩 선택 실패');
      const selected = cursor.getSelectionOrdered(), history = new CommandHistory();
      const neighborPath = target.cellPath.map((e, i) => i === 1 ? { ...e, cellIndex: 1 } : e);
      const original = read(target.cellPath), outer = read(target.cellPath.slice(0, 1)), neighbor = read(neighborPath);
      history.execute(new DeleteSelectionCommand(selected.start, selected.end), bridge);
      const deleted = read(target.cellPath), neighborAfter = read(neighborPath), outerAfter = read(target.cellPath.slice(0, 1));
      history.undo(bridge); const undone = read(target.cellPath);
      history.redo(bridge); const redone = read(target.cellPath);
      const before = Array.from(bridge.exportHwp()); let rejected = 0;
      for (const p of [
        { ...target, cellPath: undefined, cellIndex: undefined },
        { ...target, cellPath: [{ controlIndex: 0, cellIndex: -1, cellParaIndex: 0 }] },
        { ...target, cellPath: [{ controlIndex: 0, cellIndex: 999, cellParaIndex: 0 }] },
      ]) {
        try { new DeleteSelectionCommand(p, { ...p, charOffset: 1 }).execute(bridge); } catch { rejected++; }
      }
      const unchanged = JSON.stringify(before) === JSON.stringify(Array.from(bridge.exportHwp()));
      bridge.createNewDocument();
      const box = bridge.createShapeControl({ sectionIdx: 0, paraIdx: 0, charOffset: 0,
        shapeType: 'textbox', width: 12000, height: 4000 });
      if (!box.ok) throw new Error('글상자 생성 실패');
      bridge.insertTextInCell(0, box.paraIdx, box.controlIdx, 0, 0, 0, 'BOX');
      const boxPos = { sectionIndex: 0, paragraphIndex: 0, charOffset: 0,
        parentParaIndex: box.paraIdx, controlIndex: box.controlIdx,
        cellIndex: 0, cellParaIndex: 0, isTextBox: true };
      const boxCursor = new CursorState(bridge); boxCursor.moveTo(boxPos);
      if (!boxCursor.selectAllInCell()) throw new Error('글상자 선택 실패');
      const boxSelection = boxCursor.getSelectionOrdered(), boxHistory = new CommandHistory();
      const boxRead = () => bridge.getTextInCell(0, box.paraIdx, box.controlIdx, 0, 0, 0, 100);
      boxHistory.execute(new DeleteSelectionCommand(boxSelection.start, boxSelection.end), bridge);
      const boxDeleted = boxRead(); boxHistory.undo(bridge); const boxUndone = boxRead();
      boxHistory.redo(bridge); const boxRedone = boxRead();
      return { original, deleted, undone, redone, outer, outerAfter, neighbor, neighborAfter,
        rejected, unchanged, textbox: { deleted: boxDeleted, undone: boxUndone, redone: boxRedone } };
    } finally { bridge.releaseDocument(); }
  });
  console.log(JSON.stringify({ rows, contracts, errors }, null, 2));
  writeFileSync(`${output}/browser-result.json`, JSON.stringify({ rows, contracts, errors }, null, 2) + '\n');
  for (const row of rows) assert.equal(row.result, 'PASS', `${row.mode}: ${row.error}`);
  assert.deepEqual(contracts.original, ['alpha', 'beta', 'gamma']);
  assert.deepEqual(contracts.deleted, ['']); assert.deepEqual(contracts.redone, ['']);
  assert.deepEqual(contracts.undone, contracts.original);
  assert.deepEqual(contracts.neighborAfter, contracts.neighbor); assert.deepEqual(contracts.outerAfter, contracts.outer);
  assert.deepEqual(contracts.textbox, { deleted: '', undone: 'BOX', redone: '' });
  assert.equal(contracts.rejected, 3); assert.equal(contracts.unchanged, true); assert.deepEqual(errors, []);
});
