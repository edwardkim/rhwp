// 합성 표를 실제 WASM과 Studio 명령으로 편집한다. 화면 픽셀/PDF 일치 검사는 아니다.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createServer } from 'vite';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const studio = resolve(process.env.RHWP_TEST_STUDIO ?? join(dirname(fileURLToPath(import.meta.url)), '../..'));
const src = join(studio, 'src');
const core = resolve(process.env.RHWP_TEST_CORE ?? join(studio, '../pkg'));
const binding = join(core, 'rhwp.js');
const { default: init, HwpDocument } = await import(pathToFileURL(binding));
await init({ module_or_path: readFileSync(join(core, 'rhwp_bg.wasm')) });
const vite = await createServer({
  configFile: false, root: studio, appType: 'custom', logLevel: 'error',
  resolve: { alias: { '@': src, '@wasm/rhwp.js': binding, '@wasm': core } },
  server: { middlewareMode: true },
});
try {
const { WasmBridge } = await vite.ssrLoadModule('/src/core/wasm-bridge.ts');
const { CommandHistory } = await vite.ssrLoadModule('/src/engine/history.ts');
const { InputHandler } = await vite.ssrLoadModule('/src/engine/input-handler.ts');
const { ApplyParaFormatCommand } = await vite.ssrLoadModule('/src/engine/command.ts');

for (const depth of [2, 3]) {
  const doc = HwpDocument.createEmpty();
  doc.createBlankDocument();
  let html = '<table><tr><td><p>inner-a</p><p>inner-a2</p></td><td><p>inner-b</p></td><td><p>untouched</p></td></tr></table>';
  for (let level = 1; level < depth; level++) html = `<table><tr><td>outer-${level}${html}</td></tr></table>`;
  doc.pasteHtml(0, 0, 0, html);
  const matches = JSON.parse(doc.searchAllText('inner-', false, true));
  assert.equal(matches.length, 3);
  const anchor = matches[0];
  const path = anchor.cellPath;
  assert.equal(path.length, depth);
  const untouched = JSON.parse(doc.searchAllText('untouched', false, true))[0];
  const outerPath = path.slice(0, 1).map(entry => ({ ...entry, cellParaIndex: 0 }));
  const wasm = Object.create(WasmBridge.prototype);
  wasm.doc = doc;
  const history = new CommandHistory();
  const position = (cellPath) => ({
    sectionIndex: anchor.sec, paragraphIndex: anchor.para, parentParaIndex: anchor.para,
    controlIndex: cellPath[0].controlIndex, cellIndex: cellPath[0].cellIndex,
    cellParaIndex: cellPath[0].cellParaIndex, cellPath, charOffset: 0,
  });
  let current = position(path), block = true, selection = null;
  const excluded = new Set();
  const handler = Object.create(InputHandler.prototype);
  handler.wasm = wasm;
  handler.cursor = {
    getPosition: () => current,
    isInHeaderFooter: () => false,
    isInFootnote: () => false,
    isInCellSelectionMode: () => block,
    getCellTableContext: () => ({ sec: anchor.sec, ppi: anchor.para, ci: path[0].controlIndex, cellPath: path }),
    getSelectedCellRange: () => ({ startRow: 0, endRow: 0, startCol: 0, endCol: 1 }),
    getExcludedCells: () => excluded,
    getSelectionOrdered: () => selection,
  };
  let operations = 0;
  handler.executeOperation = descriptor => {
    assert.equal(descriptor.kind, 'command');
    operations++;
    history.execute(descriptor.command, wasm);
  };
  const props = (cellPath, document = doc) => JSON.parse(document.getCellParaPropertiesAtByPath(anchor.sec, anchor.para, JSON.stringify(cellPath)));
  const state = () => matches.map(match => props(match.cellPath));
  const outsideBefore = props(outerPath);
  const untouchedBefore = props(untouched.cellPath);
  const before = state();
  for (const alignment of ['center', 'right', 'left', 'justify']) {
    handler.applyParaAlign(alignment);
    assert.ok(state().every(value => value.alignment === alignment));
    assert.equal(handler.getParaProperties().alignment, alignment, '툴바는 최내곽 문단 모양을 조회한다');
  }
  assert.equal(operations, 4);
  for (let i = 0; i < 4; i++) history.undo(wasm);
  assert.deepEqual(state(), before, '모든 모양 ID와 문단 속성을 복원한다');
  for (let i = 0; i < 4; i++) history.redo(wasm);
  assert.ok(state().every(value => value.alignment === 'justify'));
  assert.deepEqual(props(outerPath), outsideBefore, '바깥 셀을 변경하지 않는다');
  assert.deepEqual(props(untouched.cellPath), untouchedBefore, '선택하지 않은 셀을 변경하지 않는다');

  excluded.add('0,1');
  handler.applyParaAlign('center');
  assert.deepEqual(state().map(value => value.alignment), ['center', 'center', 'justify']);
  excluded.add('0,0');
  const count = operations;
  handler.applyParaAlign('right');
  assert.equal(operations, count, '빈 블록은 앵커 셀로 fallback하지 않는다');
  excluded.clear();

  block = false;
  selection = { start: position(matches[0].cellPath), end: position(matches[1].cellPath) };
  handler.applyParaAlign('right');
  assert.deepEqual(state().map(value => value.alignment), ['right', 'right', 'justify']);
  history.undo(wasm);
  assert.deepEqual(state().map(value => value.alignment), ['center', 'center', 'justify']);
  history.redo(wasm);
  selection = { start: position(matches[0].cellPath), end: position(matches[2].cellPath) };
  assert.deepEqual(handler.getParaFormatTargetsAtCursor(), [], '다른 셀의 텍스트 선택은 섞지 않는다');
  selection = null;
  current = position(matches[2].cellPath);
  handler.applyParaAlign('center');
  assert.deepEqual(state().map(value => value.alignment), ['right', 'right', 'center']);

  const beforeSpacing = state();
  handler.setLineSpacing(200);
  assert.equal(props(matches[2].cellPath).lineSpacing, 200);
  assert.deepEqual(state().slice(0, 2), beforeSpacing.slice(0, 2));
  const afterSpacing = state();
  history.undo(wasm);
  assert.deepEqual(state(), beforeSpacing, '줄간격도 원래 모양 ID로 복원한다');
  history.redo(wasm);
  assert.deepEqual(state(), afterSpacing);
  history.undo(wasm);

  const invalid = path.map(entry => ({ ...entry }));
  invalid[invalid.length - 1].cellIndex = 9999;
  const beforeInvalid = state();
  assert.throws(() => history.execute(new ApplyParaFormatCommand([
    { kind: 'cellPath', sec: anchor.sec, parentPara: anchor.para, path },
    { kind: 'cellPath', sec: anchor.sec, parentPara: anchor.para, path: invalid },
  ], { alignment: 'left' }, current), wasm));
  assert.deepEqual(state(), beforeInvalid, '대상 사전 조회 실패는 일부 셀만 변경하지 않는다');
  assert.throws(() => doc.applyParaFormatInCellByPath(0, anchor.para, '[]', '{"alignment":"center"}'));
  assert.throws(() => doc.setCellParaShapeIdByPath(0, anchor.para, JSON.stringify(path), 65535));

  for (const exportName of ['exportHwp', 'exportHwpx']) {
    const reopened = new HwpDocument(doc[exportName]());
    try {
      assert.deepEqual(matches.map(match => props(match.cellPath, reopened).alignment), ['right', 'right', 'center']);
      assert.equal(props(untouched.cellPath, reopened).alignment, untouchedBefore.alignment);
    } finally { reopened.free(); }
  }
  history.clear(wasm);
  doc.free();
}
console.log('NESTED_PARAGRAPH_FORMAT_OK: depth 2/3, block/range/caret, excluded cells, undo/redo, HWP/HWPX');

} finally { await vite.close(); }
