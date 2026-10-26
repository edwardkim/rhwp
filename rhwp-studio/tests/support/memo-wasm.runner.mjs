// 빌드된 실제 WASM + Studio command/bridge/history 로 메모 기준 글자 삭제의 undo 를 확인한다.
// pkg/ 가 있으면 tests/memo-delete-undo.test.ts 가 실행한다. 직접 실행:
// node --experimental-transform-types --no-warnings tests/support/memo-wasm.runner.mjs
import { registerHooks } from 'node:module';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { pathToFileURL, fileURLToPath } from 'node:url';
import assert from 'node:assert/strict';

const root = join(dirname(fileURLToPath(import.meta.url)), '../..');
const src = join(root, 'src');
const repo = join(root, '..');
registerHooks({ resolve(specifier, context, next) {
  // JSON 메트릭은 TypeScript 확장자를 붙이지 않고 Node의 import attributes로 읽는다.
  if (specifier.endsWith('.json')) return next(specifier, context);
  if (specifier === '@wasm/rhwp.js') return { url: pathToFileURL(join(repo, 'pkg/rhwp.js')).href, shortCircuit: true };
  if (specifier.startsWith('@/')) return { url: pathToFileURL(join(src, specifier.slice(2) + '.ts')).href, shortCircuit: true };
  if (/^\.\.?\//.test(specifier) && !/\.[cm]?[tj]s$/.test(specifier)) {
    return { url: new URL(specifier + '.ts', context.parentURL).href, shortCircuit: true };
  }
  return next(specifier, context);
} });
const { initSync, HwpDocument } = await import(pathToFileURL(join(repo, 'pkg/rhwp.js')));
initSync({ module: readFileSync(join(repo, 'pkg/rhwp_bg.wasm')) });
const { WasmBridge } = await import(pathToFileURL(join(src, 'core/wasm-bridge.ts')));
const { DeleteTextCommand, DeleteSelectionCommand } = await import(pathToFileURL(join(src, 'engine/command.ts')));
const { CommandHistory } = await import(pathToFileURL(join(src, 'engine/history.ts')));
const results = [];

function session(doc) {
  const wasm = new WasmBridge();
  // 노드 환경에서 DOM 폰트 초기화 없이 실제 WASM 문서만 주입한다.
  wasm.doc = doc;
  let captures = 0;
  const capture = wasm.captureDeleteRange.bind(wasm);
  wasm.captureDeleteRange = (...args) => { captures += 1; return capture(...args); };
  const history = new CommandHistory();
  return {
    wasm, history,
    run(command) { history.execute(command, wasm); },
    undo() { history.undo(wasm); },
    redo() { history.redo(wasm); },
    captures: () => captures,
  };
}
const memos = doc => JSON.parse(doc.getMemoList());
/** 저장 후 다시 열어도 같은 메모가 남는지 본다. id 는 문서마다 새로 매길 수 있어 비교하지 않는다. */
function assertSaved(doc, expected) {
  const strip = list => list.map(({ id, page, ...rest }) => rest);
  for (const method of ['exportHwp', 'exportHwpx']) {
    const reopened = new HwpDocument(doc[method]());
    assert.deepEqual(strip(memos(reopened)), strip(expected), method);
    reopened.free();
  }
}
function blankWith(text) {
  const doc = HwpDocument.createEmpty();
  doc.createBlankDocument();
  doc.insertText(0, 0, 0, text);
  return doc;
}
const memoInput = JSON.stringify({ author: '검토자', text: '메모 본문 😀' });
const position = { sectionIndex: 0, paragraphIndex: 0, charOffset: 0 };

// 1) Delete 한 번으로 기준 글자 전체를 지운다.
{
  const doc = blankWith('앞 메모 뒤');
  assert.equal(JSON.parse(doc.insertMemo(0, 0, 2, 4, memoInput)).ok, true);
  const before = memos(doc);
  const s = session(doc);
  s.run(new DeleteTextCommand({ ...position, charOffset: 2 }, 2, 'forward'));
  assert.deepEqual(memos(doc), [], '기준 글자 전체 삭제는 메모도 지운다');
  assert.equal(doc.getTextRange(0, 0, 0, 4), '앞  뒤');
  for (let cycle = 0; cycle < 3; cycle++) {
    s.undo();
    assert.deepEqual(memos(doc), before, 'undo 는 메모와 본문을 되살린다');
    assert.equal(doc.getTextRange(0, 0, 0, 6), '앞 메모 뒤');
    assertSaved(doc, before);
    s.redo();
    assert.deepEqual(memos(doc), []);
  }
  s.undo();
  assert.equal(s.captures(), 4, '실행과 redo 마다 문단 조각을 한 번씩 잡는다');
  results.push('Delete 기준 전체 삭제: 3회 undo·redo, 매 undo HWP/HWPX 왕복');
}

// 2) Backspace 로 한 글자씩 지운다. 마지막 글자에서 메모가 사라진다.
{
  const doc = blankWith('앞 메모 뒤');
  JSON.parse(doc.insertMemo(0, 0, 2, 4, memoInput));
  const before = memos(doc);
  const s = session(doc);
  s.run(new DeleteTextCommand({ ...position, charOffset: 3 }, 1, 'backward', undefined, 10));
  const shortened = memos(doc);
  assert.equal(shortened.length, 1);
  assert.deepEqual([shortened[0].startCharIdx, shortened[0].endCharIdx], [2, 3], '일부 삭제는 범위만 줄인다');
  s.run(new DeleteTextCommand({ ...position, charOffset: 2 }, 1, 'backward', undefined, 20));
  assert.deepEqual(memos(doc), []);
  s.undo();
  assert.deepEqual(memos(doc), shortened);
  s.undo();
  assert.deepEqual(memos(doc), before);
  s.redo(); s.redo(); s.undo(); s.undo();
  assert.deepEqual(memos(doc), before);
  assertSaved(doc, before);
  results.push('Backspace 한 글자씩: 단계별 undo·redo 와 HWP/HWPX 왕복');
}

// 3) 기준 범위에 닿지 않는 삭제는 조각 없이 글자 재삽입으로 되돌린다. 기준 바로 앞·뒤를
//    지우면 재삽입한 글자가 메모 범위에 붙으므로 조각으로 되돌린다.
{
  const doc = blankWith('앞쪽 메모 뒤쪽');
  JSON.parse(doc.insertMemo(0, 0, 3, 5, memoInput));
  const before = memos(doc);
  const s = session(doc);
  s.run(new DeleteTextCommand({ ...position, charOffset: 0 }, 2, 'forward'));
  s.run(new DeleteTextCommand({ ...position, charOffset: 5 }, 2, 'forward'));
  assert.equal(s.captures(), 0, '기준에서 떨어진 삭제는 일반 경로');
  s.undo(); s.undo();
  assert.deepEqual(memos(doc), before);
  assert.equal(doc.getTextRange(0, 0, 0, 9), '앞쪽 메모 뒤쪽');
  for (const [offset, count] of [[2, 1], [5, 1], [0, 3], [5, 4]]) {
    s.run(new DeleteTextCommand({ ...position, charOffset: offset }, count, 'forward'));
    assert.deepEqual(memos(doc).map(m => m.text), [before[0].text], `${offset}+${count} 삭제는 메모를 남긴다`);
    s.undo();
    assert.deepEqual(memos(doc), before, `${offset}+${count} 삭제 undo 는 메모 범위를 그대로 되돌린다`);
    assert.equal(doc.getTextRange(0, 0, 0, 9), '앞쪽 메모 뒤쪽');
  }
  assert.equal(s.captures(), 4, '기준 바로 앞·뒤 삭제는 문단 조각을 잡는다');
  const plain = session(blankWith('메모 없음'));
  plain.run(new DeleteTextCommand(position, 2, 'forward'));
  plain.undo();
  assert.equal(plain.captures(), 0);
  assert.equal(plain.wasm.doc.getTextRange(0, 0, 0, 5), '메모 없음');
  results.push('기준에서 떨어진 삭제·메모 없는 문서는 기존 경로, 기준 앞·뒤 삭제는 범위 그대로 복원');
}

// 3-1) 길이 0 메모는 글자 삭제로 지워지지 않지만, 그 자리를 지운 뒤 재삽입하면 위치가 앞으로 밀린다.
{
  const doc = blankWith('앞쪽뒤쪽');
  assert.equal(JSON.parse(doc.insertMemo(0, 0, 2, 2, memoInput)).ok, true);
  const before = memos(doc);
  const s = session(doc);
  for (const [offset, count] of [[0, 2], [1, 2], [2, 2]]) {
    s.run(new DeleteTextCommand({ ...position, charOffset: offset }, count, 'forward'));
    assert.equal(memos(doc).length, 1, `${offset}+${count} 삭제는 길이 0 메모를 남긴다`);
    s.undo();
    assert.deepEqual(memos(doc), before, `${offset}+${count} 삭제 undo 는 길이 0 메모 위치를 되돌린다`);
    assert.equal(doc.getTextRange(0, 0, 0, 4), '앞쪽뒤쪽');
  }
  results.push('길이 0 메모 주변 삭제 undo 의 위치 복원');
}

// 4) 선택 삭제(잘라내기도 같은 명령)는 문단 조각으로 이미 메모를 되살린다.
{
  const doc = blankWith('앞 메모 뒤');
  JSON.parse(doc.insertMemo(0, 0, 2, 4, memoInput));
  const before = memos(doc);
  const s = session(doc);
  s.run(new DeleteSelectionCommand({ ...position, charOffset: 1 }, { ...position, charOffset: 5 }));
  assert.deepEqual(memos(doc), []);
  s.undo();
  assert.deepEqual(memos(doc), before);
  s.redo(); s.undo();
  assert.deepEqual(memos(doc), before);
  results.push('선택 삭제 undo·redo');
}

// 5) 표 칸 안의 메모.
{
  const doc = HwpDocument.createEmpty();
  doc.createBlankDocument();
  const table = JSON.parse(doc.createTable(0, 0, 0, 1, 2));
  assert.equal(table.ok, true);
  const ppi = table.paraIdx;
  const ci = table.controlIdx;
  doc.insertTextInCell(0, ppi, ci, 1, 0, 0, '칸 메모');
  const path = [{ controlIndex: ci, cellIndex: 1, cellParaIndex: 0 }];
  assert.equal(JSON.parse(doc.insertMemoByPath(0, ppi, JSON.stringify(path), 2, 4, memoInput)).ok, true);
  const before = memos(doc);
  assert.equal(before.length, 1);
  const cellPos = { sectionIndex: 0, paragraphIndex: 0, charOffset: 2, parentParaIndex: ppi,
    controlIndex: ci, cellIndex: 1, cellParaIndex: 0, cellPath: path };
  const s = session(doc);
  s.run(new DeleteTextCommand(cellPos, 2, 'forward'));
  assert.deepEqual(memos(doc), []);
  s.undo();
  assert.deepEqual(memos(doc), before);
  s.redo(); s.undo();
  assert.deepEqual(memos(doc), before);
  assertSaved(doc, before);
  results.push('표 칸 메모 삭제 undo·redo 와 HWP/HWPX 왕복');
}

console.log(results.map(line => `ok - ${line}`).join('\n'));
console.log('MEMO_WASM_OK');
