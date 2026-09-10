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
export function confirmHyperlinkEdit(edit) { edit(); }
export class HyperlinkDialog {
  constructor(initial, apply) { this.initial = initial; this.apply = apply; current = this; }
  show() {}
}
`);
const toastModule = 'data:text/javascript,' + encodeURIComponent(`export function showToast(o) { throw new Error(o.message); }`);
registerHooks({ resolve(specifier, context, next) {
  if (specifier === '@/ui/hyperlink-dialog') return { url: dialogModule, shortCircuit: true };
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
const { SnapshotCommand } = await import(pathToFileURL(join(src, 'engine/command.ts')));
const { CommandHistory } = await import(pathToFileURL(join(src, 'engine/history.ts')));
const { hyperlinkCommand } = await import(pathToFileURL(join(src, 'command/commands/hyperlink.ts')));
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
  const services = { wasm, getInputHandler: () => ih, getContext: () => ({ isEditable: editable, isFormMode: false }) };
  return {
    wasm, history,
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

const out = process.env.RHWP_HYPERLINK_EVIDENCE_DIR;
if (out) {
  mkdirSync(out, { recursive: true });
  writeFileSync(join(out, 'wasm-command-validation.json'), JSON.stringify({ results, passed: results.length }, null, 2) + '\n');
}
console.log(JSON.stringify({ results, passed: results.length }, null, 2));
