// 실제 command/history/bridge/InputHandler를 실행한다. pkg/pkg-node를 읽지 않는다.
import assert from 'node:assert/strict';
import { registerHooks } from 'node:module';
import { dirname, join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const src = join(dirname(fileURLToPath(import.meta.url)), '../../src');
registerHooks({
  resolve(specifier, context, next) {
    if (specifier === '@wasm/rhwp.js') return {
      url: 'data:text/javascript,export default function init(){throw Error("WASM unavailable")};export class HwpDocument{};export function version(){return "test"}',
      shortCircuit: true,
    };
    if (specifier.startsWith('@/')) return { url: pathToFileURL(join(src, specifier.slice(2) + '.ts')).href, shortCircuit: true };
    if (/^\.{1,2}\//.test(specifier) && !/\.[cm]?[tj]s$/.test(specifier)) {
      return { url: pathToFileURL(join(dirname(fileURLToPath(context.parentURL)), specifier + '.ts')).href, shortCircuit: true };
    }
    return next(specifier, context);
  },
});
const { ApplyCharFormatCommand } = await import(pathToFileURL(join(src, 'engine/command.ts')));
const { CommandHistory } = await import(pathToFileURL(join(src, 'engine/history.ts')));
const { InputHandler } = await import(pathToFileURL(join(src, 'engine/input-handler.ts')));
const { WasmBridge } = await import(pathToFileURL(join(src, 'core/wasm-bridge.ts')));
const { validateCharShapeRuns } = await import(pathToFileURL(join(src, 'core/char-shape-runs.ts')));
const { CharFormatError, CharFormatRecoveryError } = await import(pathToFileURL(join(src, 'core/char-format-error.ts')));

const pos = (p, off) => ({ sectionIndex: 0, paragraphIndex: p, charOffset: off });
const run = (id, start = 0, end = 3) => ({ startOffset: start, endOffset: end, charShapeId: id });
function fixture(cell = false) {
  const state = [[run(1, 0, 1), run(2, 1, 3)], [run(3)], [run(4)]];
  const before = structuredClone(state);
  const restores = [];
  const applyError = new Error('apply paragraph 2');
  const rollbackError = new Error('restore paragraph 1');
  const flags = { apply: false, restore: new Set(), capture: false };
  let batches = 0;
  const wasm = {
    getParagraphLength: () => 3,
    getCharShapeRuns: (_, p) => {
      if (flags.capture) throw new Error('capture failure');
      return structuredClone(state[p]);
    },
    applyCharFormat: (_, p) => {
      // 실패한 호출도 변경했을 수 있다.
      state[p] = state[p].map(r => ({ ...r, charShapeId: r.charShapeId + 10 }));
      if (flags.apply && p === 2) throw applyError;
    },
    setCharShapeRuns: (_, p, _s, _e, runs) => {
      restores.push(p);
      if (flags.restore.has(p)) throw rollbackError;
      state[p] = structuredClone(runs);
    },
    runInBatch: fn => { batches++; return fn(); },
  };
  const from = pos(0, 0), to = pos(2, 3);
  if (cell) {
    const para = path => JSON.parse(path).at(-1).cellParaIndex;
    wasm.getCellParagraphLengthByPath = () => 3;
    wasm.getCharShapeRunsInCellByPath = (sec, _p, path, s, e) => wasm.getCharShapeRuns(sec, para(path), s, e);
    wasm.applyCharFormatInCellByPath = (sec, _p, path, s, e, json) => wasm.applyCharFormat(sec, para(path), s, e, json);
    wasm.setCharShapeRunsInCellByPath = (sec, _p, path, s, e, runs) => wasm.setCharShapeRuns(sec, para(path), s, e, runs);
    for (const [p, index] of [[from, 0], [to, 2]]) Object.assign(p, {
      parentParaIndex: 0, controlIndex: 0, cellIndex: 0, cellParaIndex: index,
      cellPath: [{ controlIndex: 0, cellIndex: 0, cellParaIndex: 0 }, { controlIndex: 0, cellIndex: 0, cellParaIndex: index }],
    });
  }
  return { state, before, restores, flags, wasm, applyError, rollbackError,
    history: new CommandHistory(), command: new ApplyCharFormatCommand(from, to, { shadeColor: '#ffff00' }),
    batches: () => batches };
}

for (const cell of [false, true]) {
  const f = fixture(cell);
  f.flags.apply = true;
  f.flags.restore.add(1);
  assert.throws(() => f.history.execute(f.command, f.wasm), e => {
    assert.ok(e instanceof CharFormatRecoveryError);
    assert.deepEqual(e.errors, [f.applyError, f.rollbackError]);
    return true;
  });
  assert.deepEqual(f.restores, [2, 1, 0], '한 복원 실패가 나머지를 중단하지 않는다');
  assert.deepEqual(f.state[0], f.before[0]);
  assert.notDeepEqual(f.state[1], f.before[1]);
  assert.equal(f.history.peekUndoTop(), f.command);
  assert.throws(() => f.history.undo(f.wasm), CharFormatRecoveryError);
  assert.equal(f.history.peekUndoTop(), f.command, 'Undo 재실패도 정보를 보존한다');
  f.flags.restore.clear();
  f.flags.apply = false;
  f.history.undo(f.wasm);
  assert.deepEqual(f.state, f.before);
  f.history.redo(f.wasm);
  const after = structuredClone(f.state);
  f.history.undo(f.wasm);
  f.flags.restore.add(1);
  assert.throws(() => f.history.redo(f.wasm), CharFormatRecoveryError);
  assert.equal(f.history.peekUndoTop(), f.command, '부분 Redo는 Undo 대상이다');
  assert.equal(f.history.canRedo(), false);
  f.flags.restore.clear();
  f.history.undo(f.wasm);
  assert.deepEqual(f.state, f.before);
  f.history.redo(f.wasm);
  assert.deepEqual(f.state, after);
  assert.equal(f.batches() > 0, cell);
}

{
  const f = fixture();
  f.flags.capture = true;
  assert.throws(() => f.history.execute(f.command, f.wasm), CharFormatError);
  assert.deepEqual(f.state, f.before);
  assert.equal(f.history.canUndo(), false);
  f.flags.capture = false;
  f.flags.apply = true;
  assert.throws(() => f.history.execute(f.command, f.wasm), e => e.cause === f.applyError);
  assert.deepEqual(f.state, f.before);
  assert.equal(f.history.canUndo(), false, 'rollback 성공은 실패 command를 기록하지 않는다');
}

// 새 오류 계약이 다른 command의 기존 실패시 제거 계약을 바꾸지 않는다.
{
  const f = fixture();
  f.history.execute(f.command, f.wasm);
  f.history.undo(f.wasm);
  const redo = f.history.peekRedoTop();
  f.history.execute(new ApplyCharFormatCommand(pos(0, 0), pos(0, 0), {}), f.wasm);
  assert.equal(f.history.canUndo(), false);
  assert.equal(f.history.peekRedoTop(), redo, '빈 선택은 Redo를 보존한다');
  f.flags.apply = true;
  assert.throws(() => f.history.execute(new ApplyCharFormatCommand(pos(0, 0), pos(2, 3), {}), f.wasm));
  assert.equal(f.history.peekRedoTop(), redo, 'rollback 성공도 기존 Redo를 보존한다');
  assert.deepEqual(f.state, f.before);
}

{
  const history = new CommandHistory();
  const command = { type: 'other', execute: () => pos(0, 0), undo: () => { throw Error('other'); }, mergeWith: () => null };
  history.execute(command, {});
  assert.throws(() => history.undo({}), /other/);
  assert.equal(history.canUndo(), false);
}

// 실제 bridge: 미열림·구버전 binding·직접 payload 검증, mutation 전 실패.
const wasm = Object.create(WasmBridge.prototype);
const methods = [
  () => wasm.getCharShapeRuns(0, 0, 0, 3),
  () => wasm.setCharShapeRuns(0, 0, 0, 3, [run(1)]),
  () => wasm.getCharShapeRunsInCellByPath(0, 0, '[]', 0, 3),
  () => wasm.setCharShapeRunsInCellByPath(0, 0, '[]', 0, 3, [run(1)]),
];
wasm.doc = null;
for (const call of methods) assert.throws(call, /문서가 로드되지/);
wasm.doc = {};
for (const call of methods) assert.throws(call, /최신 WASM/);
for (const bad of [null, 1, [1], [run(-1)], [run(1, 1, 3)], [run(1), run(2)], [{ ...run(1), extra: true }]]) {
  assert.throws(() => validateCharShapeRuns(bad, 0, 3));
}
assert.deepEqual(validateCharShapeRuns([run(1)], 0, 3), [run(1)]);

// 실제 UI command 라우터와 Undo/Redo 경계: 안내하고 예상치 못한 오류는 재전파한다.
const handler = Object.create(InputHandler.prototype);
Object.assign(handler, {
  wasm, history: new CommandHistory(),
  cursor: { getPosition: () => pos(0, 0), getRect: () => null },
  isOperationAllowedInEditMode: () => true,
  flushDeferredPaginationIfNeeded: () => {},
});
const notices = [];
globalThis.alert = message => notices.push(message);
const previousError = console.error;
console.error = () => {};
try {
  handler.executeOperation({ kind: 'command', command: new ApplyCharFormatCommand(pos(0, 0), pos(0, 3), {}) });
  assert.match(notices.pop(), /최신 WASM/);
  assert.equal(handler.history.canUndo(), false);
  for (const method of ['handleUndo', 'handleRedo']) {
    const errors = new CharFormatRecoveryError([new Error('restore')]);
    handler.history = { undo: () => { throw errors; }, redo: () => { throw errors; } };
    let refresh = 0;
    handler.prepareTextMutationBeforeCursor = () => {};
    handler.resetDerivedStateAfterHistoryJump = () => {};
    handler.afterEdit = () => { refresh++; };
    handler[method]();
    assert.match(notices.pop(), /Undo/);
    assert.equal(refresh, 1);
  }
  const unexpected = new Error('unrelated');
  handler.history = { execute: () => { throw unexpected; } };
  assert.throws(() => handler.executeOperation({ kind: 'command', command: { type: 'other' } }), e => e === unexpected);
} finally { console.error = previousError; }
console.log('CHAR_FORMAT_RECOVERY_OK');
