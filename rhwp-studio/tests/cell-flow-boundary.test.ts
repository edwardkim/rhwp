import test, { after } from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createRequire } from 'node:module';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

// Node의 strip-only TypeScript loader는 command.ts의 parameter property를 실행할 수 없다.
// 실제 production 모듈을 임시 CommonJS로 변환해 source 복제 없이 동작을 검증한다.
const studioRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const runtimeRoot = mkdtempSync(path.join(tmpdir(), 'rhwp-cell-flow-boundary-'));
// npm bin 을 직접 spawn 하지 않는다: Windows 의 `.bin/tsc` 는 확장자 없는 shell 스크립트라
// 실행되지 않고, `.cmd` 는 Node 가 shell 없는 spawn 을 막는다(CVE-2024-27980 완화). 어느
// 쪽이든 status=null 로 죽어 테스트가 통째로 중단된다. typescript 의 JS 진입점을 현재
// node 로 실행하면 셸 없이도 모든 OS 에서 동일하게 동작한다.
const compiler = process.env.RHWP_STUDIO_TSC ?? process.execPath;
const compilerArgs = process.env.RHWP_STUDIO_TSC
  ? []
  : [path.join(studioRoot, 'node_modules', 'typescript', 'bin', 'tsc')];
const compilation = spawnSync(compiler, [
  ...compilerArgs,
  '--ignoreConfig',
  'src/engine/command.ts',
  'src/engine/history.ts',
  'src/engine/input-edit-invalidation.ts',
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
  `cell-flow test runtime compile failed:\n${compilation.stdout}${compilation.stderr}`,
);

const require = createRequire(import.meta.url);
const {
  DeleteTextCommand,
  InsertTextCommand,
  IMMEDIATE_TEXT_MUTATION_EFFECTS,
  NO_TEXT_MUTATION_EFFECTS,
  TextMutationEffectAccumulator,
  deleteTextWithMutationEffects,
  insertTextWithMutationEffects,
  replaceBodyTextWithMutationEffects,
  canUseDeferredCellTextReplace,
  replaceCellTextWithMutationEffects,
} = require(path.join(runtimeRoot, 'engine', 'command.js'));
const { CommandHistory } = require(path.join(runtimeRoot, 'engine', 'history.js'));

after(() => {
  rmSync(runtimeRoot, { recursive: true, force: true });
});

function depth1Position(charOffset = 0) {
  return {
    sectionIndex: 0,
    paragraphIndex: 0,
    charOffset,
    parentParaIndex: 5,
    controlIndex: 2,
    cellIndex: 3,
    cellParaIndex: 0,
    cellPath: [{ controlIndex: 2, cellIndex: 3, cellParaIndex: 0 }],
  };
}

function depth2Position(charOffset = 0) {
  return {
    ...depth1Position(charOffset),
    paragraphIndex: 2,
    controlIndex: 1,
    cellIndex: 4,
    cellParaIndex: 2,
    cellPath: [
      { controlIndex: 2, cellIndex: 3, cellParaIndex: 0 },
      { controlIndex: 1, cellIndex: 4, cellParaIndex: 2 },
    ],
  };
}

function mutationResult(cellFlowChanged) {
  return {
    ok: true,
    charOffset: 1,
    paginationDeferred: true,
    cellFlowChanged,
  };
}

class FakeWasm {
  constructor(...deferredResults) {
    this.deferredResults = [...deferredResults];
    this.bodyResults = [];
    this.calls = [];
  }

  queueBodyResult(result) {
    this.bodyResults.push(result);
  }

  replaceBodyTextLocal(...args) {
    this.calls.push({ name: 'body-local', args });
    return this.bodyResults.shift() ?? {
      ok: true,
      charOffset: args[2] + String(args[4]).length,
      documentPaginationPending: true,
      flowChanged: false,
    };
  }

  insertTextInCellDeferredPagination(...args) {
    this.calls.push({ name: 'deferred', args });
    const result = this.deferredResults.shift();
    assert.ok(result, 'deferred mutation result fixture exhausted');
    return result;
  }

  deleteTextInCellDeferredPagination(...args) {
    this.calls.push({ name: 'delete-deferred', args });
    const result = this.deferredResults.shift();
    assert.ok(result, 'deferred mutation result fixture exhausted');
    return result;
  }

  replaceTextInCellDeferredPagination(...args) {
    this.calls.push({ name: 'replace-deferred', args });
    const result = this.deferredResults.shift();
    assert.ok(result, 'deferred mutation result fixture exhausted');
    return result;
  }

  insertTextInCell(...args) {
    this.calls.push({ name: 'cell-immediate', args });
    return JSON.stringify({ ok: true, charOffset: args[5] + String(args[6]).length });
  }

  insertTextInCellByPath(...args) {
    this.calls.push({ name: 'path-immediate', args });
    return JSON.stringify({ ok: true, charOffset: args[3] + String(args[4]).length });
  }

  insertText(...args) {
    this.calls.push({ name: 'body-immediate', args });
    return JSON.stringify({ ok: true, charOffset: args[2] + String(args[3]).length });
  }

  deleteTextInCell(...args) {
    this.calls.push({ name: 'delete-cell', args });
    return JSON.stringify({ ok: true, charOffset: args[5] });
  }

  getTextInCell(...args) {
    return '1'.repeat(args[6]);
  }

  deleteTextInCellByPath(...args) {
    this.calls.push({ name: 'delete-path', args });
    return JSON.stringify({ ok: true, charOffset: args[3] });
  }

  deleteText(...args) {
    this.calls.push({ name: 'delete-body', args });
    return JSON.stringify({ ok: true, charOffset: args[2] });
  }

  getTextRange(...args) {
    return 'x'.repeat(args[3]);
  }
}

test('InsertTextCommand effect는 실행별로 한 번만 소비되고 undo에서 초기화된다', () => {
  const wasm = new FakeWasm(mutationResult(true), mutationResult(true));
  const command = new InsertTextCommand(depth1Position(), '1', 1_000);

  command.execute(wasm);
  assert.deepEqual(command.consumeTextMutationEffects(), {
    documentPaginationPending: true,
    flowChanged: true,
    paginationCompleted: false,
  });
  assert.deepEqual(command.consumeTextMutationEffects(), NO_TEXT_MUTATION_EFFECTS);

  command.execute(wasm);
  command.undo(wasm);
  assert.deepEqual(
    command.consumeTextMutationEffects(),
    NO_TEXT_MUTATION_EFFECTS,
    'undo must not expose the preceding execute effect',
  );
});

test('TextMutationEffectAccumulator는 묶음 effect를 OR 누적하고 한 번만 소비한다', () => {
  const accumulator = new TextMutationEffectAccumulator();
  accumulator.add(IMMEDIATE_TEXT_MUTATION_EFFECTS);
  accumulator.add({ documentPaginationPending: true, flowChanged: false, paginationCompleted: false });
  accumulator.add({ documentPaginationPending: true, flowChanged: true, paginationCompleted: false });

  assert.deepEqual(accumulator.consume(), {
    documentPaginationPending: true,
    flowChanged: true,
    paginationCompleted: true,
  });
  assert.deepEqual(accumulator.consume(), NO_TEXT_MUTATION_EFFECTS);

  accumulator.add({ documentPaginationPending: true, flowChanged: true, paginationCompleted: false });
  accumulator.clear();
  assert.deepEqual(accumulator.consume(), NO_TEXT_MUTATION_EFFECTS);
});

test('flat 셀 삭제는 deferred effect를 반환하고 undo는 즉시 pagination을 사용한다', () => {
  const wasm = new FakeWasm(mutationResult(true));
  const history = new CommandHistory();

  history.execute(new DeleteTextCommand(depth1Position(), 1, 'forward'), wasm);
  assert.deepEqual(history.consumeLastExecutionEffects(), {
    documentPaginationPending: true,
    flowChanged: true,
    paginationCompleted: false,
  });
  assert.equal(wasm.calls.at(-1).name, 'delete-deferred');
  assert.deepEqual(history.consumeLastExecutionEffects(), NO_TEXT_MUTATION_EFFECTS);
  history.undo(wasm);
  assert.equal(wasm.calls.at(-1).name, 'cell-immediate');
  assert.deepEqual(history.consumeLastExecutionEffects(), NO_TEXT_MUTATION_EFFECTS);
});

test('history merge는 이전 effect를 누수하지 않고 redo 때 실제 결과를 다시 계산한다', () => {
  const wasm = new FakeWasm(
    mutationResult(true),
    mutationResult(false),
    mutationResult(true),
    mutationResult(false),
  );
  const history = new CommandHistory();

  history.execute(new InsertTextCommand(depth1Position(0), '1', 1_000), wasm);
  assert.deepEqual(history.consumeLastExecutionEffects(), {
    documentPaginationPending: true,
    flowChanged: true,
    paginationCompleted: false,
  });
  assert.deepEqual(history.consumeLastExecutionEffects(), NO_TEXT_MUTATION_EFFECTS);

  // 연속 위치·300ms 이내이므로 두 command는 하나의 history entry로 병합된다.
  history.execute(new InsertTextCommand(depth1Position(1), '2', 1_100), wasm);
  assert.deepEqual(
    history.consumeLastExecutionEffects(),
    { documentPaginationPending: true, flowChanged: false, paginationCompleted: false },
    'merged history entry must not inherit the preceding true effect',
  );

  assert.deepEqual(history.undo(wasm), depth1Position(0));
  assert.deepEqual(history.consumeLastExecutionEffects(), NO_TEXT_MUTATION_EFFECTS);

  history.redo(wasm);
  assert.deepEqual(
    history.consumeLastExecutionEffects(),
    { documentPaginationPending: true, flowChanged: true, paginationCompleted: false },
    'redo must expose its newly calculated mutation result',
  );

  history.undo(wasm);
  assert.deepEqual(
    history.consumeLastExecutionEffects(),
    NO_TEXT_MUTATION_EFFECTS,
    'undo must clear an unconsumed redo effect',
  );

  history.redo(wasm);
  assert.deepEqual(
    history.consumeLastExecutionEffects(),
    { documentPaginationPending: true, flowChanged: false, paginationCompleted: false },
    'a later redo must replace, not reuse, the preceding true result',
  );
});

test('insert helper는 depth 1 셀만 deferred로 보내고 depth 2 셀은 path immediate로 보낸다', () => {
  const wasm = new FakeWasm(mutationResult(true));

  assert.deepEqual(insertTextWithMutationEffects(wasm, depth1Position(), '1'), {
    documentPaginationPending: true,
    flowChanged: true,
    paginationCompleted: false,
  });
  assert.equal(wasm.calls.length, 1);
  assert.equal(wasm.calls[0].name, 'deferred');

  wasm.calls.length = 0;
  assert.deepEqual(
    insertTextWithMutationEffects(wasm, depth2Position(), '1'),
    IMMEDIATE_TEXT_MUTATION_EFFECTS,
  );
  assert.equal(wasm.calls.length, 1);
  assert.equal(wasm.calls[0].name, 'path-immediate');
  assert.equal(JSON.parse(wasm.calls[0].args[2]).length, 2);

  const fallbackWasm = new FakeWasm({
    ok: true,
    charOffset: 1,
    paginationDeferred: false,
    cellFlowChanged: false,
  });
  assert.deepEqual(
    insertTextWithMutationEffects(fallbackWasm, depth1Position(), '1'),
    IMMEDIATE_TEXT_MUTATION_EFFECTS,
    'immediate bridge fallback은 deferred pending을 만들지 않아야 한다',
  );
});

test('delete helper는 depth 1 셀만 deferred로 보내고 depth 2 셀은 path immediate로 보낸다', () => {
  const wasm = new FakeWasm(mutationResult(false));

  assert.deepEqual(deleteTextWithMutationEffects(wasm, depth1Position(1), 1), {
    documentPaginationPending: true,
    flowChanged: false,
    paginationCompleted: false,
  });
  assert.equal(wasm.calls.length, 1);
  assert.equal(wasm.calls[0].name, 'delete-deferred');

  wasm.calls.length = 0;
  assert.deepEqual(
    deleteTextWithMutationEffects(wasm, depth2Position(1), 1),
    IMMEDIATE_TEXT_MUTATION_EFFECTS,
  );
  assert.equal(wasm.calls.length, 1);
  assert.equal(wasm.calls[0].name, 'delete-path');
  assert.equal(JSON.parse(wasm.calls[0].args[2]).length, 2);

  const fallbackWasm = new FakeWasm({
    ok: true,
    charOffset: 0,
    paginationDeferred: false,
    cellFlowChanged: false,
  });
  assert.deepEqual(
    deleteTextWithMutationEffects(fallbackWasm, depth1Position(1), 1),
    IMMEDIATE_TEXT_MUTATION_EFFECTS,
    '구형 WASM bridge fallback은 deferred pending을 만들지 않아야 한다',
  );
});

test('depth-1 IME replace는 atomic deferred mutation 한 번만 사용한다', () => {
  const wasm = new FakeWasm(mutationResult(false));
  const position = depth1Position(7);

  assert.equal(canUseDeferredCellTextReplace(position, 1, '하'), true);
  assert.deepEqual(
    replaceCellTextWithMutationEffects(wasm, position, 1, '하'),
    {
      documentPaginationPending: true,
      flowChanged: false,
      paginationCompleted: false,
    },
  );
  assert.deepEqual(wasm.calls, [{
    name: 'replace-deferred',
    args: [0, 5, 2, 3, 0, 7, 1, '하'],
  }]);
});

test('flow boundary replace effect는 pre-caret flush 신호를 보존한다', () => {
  const wasm = new FakeWasm(mutationResult(true));

  assert.deepEqual(
    replaceCellTextWithMutationEffects(wasm, depth1Position(7), 1, '가나다라마바사아'),
    {
      documentPaginationPending: true,
      flowChanged: true,
      paginationCompleted: false,
    },
  );
});

test('중첩 셀과 빈 replacement는 atomic cell replace 대상이 아니다', () => {
  assert.equal(canUseDeferredCellTextReplace(depth2Position(), 1, '하'), false);
  assert.equal(canUseDeferredCellTextReplace(depth1Position(), 1, ''), false);
  assert.equal(canUseDeferredCellTextReplace(depth1Position(), 0, '하'), false);
  assert.equal(canUseDeferredCellTextReplace(depth1Position(), 1, '가나다라마바사아자'), false);
});

test('atomic API fallback 결과는 immediate-completed effect다', () => {
  const wasm = new FakeWasm({
    ok: true,
    charOffset: 8,
    paginationDeferred: false,
    cellFlowChanged: false,
  });

  assert.deepEqual(
    replaceCellTextWithMutationEffects(wasm, depth1Position(7), 1, '하'),
    {
      documentPaginationPending: false,
      flowChanged: false,
      paginationCompleted: true,
    },
  );
});

test('본문 insert와 delete command는 stable local replace effect를 반환한다', () => {
  const wasm = new FakeWasm();
  const position = { sectionIndex: 0, paragraphIndex: 2, charOffset: 3 };

  const insert = new InsertTextCommand(position, '가');
  insert.execute(wasm);
  assert.deepEqual(insert.consumeTextMutationEffects(), {
    documentPaginationPending: true,
    flowChanged: false,
    paginationCompleted: false,
  });
  assert.deepEqual(wasm.calls.at(-1), {
    name: 'body-local',
    args: [0, 2, 3, 0, '가'],
  });

  const deletion = new DeleteTextCommand(position, 1, 'forward');
  deletion.execute(wasm);
  assert.deepEqual(deletion.consumeTextMutationEffects(), {
    documentPaginationPending: true,
    flowChanged: false,
    paginationCompleted: false,
  });
  assert.deepEqual(wasm.calls.at(-1), {
    name: 'body-local',
    args: [0, 2, 3, 1, ''],
  });
});

test('본문 flow boundary local result는 pagination 완료 effect로 전달된다', () => {
  const wasm = new FakeWasm();
  const position = { sectionIndex: 0, paragraphIndex: 2, charOffset: 3 };
  wasm.queueBodyResult({
    ok: true,
    charOffset: 4,
    documentPaginationPending: false,
    flowChanged: true,
  });

  assert.deepEqual(
    replaceBodyTextWithMutationEffects(wasm, position, 0, '가'),
    {
      documentPaginationPending: false,
      flowChanged: true,
      paginationCompleted: true,
    },
  );
});

test('본문 command undo는 local pending을 만들지 않고 기존 immediate API를 사용한다', () => {
  const wasm = new FakeWasm();
  const position = { sectionIndex: 0, paragraphIndex: 2, charOffset: 3 };
  const command = new InsertTextCommand(position, '가');

  command.execute(wasm);
  command.undo(wasm);

  assert.equal(wasm.calls.at(-1).name, 'delete-body');
  assert.deepEqual(command.consumeTextMutationEffects(), NO_TEXT_MUTATION_EFFECTS);
});
