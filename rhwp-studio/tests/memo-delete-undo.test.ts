import test, { after } from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createRequire } from 'node:module';
import { existsSync, mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

// 메모 기준 글자에 닿는 DeleteTextCommand 의 undo 계약. 실제 WASM 왕복은 맨 아래 테스트가
// tests/support/memo-wasm.runner.mjs 로 확인한다. command.ts 의 parameter property 때문에
// cell-flow-boundary.test.ts 와 같이 production 모듈을 임시 CommonJS 로 변환해 실행한다.
const studioRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const runtimeRoot = mkdtempSync(path.join(tmpdir(), 'rhwp-memo-delete-undo-'));
const compiler = process.env.RHWP_STUDIO_TSC ?? process.execPath;
const compilerArgs = process.env.RHWP_STUDIO_TSC
  ? []
  : [path.join(studioRoot, 'node_modules', 'typescript', 'bin', 'tsc')];
const compilation = spawnSync(compiler, [
  ...compilerArgs,
  '--ignoreConfig',
  'src/engine/command.ts',
  'src/engine/history.ts',
  '--target', 'ES2022',
  '--module', 'commonjs',
  '--rootDir', 'src',
  '--outDir', runtimeRoot,
  '--skipLibCheck',
  '--noCheck',
], {
  cwd: studioRoot,
  encoding: 'utf8',
});

assert.equal(
  compilation.status,
  0,
  `memo delete test runtime compile failed:\n${compilation.stdout}${compilation.stderr}`,
);

const require = createRequire(import.meta.url);
const { DeleteTextCommand } = require(path.join(runtimeRoot, 'engine', 'command.js'));
const { CommandHistory } = require(path.join(runtimeRoot, 'engine', 'history.js'));
const { touchesMemoAnchor } = require(path.join(runtimeRoot, 'core', 'memo.js'));

after(() => {
  rmSync(runtimeRoot, { recursive: true, force: true });
});

const body = { sectionIndex: 0, paragraphIndex: 2, charOffset: 0 };
const bodyTarget = { section: 0, para: 2, cellPath: [] };

function memo(start, end, extra = {}) {
  return { sectionIndex: 0, paraIndex: 2, startCharIdx: start, endCharIdx: end, path: null, ...extra };
}

class FakeWasm {
  constructor({ links = [], memos = [] } = {}) {
    this.links = links;
    this.memos = memos;
    this.calls = [];
    this.nextFragment = 1;
  }

  getHyperlinkContext() { return { text: '', links: this.links }; }
  getMemoAnchors() { this.calls.push({ name: 'memos' }); return this.memos; }
  captureDeleteRange(...args) {
    this.calls.push({ name: 'capture', args });
    return this.nextFragment++;
  }
  restoreDeleteFragment(id) { this.calls.push({ name: 'restore', args: [id] }); return ''; }
  discardDeleteFragment(id) { this.calls.push({ name: 'discard', args: [id] }); }
  replaceBodyTextLocal(...args) {
    this.calls.push({ name: 'delete', args });
    return { ok: true, charOffset: args[2], documentPaginationPending: false, flowChanged: false };
  }
  deleteTextInCellByPath(...args) { this.calls.push({ name: 'delete-cell', args }); }
  insertText(...args) { this.calls.push({ name: 'insert', args }); }
  insertTextInCellByPath(...args) { this.calls.push({ name: 'insert-cell', args }); }

  names() { return this.calls.map(call => call.name); }
}

test('메모 기준 범위와 겹치거나 맞닿는 삭제만 판정한다', () => {
  const anchors = [memo(2, 5)];
  assert.equal(touchesMemoAnchor(anchors, bodyTarget, 2, 5), true, '기준 전체');
  assert.equal(touchesMemoAnchor(anchors, bodyTarget, 4, 5), true, '마지막 글자');
  assert.equal(touchesMemoAnchor(anchors, bodyTarget, 0, 3), true, '앞쪽 걸침');
  assert.equal(touchesMemoAnchor(anchors, bodyTarget, 1, 2), true, '바로 앞 글자');
  assert.equal(touchesMemoAnchor(anchors, bodyTarget, 5, 6), true, '바로 뒤 글자');
  assert.equal(touchesMemoAnchor(anchors, bodyTarget, 0, 1), false, '떨어진 앞 글자');
  assert.equal(touchesMemoAnchor(anchors, bodyTarget, 6, 7), false, '떨어진 뒤 글자');
  assert.equal(touchesMemoAnchor([memo(3, 3)], bodyTarget, 2, 4), true, '길이 0 메모를 품은 삭제');
  assert.equal(touchesMemoAnchor([memo(3, 3)], bodyTarget, 1, 3), true, '길이 0 메모 바로 앞 삭제');
  assert.equal(touchesMemoAnchor([memo(3, 3)], bodyTarget, 0, 2), false, '길이 0 메모와 떨어진 삭제');
  assert.equal(touchesMemoAnchor([memo(2, 5, { paraIndex: 3 })], bodyTarget, 2, 5), false, '다른 문단');
  assert.equal(touchesMemoAnchor([memo(2, 5, { sectionIndex: 1 })], bodyTarget, 2, 5), false, '다른 구역');
  assert.equal(
    touchesMemoAnchor([memo(null, null, { paraIndex: null })], bodyTarget, 0, 10),
    false,
    '위치를 알 수 없는 메모',
  );
});

test('표 칸·글상자 메모는 전체 경로가 같을 때만 겹친다', () => {
  const cellTarget = { section: 0, para: 2, cellPath: [[1, 3, 0], [0, 2, 1]] };
  const nested = memo(0, 2, { path: [
    { type: 'cell', controlIndex: 1, cellIndex: 3, paraIndex: 0 },
    { type: 'cell', controlIndex: 0, cellIndex: 2, paraIndex: 1 },
  ] });
  assert.equal(touchesMemoAnchor([nested], cellTarget, 0, 1), true);
  assert.equal(touchesMemoAnchor([nested], { ...cellTarget, cellPath: [[1, 3, 0], [0, 2, 0]] }, 0, 1), false);
  assert.equal(touchesMemoAnchor([nested], { ...cellTarget, cellPath: [[1, 3, 0]] }, 0, 1), false);
  assert.equal(touchesMemoAnchor([nested], bodyTarget, 0, 1), false, '본문 같은 범위');
  const textBox = memo(0, 2, { path: [{ type: 'textbox', controlIndex: 4, paraIndex: 0 }] });
  assert.equal(touchesMemoAnchor([textBox], { section: 0, para: 2, cellPath: [[4, 0, 0]] }, 1, 2), true);
});

test('링크·메모에 닿지 않는 삭제는 문단 조각 없이 글자 재삽입으로 되돌린다', () => {
  const wasm = new FakeWasm({ memos: [memo(5, 8)] });
  const history = new CommandHistory();
  history.execute(new DeleteTextCommand({ ...body, charOffset: 1 }, 2, 'forward', '가나', 100), wasm);
  history.undo(wasm);
  assert.deepEqual(wasm.names(), ['memos', 'delete', 'insert']);
  assert.deepEqual(wasm.calls.at(-1).args, [0, 2, 1, '가나']);
});

test('메모 기준 글자 전체 삭제는 문단 조각을 잡아 undo 가 메모를 되살린다', () => {
  const wasm = new FakeWasm({ memos: [memo(1, 3)] });
  const history = new CommandHistory();
  history.execute(new DeleteTextCommand({ ...body, charOffset: 1 }, 2, 'forward', '메모', 100), wasm);
  assert.deepEqual(wasm.names(), ['memos', 'capture', 'delete']);
  assert.deepEqual(wasm.calls[1].args, [0, 2, 2]);

  history.undo(wasm);
  assert.deepEqual(wasm.calls.at(-1), { name: 'restore', args: [1] });
  assert.equal(wasm.names().includes('insert'), false, '조각 복원은 글자 재삽입을 함께 하지 않는다');

  // redo 는 execute 를 다시 불러 새 조각을 잡는다.
  history.redo(wasm);
  assert.deepEqual(wasm.names().slice(-3), ['memos', 'capture', 'delete']);
  history.undo(wasm);
  assert.deepEqual(wasm.calls.at(-1), { name: 'restore', args: [2] });
});

test('메모 기준 글자를 한 글자씩 지우면 각 삭제가 따로 되돌려진다', () => {
  const wasm = new FakeWasm({ memos: [memo(0, 2)] });
  const history = new CommandHistory();
  // Backspace 두 번(300ms 이내). 조각을 가진 삭제는 병합하지 않는다.
  history.execute(new DeleteTextCommand({ ...body, charOffset: 1 }, 1, 'backward', '모', 100), wasm);
  wasm.memos = [memo(0, 1)];
  history.execute(new DeleteTextCommand(body, 1, 'backward', '메', 150), wasm);
  assert.equal(wasm.names().filter(name => name === 'capture').length, 2);

  history.undo(wasm);
  assert.deepEqual(wasm.calls.at(-1), { name: 'restore', args: [2] });
  history.undo(wasm);
  assert.deepEqual(wasm.calls.at(-1), { name: 'restore', args: [1] });
});

test('링크가 겹치면 메모 목록을 묻지 않는다', () => {
  const wasm = new FakeWasm({ links: [{ fieldId: 7, start: 0, end: 3, text: '', uri: '' }] });
  new DeleteTextCommand(body, 1, 'forward', '가', 100).execute(wasm);
  assert.deepEqual(wasm.names(), ['capture', 'delete']);
});

test('중첩 표 칸의 메모 삭제도 칸 문단을 잡는다', () => {
  const position = {
    sectionIndex: 0, paragraphIndex: 1, charOffset: 0, parentParaIndex: 2,
    controlIndex: 1, cellIndex: 3, cellParaIndex: 0,
    cellPath: [
      { controlIndex: 1, cellIndex: 3, cellParaIndex: 0 },
      { controlIndex: 0, cellIndex: 2, cellParaIndex: 1 },
    ],
  };
  const wasm = new FakeWasm({ memos: [memo(0, 1, { path: [
    { type: 'cell', controlIndex: 1, cellIndex: 3, paraIndex: 0 },
    { type: 'cell', controlIndex: 0, cellIndex: 2, paraIndex: 1 },
  ] })] });
  const history = new CommandHistory();
  history.execute(new DeleteTextCommand(position, 1, 'forward', '칸', 100), wasm);
  assert.deepEqual(wasm.names(), ['memos', 'capture', 'delete-cell']);
  assert.deepEqual(wasm.calls[1].args, [0, 2, 2]);
  history.undo(wasm);
  assert.deepEqual(wasm.calls.at(-1), { name: 'restore', args: [1] });
});

test('실제 WASM: 메모 기준 글자 삭제·선택 삭제 undo/redo 와 HWP/HWPX 왕복', (t) => {
  if (!existsSync(path.join(studioRoot, '..', 'pkg', 'rhwp_bg.wasm'))) {
    t.skip('fresh WASM 필요: scripts/wasm-pack-locked.sh --target web --out-dir pkg --dev');
    return;
  }
  const result = spawnSync(process.execPath, ['--experimental-transform-types', '--no-warnings',
    path.join(studioRoot, 'tests', 'support', 'memo-wasm.runner.mjs')], { encoding: 'utf8' });
  assert.ifError(result.error);
  assert.equal(result.status, 0, `${result.stdout}
${result.stderr}`);
  assert.match(result.stdout, /MEMO_WASM_OK/);
});
