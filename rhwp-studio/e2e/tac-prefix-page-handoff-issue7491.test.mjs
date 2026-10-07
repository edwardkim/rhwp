/** Fresh WASM live cell-edit geometry; this does not claim saved-cell height fidelity. */
import assert from 'node:assert/strict';
import {runTest, loadHwpFile, screenshot} from './helpers.mjs';

await runTest('TAC prefix stays before the grown object row page handoff', async ({page}) => {
  await loadHwpFile(page, 'issue6882/synth_cell_enter_table_growth.hwp');
  const result = await page.evaluate(async () => {
    const w = window.__wasm;
    const d = w.doc;
    const last = d.getCellParagraphCount(0, 1, 0, 31) - 1;
    const len = d.getCellParagraphLength(0, 1, 0, 31, last);
    for (let i = 0; i < 8; i++) {
      d.splitParagraphInCell(0, 1, 0, 31, last + i, i === 0 ? len : 0, undefined);
    }
    await window.__canvasView.loadDocument();
    return {pages: d.pageCount(), table: w.getTableBBox(0, 1, 0)};
  });
  assert.equal(result.pages, 3, 'Hancom 2020 independently produces 3 pages');
  assert.equal(result.table.pageIndex, 1, 'grown table belongs to p2');
  assert.ok(Math.abs(result.table.y - 69.844) < 0.5, `Hancom table top: ${JSON.stringify(result)}`);
  assert.ok(result.table.y + result.table.height <= 1052.64 + 0.5, 'painted table fits within the physical body');
  console.log('PASS: fresh WASM live table page/top/body boundary', JSON.stringify(result));
  await screenshot(page, 'issue-7491-tac-prefix-page-handoff');
});
