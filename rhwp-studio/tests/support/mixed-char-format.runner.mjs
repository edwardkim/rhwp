// #6788: 실제 command/history/bridge + fresh Node WASM 행위 회귀.
import assert from 'node:assert/strict';
import { registerHooks } from 'node:module';
import { dirname, join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const studio = join(dirname(fileURLToPath(import.meta.url)), '../..');
const src = join(studio, 'src');
const binding = join(studio, '../pkg-node/rhwp.js');
registerHooks({
  resolve(specifier, context, next) {
    if (specifier === '@wasm/rhwp.js') return { url: pathToFileURL(binding).href, shortCircuit: true };
    if (specifier.startsWith('@/')) return { url: pathToFileURL(join(src, specifier.slice(2) + '.ts')).href, shortCircuit: true };
    if (/^\.{1,2}\//.test(specifier) && !/\.[cm]?[tj]s$/.test(specifier)) {
      return { url: pathToFileURL(join(dirname(fileURLToPath(context.parentURL)), specifier + '.ts')).href, shortCircuit: true };
    }
    return next(specifier, context);
  },
});
const { HwpDocument } = await import(pathToFileURL(binding));
const { WasmBridge } = await import(pathToFileURL(join(src, 'core/wasm-bridge.ts')));
const { ApplyCharFormatCommand, SnapshotCommand } = await import(pathToFileURL(join(src, 'engine/command.ts')));
const { CommandHistory } = await import(pathToFileURL(join(src, 'engine/history.ts')));
const { InputHandler } = await import(pathToFileURL(join(src, 'engine/input-handler.ts')));
const { parseCharShapeRuns } = await import(pathToFileURL(join(src, 'core/char-shape-runs.ts')));
const pos = (p, off) => ({ sectionIndex: 0, paragraphIndex: p, charOffset: off });
const text = '가😀나다라마바';
const purple = '#a020c0';
const yellow = { shadeColor: '#ffff00' };
function bridge(doc) {
  const wasm = Object.create(WasmBridge.prototype);
  wasm.doc = doc;
  return wasm;
}
function blank() {
  const doc = HwpDocument.createEmpty();
  doc.createBlankDocument();
  return doc;
}
function bodyState(wasm, count = 1) {
  return Array.from({ length: count }, (_, p) => Array.from({ length: wasm.getParagraphLength(0, p) }, (_, o) => wasm.getCharPropertiesAt(0, p, o)));
}
function checkHighlight(before, after, from, to) {
  for (let p = 0; p < before.length; p++) {
    for (let o = 0; o < before[p].length; o++) {
      const selected = (p > from.paragraphIndex || o >= from.charOffset) && (p < to.paragraphIndex || o < to.charOffset);
      const expected = { ...before[p][o] };
      const actual = { ...after[p][o] };
      delete expected.charShapeId;
      delete actual.charShapeId;
      if (selected) expected.shadeColor = '#ffff00';
      assert.deepEqual(actual, expected, `문단 ${p}, 문자 ${o}: 미지정 속성/선택 밖 보존`);
    }
  }
}
let scenarios = 0;
for (const [count, from, to] of [[1, pos(0, 0), pos(0, 7)], [1, pos(0, 2), pos(0, 4)], [1, pos(0, 3), pos(0, 6)], [2, pos(0, 1), pos(1, 6)]]) {
  const doc = blank();
  for (let p = 0; p < count; p++) {
    if (p) doc.splitParagraph(0, p - 1, 7);
    doc.insertText(0, p, 0, text);
    doc.applyCharFormat(0, p, 2, 4, JSON.stringify({ textColor: purple, bold: true, fontSize: 1800 }));
  }
  const wasm = bridge(doc);
  const history = new CommandHistory();
  let restoreCalls = 0;
  const restore = wasm.setCharShapeRuns.bind(wasm);
  wasm.setCharShapeRuns = (...args) => { restoreCalls++; return restore(...args); };
  const before = bodyState(wasm, count);
  history.execute(new ApplyCharFormatCommand(from, to, yellow), wasm);
  const after = bodyState(wasm, count);
  checkHighlight(before, after, from, to);
  for (let cycle = 0; cycle < 3; cycle++) {
    assert.deepEqual(history.undo(wasm), from);
    assert.deepEqual(bodyState(wasm, count), before, 'Undo 전체 모양/ID');
    assert.deepEqual(history.redo(wasm), from);
    assert.deepEqual(bodyState(wasm, count), after, 'Redo 전체 모양/ID');
  }
  assert.equal(restoreCalls, count * 6, '구간 수와 무관하게 문단당 복원 호출 한 번');
  for (const exportName of ['exportHwp', 'exportHwpx']) {
    const reopened = new HwpDocument(doc[exportName]());
    const roundtrip = bodyState(bridge(reopened), count);
    assert.deepEqual(roundtrip.map(p => p.map(v => [v.textColor, v.shadeColor])), after.map(p => p.map(v => [v.textColor, v.shadeColor])));
    reopened.free();
  }
  history.clear(wasm);
  doc.free();
  scenarios++;
}

function findCell(doc, nested) {
  for (let p = 0; p < doc.getParagraphCount(0); p++) {
    for (let c = 0; c < 8; c++) {
      const outer = { controlIndex: c, cellIndex: 0, cellParaIndex: 0 };
      for (let cp = 0; cp < (nested ? 4 : 1); cp++) {
        for (let inner = 0; inner < (nested ? 4 : 1); inner++) {
          const path = nested ? [{ ...outer, cellParaIndex: cp }, { controlIndex: inner, cellIndex: 0, cellParaIndex: 0 }] : [outer];
          try { if (doc.getCellParagraphLengthByPath(0, p, JSON.stringify(path)) >= 6) return { p, path }; } catch { /* 후보 탐색 */ }
        }
      }
    }
  }
  throw new Error('유효 셀 fixture를 찾지 못했습니다');
}
for (const nested of [false, true]) {
  const doc = blank();
  const contents = nested ? '<table><tr><td>바깥문자<table><tr><td>가나다라마바</td></tr></table></td></tr></table>' : '<table><tr><td>가나다라마바</td></tr></table>';
  doc.pasteHtml(0, 0, 0, contents);
  const { p, path } = findCell(doc, nested);
  const json = JSON.stringify(path);
  const wasm = bridge(doc);
  doc.applyCharFormatInCellByPath(0, p, json, 2, 4, JSON.stringify({ textColor: purple }));
  const state = () => Array.from({ length: doc.getCellParagraphLengthByPath(0, p, json) }, (_, off) => wasm.getCellCharPropertiesAtByPath(0, p, json, off));
  const before = state();
  const start = { ...pos(p, 1), parentParaIndex: p, controlIndex: path[0].controlIndex, cellIndex: 0, cellParaIndex: 0, cellPath: path };
  const end = { ...start, charOffset: 5 };
  const history = new CommandHistory();
  history.execute(new ApplyCharFormatCommand(start, end, yellow), wasm);
  const after = state();
  checkHighlight([before], [after], pos(0, 1), pos(0, 5));
  for (let i = 0; i < 3; i++) {
    history.undo(wasm); assert.deepEqual(state(), before);
    history.redo(wasm); assert.deepEqual(state(), after);
  }
  history.undo(wasm);
  // 실제 F5 경로의 operation을 실제 SnapshotCommand/History로 실행한다.
  const handler = Object.create(InputHandler.prototype);
  handler.wasm = wasm;
  handler.cursor = { getPosition: () => start };
  handler.eventBus = { emit() {} };
  handler.executeOperation = (desc) => {
    assert.equal(desc.kind, 'snapshot');
    history.execute(new SnapshotCommand(desc.operationType, start, start, desc.operation), wasm);
  };
  handler.applyCharFormatToCellBlock({ sec: 0, ppi: p, ci: path[0].controlIndex, cellIndices: [0], cellPath: path }, yellow);
  const blockAfter = state();
  checkHighlight([before], [blockAfter], pos(0, 0), pos(0, before.length));
  history.undo(wasm); assert.deepEqual(state(), before);
  history.redo(wasm); assert.deepEqual(state(), blockAfter);
  history.clear(wasm);
  doc.free();
  scenarios += 2;
}

// 실제 머리말/꼬리말 선택 operation의 snapshot 복원 대조.
for (const isHeader of [true, false]) {
  const doc = blank();
  doc.createHeaderFooter(0, isHeader, 0);
  doc.insertTextInHeaderFooter(0, isHeader, 0, 0, 0, '가나다라마바');
  doc.applyCharFormatInHeaderFooter(0, isHeader, 0, 0, 2, 0, 4, JSON.stringify({ textColor: purple }));
  const wasm = bridge(doc);
  const history = new CommandHistory();
  const state = () => Array.from({ length: 6 }, (_, off) => wasm.getCharPropertiesInHeaderFooter(0, isHeader, 0, 0, off));
  const before = state();
  const start = { sectionIdx: 0, isHeader, applyTo: 0, paraIdx: 0, charOffset: 0 };
  const handler = Object.create(InputHandler.prototype);
  handler.cursor = {
    hfParaIdx: 0, hfCharOffset: 6, getPosition: () => pos(0, 0),
    getHeaderFooterSelectionOrdered: () => ({ start, end: { ...start, charOffset: 6 }, previewPage: 0 }),
  };
  handler.executeOperation = (desc) => {
    assert.equal(desc.kind, 'snapshot');
    history.execute(new SnapshotCommand(desc.operationType, pos(0, 0), pos(0, 0), desc.operation), wasm);
  };
  assert.equal(handler.applyCharFormatInHeaderFooterSelection(yellow), true);
  const after = state();
  checkHighlight([before], [after], pos(0, 0), pos(0, 6));
  for (let i = 0; i < 3; i++) {
    history.undo(wasm); assert.deepEqual(state(), before);
    history.redo(wasm); assert.deepEqual(state(), after);
  }
  history.clear(wasm);
  doc.free();
  scenarios++;
}

// 최초 실행 도중 두 번째 문단에서 실패하면 첫 문단도 되돌리고 history에 기록하지 않는다.
{
  const doc = blank();
  doc.insertText(0, 0, 0, text);
  doc.splitParagraph(0, 0, 7);
  doc.insertText(0, 1, 0, text);
  const wasm = bridge(doc);
  const before = bodyState(wasm, 2);
  const apply = wasm.applyCharFormat.bind(wasm);
  wasm.applyCharFormat = (sec, p, ...args) => {
    if (p === 1) throw new Error('injected failure');
    return apply(sec, p, ...args);
  };
  const history = new CommandHistory();
  assert.throws(() => history.execute(new ApplyCharFormatCommand(pos(0, 0), pos(1, 7), yellow), wasm),
    error => error.cause?.message === 'injected failure');
  assert.deepEqual(bodyState(wasm, 2), before);
  assert.equal(history.undo(wasm), null);
  doc.free();
  scenarios++;
}

// 빈 선택은 history를 소비하지 않는다.
{
  const doc = blank();
  const wasm = bridge(doc);
  const history = new CommandHistory();
  history.execute(new ApplyCharFormatCommand(pos(0, 0), pos(0, 0), yellow), wasm);
  assert.equal(history.undo(wasm), null);
  doc.free();
  scenarios++;
}

// 누락 binding/비정상 구간 응답은 mutation 전에 거절한다.
{
  let mutations = 0;
  const fake = { applyCharFormat() { mutations++; }, getCharShapeRuns() { return '[]'; } };
  const history = new CommandHistory();
  assert.throws(() => history.execute(new ApplyCharFormatCommand(pos(0, 0), pos(0, 6), yellow), bridge(fake)), /최신 WASM/);
  assert.equal(mutations, 0);
  for (const json of ['null', '[]', '[{"startOffset":1,"endOffset":6,"charShapeId":0}]', '[{"startOffset":0,"endOffset":6,"charShapeId":-1}]']) {
    assert.throws(() => parseCharShapeRuns(json, 0, 6));
  }
  assert.deepEqual(parseCharShapeRuns('[]', 0, 0), []);
  scenarios++;
}
console.log(`MIXED_CHAR_FORMAT_OK scenarios=${scenarios}`);
