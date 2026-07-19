import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [Task #2328] 스냅샷 상한 정합 + 예외 안전 스택 이동 소스 가드.
//
// node --test 는 strip-only TS 라 engine 클래스(parameter property 포함)를
// 실행할 수 없어, 이 저장소의 undo 테스트 관례대로 소스 배선을 검증한다.
// 행위 증명은 브라우저 실동작(수정 전/후 60회 스냅샷 + 오래된 undo 무예외)으로
// 별도 수행한다 (PR 검증 섹션).

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const source = (rel: string): string => readFileSync(join(rootDir, rel), 'utf8');

/** `undo(...) {` ~ 다음 메서드 전까지의 블록을 추출한다. */
function methodBlock(src: string, signature: string): string {
  const start = src.indexOf(signature);
  assert.notEqual(start, -1, `${signature} not found`);
  // 다음 최상위 메서드까지( '\n  ' + 선택적 접근자/async/get/set + 식별자() ).
  // 접근자 접두어를 허용해 modifier 붙은 이웃 메서드로 블록이 새지 않게 한다.
  const next = src.slice(start + signature.length)
    .search(/\n {2}(?:public |private |protected |static |async |get |set )*[a-zA-Z][\w]*\s*\(/);
  return next === -1 ? src.slice(start) : src.slice(start, start + signature.length + next);
}

const history = source('src/engine/history.ts');
const commandFull = source('src/engine/command.ts');
// execute/undo 시그니처가 커맨드 클래스마다 반복되므로 SnapshotCommand 클래스
// 본문으로 범위를 좁힌다(다음 export class 경계까지 — 뒤 클래스로의 누출 방지).
const snapClassStart = commandFull.indexOf('export class SnapshotCommand');
assert.notEqual(snapClassStart, -1, 'SnapshotCommand 클래스 not found');
const snapClassEndRel = commandFull.slice(snapClassStart + 1).indexOf('\nexport class ');
const command = snapClassEndRel === -1
  ? commandFull.slice(snapClassStart)
  : commandFull.slice(snapClassStart, snapClassStart + 1 + snapClassEndRel);

test('[결함2] undo 는 op-우선 + 실패시-드롭 하이브리드다(pop-먼저 금지, 락업 금지)', () => {
  const block = methodBlock(history, 'undo(wasm: WasmBridge): DocumentPosition | null {');
  // 성공 경로: peek → try{command.undo} → pop → redo.push.
  const idxPeek = block.indexOf('this.undoStack[this.undoStack.length - 1]');
  const idxUndoCall = block.indexOf('command.undo(wasm)');
  const idxRedoPush = block.indexOf('this.redoStack.push(command)');
  assert.ok(idxPeek !== -1 && idxUndoCall !== -1 && idxRedoPush !== -1);
  assert.ok(idxPeek < idxUndoCall && idxUndoCall < idxRedoPush,
    'peek → command.undo → redo.push 순서여야 함');
  // op 전에 pop 하지 않는다(성공 엔트리 무손실).
  assert.ok(!/const command = this\.undoStack\.pop\(\);[\s\S]*command\.undo/.test(block),
    'pop-먼저(pop 후 undo) 패턴 잔존');
  // 실패 경로: try/catch 로 오염 엔트리를 pop·discard 후 전파(락업 방지).
  // pop/discard 순서는 무관(JS 스택 vs WASM id 해제, 독립) — 존재만 강제한다.
  const catchBody = block.slice(block.search(/\}\s*catch/));
  assert.match(block, /try\s*\{[\s\S]*command\.undo\(wasm\)[\s\S]*\}\s*catch/, 'command.undo 를 try 로 감싸야 함');
  assert.match(catchBody, /this\.undoStack\.pop\(\)/, 'catch 에서 오염 엔트리 pop');
  assert.match(catchBody, /discard\?\.\(wasm\)/, 'catch 에서 스냅샷 discard');
  assert.match(catchBody, /throw/, 'catch 에서 rethrow');
});

test('[결함2] redo 도 execute-우선 + 실패시-드롭 하이브리드다', () => {
  const block = methodBlock(history, 'redo(wasm: WasmBridge): DocumentPosition | null {');
  const idxPeek = block.indexOf('this.redoStack[this.redoStack.length - 1]');
  const idxExec = block.indexOf('command.execute(wasm)');
  const idxUndoPush = block.indexOf('this.undoStack.push(command)');
  assert.ok(idxPeek < idxExec && idxExec < idxUndoPush,
    'peek → execute → undo.push 순서여야 함');
  const catchBody = block.slice(block.search(/\}\s*catch/));
  assert.match(block, /try\s*\{[\s\S]*command\.execute\(wasm\)[\s\S]*\}\s*catch/, 'command.execute 를 try 로 감싸야 함');
  assert.match(catchBody, /this\.redoStack\.pop\(\)/, 'catch 에서 오염 엔트리 pop');
  assert.match(catchBody, /discard\?\.\(wasm\)/, 'catch 에서 discard');
  assert.match(catchBody, /throw/, 'catch 에서 rethrow');
});

test('[결함3] execute 는 operation·after-save 어느 throw 에도 스냅샷을 누수하지 않는다', () => {
  const block = methodBlock(command, 'execute(wasm: WasmBridge): DocumentPosition {');
  assert.match(block, /this\.beforeId = wasm\.saveSnapshot\(\);/, 'before 저장이 있어야 함');
  // try 는 operation 과 after-save 를 모두 감싸야 한다 — after-save(대용량 클론
  // 메모리 압박 등) throw 도 before 누수 → orphan 이므로 대칭 보호 필수.
  assert.match(block, /try\s*\{[\s\S]*this\.operation\(wasm\)[\s\S]*this\.afterId = wasm\.saveSnapshot\(\)[\s\S]*\}\s*catch[\s\S]*throw/,
    'operation 과 after-save 를 함께 try 로 감싸야 함(after-save 가 try 밖이면 누수)');
  // catch 는 before/after 를 해제해야 한다(discard() 는 둘 다 null-safe 처리).
  assert.match(block, /catch[\s\S]*this\.discard\(wasm\)[\s\S]*throw/,
    'catch 에서 discard(wasm)로 before/after 대칭 해제 후 rethrow 해야 함');
  // after-save 가 try 밖(구 구조)이면 실패해야 한다 — catch 다음에 saveSnapshot 금지.
  assert.ok(!/\}\s*catch[\s\S]*?throw;\s*\}\s*this\.afterId = wasm\.saveSnapshot/.test(block),
    'after-save 가 catch 밖(try 이후)에 남아있음 — 누수 경로');
});

test('[결함1] 스냅샷 예산은 WASM 상한에서 순간 +2 여유를 뺀 값이다', () => {
  // 새 SnapshotCommand.execute 는 before/after 2개를 예산 강제 이전에 저장하므로,
  // 예산 == MAX 면 그 순간 store 가 MAX 초과 → WASM 무통보 축출 → orphan.
  // 예산 = MAX - 2 여야 순간 +2 가 MAX 를 넘지 않는다(인터리브 회귀 근절).
  assert.match(history, /const WASM_MAX_SNAPSHOTS = 100;/,
    'WASM MAX_SNAPSHOTS(document.rs) 미러 상수가 있어야 함');
  assert.match(history, /const SNAPSHOT_ID_BUDGET = WASM_MAX_SNAPSHOTS - 2;/,
    '예산은 MAX - 2 (순간 +2 여유) 여야 함 — MAX 와 같으면 orphan 회귀');
  // 예산 강제 헬퍼: 예산 초과 시 undo 스택 front 를 shift + discard.
  const block = methodBlock(history, 'enforceSnapshotBudget(wasm: WasmBridge): void {');
  assert.match(block, /liveSnapshotIds\(\)\s*>\s*SNAPSHOT_ID_BUDGET/, '예산 초과 판정');
  assert.match(block, /this\.undoStack\.length\s*>\s*1/, 'front 축출은 최소 1개 보존(length>1) 가드');
  assert.match(block, /this\.undoStack\.shift\(\)/, 'front 축출(shift)');
  assert.match(block, /discard\?\.\(wasm\)/, '축출 시 스냅샷 discard');
  // liveSnapshotIds 는 undo·redo 양 스택을 모두 세야 한다(순간 저장이 redo id 와
  // 합산돼 store 를 넘길 수 있으므로 — 한 스택만 세면 과소집계 → orphan 회귀).
  const live = methodBlock(history, 'liveSnapshotIds(): number {');
  assert.match(live, /this\.undoStack/, 'undoStack 합산');
  assert.match(live, /this\.redoStack/, 'redoStack 합산(누락 시 과소집계)');
  // execute 는 push·maxSize 축출 이후에 예산을 강제해야 방금 명령의 +2 가 반영된다.
  const exec = methodBlock(history, 'execute(command: EditCommand, wasm: WasmBridge): DocumentPosition {');
  const idxPush = exec.indexOf('this.undoStack.push(command)');
  const idxEnforce = exec.indexOf('this.enforceSnapshotBudget(wasm)');
  assert.ok(idxPush !== -1 && idxEnforce !== -1 && idxPush < idxEnforce,
    'execute 가 push 이후에 enforceSnapshotBudget 를 호출해야 함(전이면 +2 미반영)');
});

test('SnapshotCommand 는 점유 스냅샷 id 수를 보고한다(예산 계산용)', () => {
  const block = methodBlock(command, 'snapshotResourceCount(): number {');
  assert.match(block, /beforeId !== null[\s\S]*afterId !== null/, 'before/after 살아있는 id 수 반환');
});
