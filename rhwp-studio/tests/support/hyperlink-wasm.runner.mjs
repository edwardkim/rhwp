// #6963: 빌드된 실제 WASM + Studio command/bridge/history를 실행한다.
// node --experimental-transform-types --no-warnings tests/support/hyperlink-wasm.runner.mjs
// pkg/가 필요한 focused 검증이다. DOM 대신 모달 입출력만 대체하며 화면 검증은 별도 수행한다.
import { registerHooks } from 'node:module';
import { readFileSync, mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { pathToFileURL, fileURLToPath } from 'node:url';
import assert from 'node:assert/strict';

const root = join(dirname(fileURLToPath(import.meta.url)), '../..');
const src = join(root, 'src');
const repo = join(root, '..');
const dialogModule = 'data:text/javascript,' + encodeURIComponent(`
export let current;
export let deletion;
export function confirmHyperlinkDelete(remove, cancel) { deletion = { remove, cancel }; }
export function confirmHyperlinkEdit(edit) { edit(); }
export class HyperlinkDialog {
  constructor(initial, apply) { this.initial = initial; this.apply = apply; current = this; }
  show() {}
}
`);
const toastModule = 'data:text/javascript,' + encodeURIComponent(`export function showToast(o) { throw new Error(o.message); }`);
registerHooks({ resolve(specifier, context, next) {
  if (specifier === '@/ui/hyperlink-dialog' || specifier === '@/ui/hyperlink-delete-dialog') return { url: dialogModule, shortCircuit: true };
  if (specifier === '@/ui/toast') return { url: toastModule, shortCircuit: true };
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
const { SnapshotCommand, DeleteTextCommand, InsertTextCommand } = await import(pathToFileURL(join(src, 'engine/command.ts')));
const { CommandHistory } = await import(pathToFileURL(join(src, 'engine/history.ts')));
const { hyperlinkCommand } = await import(pathToFileURL(join(src, 'command/commands/hyperlink.ts')));
const { tryConfirmDeleteHyperlink } = await import(pathToFileURL(join(src, 'engine/input-handler-hyperlink-delete.ts')));
const dialogs = await import(dialogModule);
const results = [];

function session(doc, position) {
  const wasm = new WasmBridge();
  // 노드 환경에서 DOM 폰트 초기화 없이 실제 WASM 문서만 주입한다.
  wasm.doc = doc;
  let pos = position;
  let selection = null;
  let editable = true;
  const history = new CommandHistory();
  const ih = {
    canEditHyperlink: () => true,
    hasSelection: () => selection !== null,
    getSelection: () => selection,
    getCursorPosition: () => pos,
    focus() {},
    executeOperation(desc) {
      assert.equal(desc.kind, 'snapshot');
      const command = new SnapshotCommand(desc.operationType, pos, pos, desc.operation, desc.selectionBefore);
      pos = history.execute(command, wasm);
      selection = null;
    },
  };
  const deleteHost = { wasm, canEditHyperlink: () => editable, isFormMode: () => false,
    isActive: () => true, focusTextarea() {}, executeOperation: desc => ih.executeOperation(desc) };
  const services = { wasm, getInputHandler: () => ih, getContext: () => ({ isEditable: editable, isFormMode: false }) };
  return {
    wasm, history,
    deleteAt(offset) { return tryConfirmDeleteHyperlink(deleteHost, { ...position, charOffset: offset }); },
    backspaceAt(offset) { return tryConfirmDeleteHyperlink(deleteHost, { ...position, charOffset: offset }, 'backward'); },
    open() { hyperlinkCommand.execute(services); return dialogs.current; },
    select(start, end) { selection = { start: { ...position, charOffset: start }, end: { ...position, charOffset: end } }; },
    cursor(offset) { pos = { ...position, charOffset: offset }; selection = null; },
    undo() { pos = history.undo(wasm); },
    redo() { pos = history.redo(wasm); },
    readOnly() { editable = false; },
  };
}
const body = { section: 0, para: 0, cellPath: [] };
const position = { sectionIndex: 0, paragraphIndex: 0, charOffset: 0 };
const uri = 'https://example.com/한글?q=1#부분';
const context = (s, target = body) => s.wasm.getHyperlinkContext(target);
const doc = HwpDocument.createEmpty();
doc.createBlankDocument(); // Studio의 새 문서와 같은 내장 템플릿 경로
const s = session(doc, position);
const initial = context(s);
const dialog = s.open();
assert.throws(() => dialog.apply({ kind: 'save', text: '실패하면 복원', uri: 'javascript:alert(1)' }));
assert.deepEqual(context(s), initial);
assert.equal(s.history.canUndo(), false);
dialog.apply({ kind: 'save', text: '한컴 링크 😀', uri });
assert.equal(context(s).links[0].uri, uri);
assert.equal(context(s).links[0].text, '한컴 링크 😀');
assert.equal(s.wasm.getCharPropertiesAt(0, 0, 0).textColor.toLowerCase(), '#0000ff');
assert.equal(s.wasm.getCharPropertiesAt(0, 0, 0).underline, true);
s.undo(); assert.deepEqual(context(s), initial);
s.redo(); assert.equal(context(s).links.length, 1);
results.push('무선택 삽입·실패 원자 복원·실제 snapshot undo/redo');

s.cursor(2);
const same = s.open();
same.apply({ kind: 'save', text: same.initial.text, uri });
s.undo(); assert.deepEqual(context(s), initial); // 같은 주소 적용은 undo 엔트리가 아니다.
s.redo();
s.cursor(2);
s.open().apply({ kind: 'save', text: context(s).links[0].text, uri: 'https://example.com/updated#수정' });
s.undo(); assert.equal(context(s).links[0].uri, uri);
s.redo(); assert.equal(context(s).links[0].uri, 'https://example.com/updated#수정');
results.push('동일 주소 무기록·주소 수정 undo/redo');

for (const method of ['exportHwp', 'exportHwpx']) {
  const reopened = new HwpDocument(doc[method]());
  assert.deepEqual(JSON.parse(reopened.getHyperlinkContext(JSON.stringify(body))), context(s), method);
  const props = JSON.parse(reopened.getCharPropertiesAt(0, 0, 0));
  assert.equal(props.textColor.toLowerCase(), '#0000ff', method);
  assert.equal(props.underline, true, method);
  reopened.free();
}
results.push('Studio bridge 편집 후 HWP/HWPX 저장 왕복');
s.cursor(2);
const beforeRemove = context(s);
s.open().apply({ kind: 'remove' });
assert.equal(context(s).text, beforeRemove.text);
assert.equal(context(s).links.length, 0);
s.undo(); assert.deepEqual(context(s), beforeRemove);
s.redo(); assert.equal(context(s).links.length, 0);
results.push('연결 해제의 문자열 보존 및 undo/redo');
s.select(0, 2);
s.open().apply({ kind: 'save', uri, text: Array.from(context(s).text).slice(0, 2).join('') });
assert.equal(context(s).links[0].text, Array.from(beforeRemove.text).slice(0, 2).join(''));
results.push('선택 문자열 삽입');
const beforeRename = context(s);
s.cursor(1);
s.open().apply({ kind: 'save', uri, text: '수정한 표시 😀 문자열' });
assert.equal(context(s).links[0].text, '수정한 표시 😀 문자열');
assert.equal(context(s).links[0].fieldId, beforeRename.links[0].fieldId);
for (const method of ['exportHwp', 'exportHwpx']) {
  const reopened = new HwpDocument(doc[method]());
  assert.deepEqual(JSON.parse(reopened.getHyperlinkContext(JSON.stringify(body))), context(s), method);
  reopened.free();
}
s.undo(); assert.deepEqual(context(s), beforeRename);
s.redo(); assert.equal(context(s).links[0].text, '수정한 표시 😀 문자열');
results.push('표시 문자열 실제 수정·필드 ID 보존·양식 저장·undo/redo');
s.cursor(0);
const stale = s.open();
doc.insertText(0, 0, 0, '외부');
assert.throws(() => stale.apply({ kind: 'remove' }), /문서 내용이 바뀌었습니다/);
const generationDialog = s.open();
s.wasm._documentGeneration += 1;
assert.throws(() => generationDialog.apply({ kind: 'remove' }), /편집 상태가 바뀌었습니다/);
results.push('외부 내용 변경·문서 교체 세대 차단');
const readOnly = s.open();
s.readOnly();
assert.throws(() => readOnly.apply({ kind: 'save', uri, text: 'test' }), /편집 상태가 바뀌었습니다/);
results.push('모달이 열린 뒤 읽기 전용 전환 차단');

// 한컴 원본의 2단계 중첩 셀: 경로는 단계 1~3에서 확인한 실제 주소다.
const lh = new HwpDocument(readFileSync(join(repo, 'samples/hwpx_sample2.hwpx')));
const target = { section: 0, para: 74, cellPath: [[0, 0, 9], [0, 0, 0]] };
const nestedPos = { ...position, paragraphIndex: 0, parentParaIndex: 74, cellPath: [
  { controlIndex: 0, cellIndex: 0, cellParaIndex: 9 },
  { controlIndex: 0, cellIndex: 0, cellParaIndex: 0 },
] };
const nested = session(lh, nestedPos);
const original = context(nested, target);
nested.cursor(original.links[0].start);
nested.open().apply({ kind: 'save', uri, text: original.links[0].text });
assert.equal(context(nested, target).links[0].uri, uri);
nested.undo(); assert.deepEqual(context(nested, target), original);
nested.cursor(original.links[0].start);
nested.open().apply({ kind: 'save', uri, text: '중첩 셀 새 이름😀' });
assert.equal(context(nested, target).links[0].text, '중첩 셀 새 이름😀');
for (const method of ['exportHwp', 'exportHwpx']) {
  const reopened = new HwpDocument(lh[method]());
  assert.deepEqual(JSON.parse(reopened.getHyperlinkContext(JSON.stringify(target))), context(nested, target), method);
  reopened.free();
}
nested.undo(); assert.deepEqual(context(nested, target), original);
nested.cursor(0);
nested.open().apply({ kind: 'save', text: '새 링크😀 ', uri });
assert.equal(context(nested, target).links.length, original.links.length + 1);
assert.equal(context(nested, target).text, '새 링크😀 ' + original.text);
nested.undo(); assert.deepEqual(context(nested, target), original);
results.push('한컴 중첩 셀 기존 링크 수정·무선택 신규 삽입·undo');

const boxDoc = HwpDocument.createEmpty();
boxDoc.createBlankDocument();
const shape = JSON.parse(boxDoc.createShapeControl(JSON.stringify({
  sectionIdx: 0, paraIdx: 0, charOffset: 0, width: 10000, height: 5000,
  shapeType: 'textbox', horzOffset: 0, vertOffset: 0,
})));
assert.equal(shape.ok, true);
const boxPos = { ...position, parentParaIndex: shape.paraIdx, controlIndex: shape.controlIdx,
  cellIndex: 0, cellParaIndex: 0, isTextBox: true };
const boxTarget = { section: 0, para: shape.paraIdx, cellPath: [[shape.controlIdx, 0, 0]] };
const box = session(boxDoc, boxPos);
const boxBefore = context(box, boxTarget);
box.open().apply({ kind: 'save', uri, text: '글상자 링크 ' });
assert.equal(context(box, boxTarget).links[0].text, '글상자 링크 ');
box.undo(); assert.deepEqual(context(box, boxTarget), boxBefore);
box.redo();
for (const method of ['exportHwp', 'exportHwpx']) {
  const reopened = new HwpDocument(boxDoc[method]());
  assert.deepEqual(JSON.parse(reopened.getHyperlinkContext(JSON.stringify(boxTarget))), context(box, boxTarget), method);
  reopened.free();
}
results.push('실제 글상자 신규 링크·undo/redo·HWP/HWPX 왕복');
const safe = context(box, boxTarget);
for (const invalid of ['{}', '{"section":-1,"para":0,"cellPath":[]}', '{"section":0,"para":0,"cellPath":[[0,0]]}']) {
  assert.throws(() => boxDoc.getHyperlinkContext(invalid));
}
assert.throws(() => boxDoc.insertHyperlinkEx(JSON.stringify({ target: boxTarget, start: -1, end: 2, uri })));
assert.throws(() => boxDoc.updateHyperlinkEx(JSON.stringify({ target: boxTarget, fieldId: 0.5, uri })));
assert.deepEqual(context(box, boxTarget), safe);
results.push('WASM JSON 경계의 누락·음수·잘못된 경로·비정수 ID 거부');

// PR #6984: undo는 보통 타이핑과 달리 삭제 전 링크 범위·서식을 복구해야 한다.
function assertSavedContext(document, current, target) {
  for (const method of ['exportHwp', 'exportHwpx']) {
    const reopened = new HwpDocument(document[method]());
    assert.deepEqual(JSON.parse(reopened.getHyperlinkContext(JSON.stringify(target))), current, method);
    reopened.free();
  }
}
function deleteAndRestore(document, currentSession, target, pos, offset, count) {
  const before = context(currentSession, target);
  const cmd = new DeleteTextCommand({ ...pos, charOffset: offset }, count, 'backward');
  currentSession.history.execute(cmd, currentSession.wasm);
  const deleted = context(currentSession, target);
  assert.notDeepEqual(deleted, before);
  for (let cycle = 0; cycle < 3; cycle++) {
    currentSession.history.undo(currentSession.wasm);
    assert.deepEqual(context(currentSession, target), before);
    assertSavedContext(document, before, target);
    currentSession.history.redo(currentSession.wasm);
    assert.deepEqual(context(currentSession, target), deleted);
  }
  currentSession.history.undo(currentSession.wasm);
}
for (const text of ['링크', '가', 'A😀']) {
  const document = HwpDocument.createEmpty();
  document.createBlankDocument();
  const edit = session(document, position);
  edit.open().apply({ kind: 'save', text, uri });
  const len = Array.from(text).length;
  deleteAndRestore(document, edit, body, position, len - 1, 1);
  deleteAndRestore(document, edit, body, position, 0, len);
  assert.equal(edit.wasm.getCharPropertiesAt(0, 0, 0).textColor.toLowerCase(), '#0000ff');
  assert.equal(edit.wasm.getCharPropertiesAt(0, 0, 0).underline, true);
  document.free();
}
results.push('링크 마지막/전체/한 글자/astral 삭제: 3회 undo·redo 및 매 undo HWP/HWPX 왕복');

const chainDoc = HwpDocument.createEmpty();
chainDoc.createBlankDocument();
const chain = session(chainDoc, position);
chain.open().apply({ kind: 'save', text: '링크', uri });
const chainBefore = context(chain);
chain.history.execute(new DeleteTextCommand({ ...position, charOffset: 1 }, 1, 'backward', undefined, 10), chain.wasm);
chain.history.execute(new DeleteTextCommand(position, 1, 'backward', undefined, 20), chain.wasm);
chain.undo();
assert.equal(context(chain).links[0].text, '링');
chain.undo();
assert.deepEqual(context(chain), chainBefore);
chain.redo(); chain.redo(); chain.undo(); chain.undo();
assert.deepEqual(context(chain), chainBefore);
// 링크 뒤 일반 글자는 계속 링크 밖이며 일반 삭제의 300ms 병합은 유지한다.
chain.history.execute(new InsertTextCommand({ ...position, charOffset: 2 }, '뒤쪽'), chain.wasm);
chain.history.execute(new DeleteTextCommand({ ...position, charOffset: 3 }, 1, 'backward', undefined, 30), chain.wasm);
chain.history.execute(new DeleteTextCommand({ ...position, charOffset: 2 }, 1, 'backward', undefined, 40), chain.wasm);
chain.undo();
assert.equal(context(chain).text, '링크뒤쪽');
assert.equal(context(chain).links[0].text, '링크');
assert.equal(chain.wasm.getCharPropertiesAt(0, 0, 2).underline, false);
results.push('연속 링크 삭제별 정확한 복원·링크 뒤 일반 삭제 병합 및 바깥 서식 유지');

const nestedLink = context(nested, target).links[0];
deleteAndRestore(lh, nested, target, nestedPos, nestedLink.end - 1, 1);
const boxLink = context(box, boxTarget).links[0];
deleteAndRestore(boxDoc, box, boxTarget, boxPos, boxLink.end - 1, 1);
results.push('실제 한컴 중첩 셀·글상자 링크 삭제 undo·redo 및 HWP/HWPX 왕복');

const startDoc = HwpDocument.createEmpty();
startDoc.createBlankDocument();
const startSession = session(startDoc, position);
startSession.open().apply({ kind: 'save', text: '링크', uri });
startSession.history.execute(new InsertTextCommand(position, 'X'), startSession.wasm);
assert.equal(context(startSession).links[0].text, '링크');
assert.equal(startSession.wasm.getCharPropertiesAt(0, 0, 0).textColor.toLowerCase(), '#000000');
assert.equal(startSession.wasm.getCharPropertiesAt(0, 0, 0).underline, false);
assert.equal(context(startSession).links[0].start, 1);
assert.equal(context(startSession).text, 'X링크');
assertSavedContext(startDoc, context(startSession), body);
for (const method of ['exportHwp', 'exportHwpx']) {
  const reopened = new HwpDocument(startDoc[method]());
  const outside = JSON.parse(reopened.getCharPropertiesAt(0, 0, 0));
  const inside = JSON.parse(reopened.getCharPropertiesAt(0, 0, 1));
  assert.equal(outside.textColor.toLowerCase(), '#000000', method);
  assert.equal(outside.underline, false, method);
  assert.equal(inside.textColor.toLowerCase(), '#0000ff', method);
  assert.equal(inside.underline, true, method);
  reopened.free();
}
startSession.undo(); startSession.redo();
assert.equal(context(startSession).links[0].text, '링크');
assert.equal(startSession.wasm.getCharPropertiesAt(0, 0, 0).underline, false);
results.push('링크 시작 입력의 일반 서식·링크 바깥 범위, undo·redo 및 HWP/HWPX 왕복');

// 한컴 실측 확인창: 한 글자 삭제 대신 필드+표시 문자열을 하나의 undo로 지운다.
const deleteDoc = HwpDocument.createEmpty();
deleteDoc.createBlankDocument(); deleteDoc.insertText(0, 0, 0, '앞링크뒤');
const deleting = session(deleteDoc, position);
deleting.select(1, 3); deleting.open().apply({ kind: 'save', text: '링크', uri });
deleting.history.clear(deleting.wasm);
const beforeConfirm = context(deleting);
assert.equal(deleting.deleteAt(0), false); // 일반 텍스트
assert.equal(deleting.deleteAt(3), false); // 링크 뒤
assert.equal(deleting.deleteAt(1), true); // 링크 앞 Delete
assert.deepEqual(context(deleting), beforeConfirm); // 확인 전에는 무변경
const firstConfirmation = dialogs.deletion;
assert.equal(deleting.deleteAt(1), true); // repeat key: 모달 중복 방지
assert.equal(dialogs.deletion, firstConfirmation);
dialogs.deletion.cancel();
assert.deepEqual(context(deleting), beforeConfirm);
assert.equal(deleting.history.canUndo(), false);
assert.equal(deleting.deleteAt(2), true); // 링크 안 Delete
dialogs.deletion.remove();
assert.equal(context(deleting).text, '앞뒤');
assert.equal(context(deleting).links.length, 0);
assertSavedContext(deleteDoc, context(deleting), body);
for (let cycle = 0; cycle < 3; cycle++) {
  deleting.undo(); assert.deepEqual(context(deleting), beforeConfirm);
  assertSavedContext(deleteDoc, context(deleting), body);
  deleting.redo(); assert.equal(context(deleting).text, '앞뒤');
  assert.equal(context(deleting).links.length, 0);
}
deleting.undo();
results.push('Delete 확인 전·취소 무변경, 중복 모달 차단, 전체 링크 삭제·3회 undo/redo·HWP/HWPX');

// Backspace는 커서 바로 앞 글자로 대상을 정한다. 링크 끝에서도 전체 삭제를 확인한다.
deleting.history.clear(deleting.wasm);
assert.equal(deleting.backspaceAt(0), false); // 문단 시작
assert.equal(deleting.backspaceAt(1), false); // 링크 앞: 삭제 대상은 일반 텍스트
assert.equal(deleting.backspaceAt(4), false); // 링크 뒤 일반 텍스트
assert.equal(deleting.backspaceAt(3), true); // 링크 끝
assert.deepEqual(context(deleting), beforeConfirm);
dialogs.deletion.cancel();
assert.deepEqual(context(deleting), beforeConfirm);
assert.equal(deleting.history.canUndo(), false);
for (const offset of [3, 2]) { // 링크 끝과 내부
  assert.equal(deleting.backspaceAt(offset), true);
  dialogs.deletion.remove();
  assert.equal(context(deleting).text, '앞뒤');
  assert.equal(context(deleting).links.length, 0);
  assertSavedContext(deleteDoc, context(deleting), body);
  deleting.undo(); assert.deepEqual(context(deleting), beforeConfirm);
  assertSavedContext(deleteDoc, context(deleting), body);
  deleting.redo(); assert.equal(context(deleting).text, '앞뒤');
  deleting.undo(); assert.deepEqual(context(deleting), beforeConfirm);
}
results.push('Backspace 링크 끝·내부 전체 삭제, 바깥 경계 제외, 취소·undo/redo·HWP/HWPX');

// 필드 제거 뒤 텍스트 삭제 실패도 전체 snapshot으로 원자 복구한다.
assert.equal(deleting.deleteAt(1), true);
const originalDelete = deleting.wasm.deleteText;
deleting.wasm.deleteText = () => { throw new Error('forced-delete-failure'); };
assert.throws(() => dialogs.deletion.remove(), /forced-delete-failure/);
deleting.wasm.deleteText = originalDelete;
assert.deepEqual(context(deleting), beforeConfirm);
assert.equal(deleting.deleteAt(1), true);
deleteDoc.insertText(0, 0, 0, '외부');
const changedDuringDialog = context(deleting);
assert.throws(() => dialogs.deletion.remove(), /편집 상태가 바뀌었습니다/);
assert.deepEqual(context(deleting), changedDuringDialog);
assert.equal(deleting.deleteAt(3), true);
deleting.wasm._documentGeneration += 1;
assert.throws(() => dialogs.deletion.remove(), /편집 상태가 바뀌었습니다/);
assert.deepEqual(context(deleting), changedDuringDialog);
assert.equal(deleting.deleteAt(3), true);
deleting.readOnly();
assert.throws(() => dialogs.deletion.remove(), /편집 상태가 바뀌었습니다/);
assert.deepEqual(context(deleting), changedDuringDialog);
results.push('전체 삭제의 부분 실패 원자 복구·모달 중 외부 편집/문서 교체/읽기 전용 전환 차단');

for (const [document, edit, address] of [[lh, nested, target], [boxDoc, box, boxTarget]]) {
  for (const direction of ['forward', 'backward']) {
    const before = context(edit, address);
    const link = before.links[0];
    assert.equal(direction === 'forward' ? edit.deleteAt(link.start) : edit.backspaceAt(link.end), true);
    dialogs.deletion.remove();
    const after = context(edit, address);
    assert.equal(after.text, Array.from(before.text).slice(0, link.start).join('') + Array.from(before.text).slice(link.end).join(''));
    assert.equal(after.links.some(other => other.fieldId === link.fieldId), false);
    assertSavedContext(document, after, address);
    edit.undo(); assert.deepEqual(context(edit, address), before);
    assertSavedContext(document, before, address);
    edit.redo(); assert.deepEqual(context(edit, address), after);
    edit.undo();
  }
}
results.push('한컴 중첩 셀·글상자 전체 링크 Delete/Backspace 및 undo/redo·저장 왕복');

const out = process.env.RHWP_HYPERLINK_EVIDENCE_DIR;
if (out) {
  mkdirSync(out, { recursive: true });
  writeFileSync(join(out, 'wasm-command-validation.json'), JSON.stringify({ results, passed: results.length }, null, 2) + '\n');
}
console.log(JSON.stringify({ results, passed: results.length }, null, 2));
