import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { codeOnly } from './support/source-guard.ts';

// [Task #2328] 스냅샷 상한 정합 + 예외 안전 스택 이동 소스 가드.
//
// node --test 는 strip-only TS 라 engine 클래스(parameter property 포함)를
// 실행할 수 없어, 이 저장소의 undo 테스트 관례대로 소스 배선을 검증한다.
// 행위 증명은 브라우저 실동작(수정 전/후 60회 스냅샷 + 오래된 undo 무예외)으로
// 별도 수행한다 (PR 검증 섹션).

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const source = (rel: string): string => codeOnly(readFileSync(join(rootDir, rel), 'utf8'));

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

test('[결함3] execute 는 operation throw 에 스냅샷을 누수하지 않는다', () => {
  const block = methodBlock(command, 'execute(wasm: WasmBridge): DocumentPosition {');
  assert.match(block, /this\.beforeId = wasm\.saveSnapshot\(\);/, 'before 저장이 있어야 함');
  // [Task #5769] after 저장은 undo 로 옮겼다. execute 에 남은 유일한 throw 원천은
  // operation 이고, 그것이 try 안에 있어야 before 누수(orphan)를 막는다.
  assert.match(block, /try\s*\{[\s\S]*this\.operation\(wasm\)[\s\S]*\}\s*catch[\s\S]*throw/,
    'operation 을 try 로 감싸야 함');
  assert.match(block, /catch[\s\S]*this\.discard\(wasm\)[\s\S]*throw/,
    'catch 에서 discard(wasm)로 해제 후 rethrow 해야 함');
  // execute 안에 after 저장이 남아 있으면 지연 저장이 무력화된다(엔트리가 다시 2슬롯).
  assert.doesNotMatch(block, /this\.afterId = wasm\.saveSnapshot\(\)/,
    '[#5769] execute 는 after 를 저장하지 않는다 — undo 시점에 잡는다');
});

test('[#5769] after 는 undo 에서 잡고, 저장 실패가 되돌리기를 막지 않는다', () => {
  const block = methodBlock(command, 'undo(wasm: WasmBridge): DocumentPosition {');
  const idxSave = block.indexOf('this.afterId = wasm.saveSnapshot()');
  const idxRestore = block.indexOf('wasm.restoreSnapshot(this.beforeId)');
  assert.ok(idxSave !== -1, 'undo 가 after 를 잡아야 redo 가 가능하다');
  assert.ok(idxRestore !== -1 && idxSave < idxRestore,
    'after 저장이 before 복원보다 먼저여야 한다 — 복원 뒤면 before 상태를 after 로 찍는다');
  assert.match(block, /catch\s*\{[\s\S]*this\.redoUnavailable = true;/,
    '저장 실패는 redo 만 포기하고 undo 는 계속해야 한다');
  const catchEnd = block.indexOf('redoUnavailable = true');
  assert.ok(block.indexOf('wasm.restoreSnapshot(this.beforeId)') > catchEnd,
    '저장 실패 뒤에도 복원에 도달해야 한다(던지면 Ctrl+Z 가 먹통이 된다)');
});

test('[#5769] redo 는 after 로 복원한 뒤 그 스냅샷을 즉시 반환한다', () => {
  const block = methodBlock(command, 'execute(wasm: WasmBridge): DocumentPosition {');
  const redoArm = block.slice(0, block.indexOf('this.executed = true'));
  assert.match(redoArm, /if \(this\.executed\)/,
    'redo 판별은 executed 플래그여야 한다 — afterId 로는 실행 직후와 구분되지 않는다');
  const idxRestore = redoArm.indexOf('wasm.restoreSnapshot(this.afterId)');
  const idxDiscard = redoArm.indexOf('wasm.discardSnapshot(this.afterId)');
  assert.ok(idxRestore !== -1 && idxDiscard !== -1 && idxRestore < idxDiscard,
    '복원 후 반환해야 undo 스택으로 돌아간 엔트리가 1슬롯을 유지한다');
  assert.match(redoArm, /throw new Error/,
    'after 가 없으면 성공한 척하지 말고 던져야 한다(히스토리가 드롭한다)');
});

test('[#3350] 최초 execute 실패는 before 스냅샷 복원 후 ID를 해제한다', () => {
  const block = methodBlock(command, 'execute(wasm: WasmBridge): DocumentPosition {');
  const catchStart = block.search(/\}\s*catch \(operationError\)/);
  assert.notEqual(catchStart, -1, '최초 execute 실패 catch가 있어야 함');
  const catchBody = block.slice(catchStart);

  const idxRestore = catchBody.indexOf('wasm.restoreSnapshot(this.beforeId)');
  assert.notEqual(idxRestore, -1, '부분 변경을 before 스냅샷으로 rollback해야 함');
  assert.match(catchBody, /catch \(rollbackError\)/,
    'rollback 실패도 별도로 포착해야 함');
  assert.match(catchBody, /new AggregateError\(\s*\[operationError, rollbackError\]/,
    '원래 operation 오류와 rollback 오류를 함께 보존해야 함');

  // [#3662] rollback **성공** 경로의 discard 를 따로 못박는다.
  //
  // 종전에는 `catchBody.indexOf('this.discard(wasm)')` 로 순서만 봤는데, 그 첫 번째 일치는
  // inner `catch (rollbackError)` 안의 discard 다. 그래서 성공 경로의 discard 를 지워도
  // 가드가 통과했다 — 스냅샷 2개가 조용히 누수되는 회귀를 못 잡는다.
  //
  // inner catch 가 닫힌 **뒤** discard → rethrow 가 이어지는지 확인해 성공 경로에 고정한다.
  assert.match(
    catchBody,
    /\}\s*this\.discard\(wasm\);\s*throw operationError;/,
    'rollback 성공 시 before/after 를 해제한 뒤 원래 operation 오류를 전파해야 함',
  );

  // inner catch 안의 discard 도 함께 남아 있어야 한다 — rollback 이 실패해도 ID 는 해제한다.
  const rollbackCatch = catchBody.slice(catchBody.indexOf('catch (rollbackError)'));
  const innerBody = rollbackCatch.slice(0, rollbackCatch.indexOf('throw new AggregateError'));
  assert.match(innerBody, /this\.discard\(wasm\)/,
    'rollback 실패 경로도 스냅샷 ID 를 해제해야 함');
});

test('[결함1] 스냅샷 예산은 WASM 상한에서 순간 여유를 뺀 값이다', () => {
  // 예산 == MAX 면 예산 강제 이전의 순간 저장이 store 를 MAX 초과로 밀어 WASM 무통보
  // 축출 → orphan 이 된다. 예산 = MAX - 2 여야 그 순간이 MAX 를 넘지 않는다.
  // [Task #5769] after 지연 저장 이후 execute 의 순간 저장은 before 하나(+1)이고 undo 의
  // 순간 저장도 +1 이라 여유 2 는 그대로 충분하다 — 상수는 Rust MAX_SNAPSHOTS 와
  // 양방향 결합이므로 여유가 남는다고 좁히지 않는다.
  assert.match(history, /^const WASM_MAX_SNAPSHOTS = 100;/m,
    'WASM MAX_SNAPSHOTS(document.rs) 미러 상수가 있어야 함');
  assert.match(history, /^const SNAPSHOT_ID_BUDGET = WASM_MAX_SNAPSHOTS - 2;/m,
    '예산은 MAX - 2 (순간 +2 여유) 여야 함 — MAX 와 같으면 orphan 회귀');
  // [#6332] 상수 결합의 studio 레인 절반 — 리터럴 pin 만으로는 결합 자체가 검증되지
  // 않아, Rust store 상한이 studio 피크 동시 참조 수를 항상 덮는지 document.rs 를
  // 직접 읽어 기계 검증한다. 피크는 예산(W-2) + 순간 저장 1 = W-1 이다 — #5769 이후
  // before/after 저장은 서로 다른 시점이고 각 저장 사이에 예산 강제가 돈다(위 주석과
  // 동일 사실). 가드는 미러 관습대로 동치 이상(MAX >= W, 여유 1)을 요구한다.
  // rust 레인 절반(순 Rust 변경은 이 파일이 안 돎)은
  // tests/cases/issue_6332_snapshot_budget_coupling.rs 가 담당한다.
  // 선언 앵커는 줄 시작(^…/m) — 주석 속 인용의 첫-매치 오염을 막는다.
  const documentRs = source('../src/document_core/commands/document.rs');
  const rustMax = Number(/^\s*const MAX_SNAPSHOTS: usize = (\d+);/m.exec(documentRs)?.[1]);
  const wasmMax = Number(/^const WASM_MAX_SNAPSHOTS = (\d+);/m.exec(history)?.[1]);
  assert.ok(Number.isInteger(rustMax),
    'document.rs 의 MAX_SNAPSHOTS 선언 줄을 찾지 못함 — 선언 형태가 바뀌었으면 이 가드를 갱신');
  assert.ok(Number.isInteger(wasmMax), 'history.ts 의 WASM_MAX_SNAPSHOTS 선언 줄을 찾지 못함');
  assert.ok(rustMax >= wasmMax,
    `Rust MAX_SNAPSHOTS(${rustMax}) < studio WASM_MAX_SNAPSHOTS(${wasmMax}) — 피크 동시 참조(${wasmMax}-1)를 덮지 못해 참조 중 스냅샷이 무통보 축출된다 (#2328/#6332)`);
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
    'execute 가 push 이후에 enforceSnapshotBudget 를 호출해야 함(전이면 미반영)');
});

test('[#6332] recordWithoutExecute 도 스냅샷 예산을 강제한다', () => {
  // 예산 불변식의 undoStack push 진입점은 execute 와 recordWithoutExecute 둘이다.
  // execute 쪽 강제는 위 [결함1] 이 고정한다 — 이 테스트는 record 경로가 미보호로
  // 남지 않게 고정한다. 현재 record 커맨드는 스냅샷 0개라 no-op 방어지만, 스냅샷
  // 보유 커맨드가 이 경로에 추가되면 강제 없이는 무통보 축출이 재발한다(#2328).
  const block = methodBlock(history, 'recordWithoutExecute(command: EditCommand, wasm?: WasmBridge): void {');
  const idxPush = block.indexOf('this.undoStack.push(command)');
  const idxEnforce = block.indexOf('this.enforceSnapshotBudget(wasm)');
  assert.ok(idxPush !== -1, 'push 지점을 찾지 못함 — 시그니처가 바뀌었으면 가드 갱신');
  assert.ok(idxEnforce !== -1 && idxPush < idxEnforce,
    'recordWithoutExecute 가 push 이후에 enforceSnapshotBudget 를 호출해야 함');
});

test('[#5769] undo 도 스냅샷 id 를 늘리므로 undo 경로에서 예산을 강제한다', () => {
  // after 지연 저장 이후 undo 는 엔트리를 1슬롯 → 2슬롯으로 바꾼다. execute 에서만
  // 강제하면 예산을 채운 뒤 연속 undo 할 때 store 가 MAX 를 넘어 #2328 무통보 축출이
  // 되살아난다.
  const undoBlock = methodBlock(history, 'undo(wasm: WasmBridge): DocumentPosition | null {');
  assert.match(undoBlock, /this\.enforceSnapshotBudgetAfterUndo\(wasm\)/,
    'undo 도 예산을 강제해야 한다');
  const idxPush = undoBlock.indexOf('this.redoStack.push(command)');
  const idxEnforce = undoBlock.indexOf('enforceSnapshotBudgetAfterUndo');
  assert.ok(idxPush !== -1 && idxEnforce !== -1 && idxPush < idxEnforce,
    '스택 이동 이후에 강제해야 방금 늘어난 +1 이 반영된다');

  // 축출은 redo 스택 bottom 부터 — undo 중인 사용자가 지키려는 것은 과거다.
  const block = methodBlock(history, 'enforceSnapshotBudgetAfterUndo(wasm: WasmBridge): void {');
  assert.match(block, /this\.redoStack\.shift\(\)/, 'redo bottom(가장 먼 미래)부터 축출');
  assert.match(block, /this\.redoStack\.length\s*>\s*1/, '최소 1개 보존 가드');
  assert.match(block, /this\.enforceSnapshotBudget\(wasm\)/, 'redo 로 부족하면 종전 규칙');
});

test('SnapshotCommand 는 점유 스냅샷 id 수를 보고한다(예산 계산용)', () => {
  const block = methodBlock(command, 'snapshotResourceCount(): number {');
  assert.match(block, /beforeId !== null[\s\S]*afterId !== null/, 'before/after 살아있는 id 수 반환');
});

test('document-agent 미commit rollback은 exact 최신 snapshot만 폐기하고 redo에 남기지 않는다', () => {
  const block = methodBlock(
    history,
    'rollbackUncommittedSnapshot(',
  );
  assert.match(block, /command\.type !== expectedType/, '다른 최신 명령을 되돌리면 안 됨');
  assert.match(block, /snapshotResourceCount\?\.\(\) !== 1/,
    '[#5769] 최초 실행 직후는 before 하나만 점유 — 상수 2 를 두면 이 경로가 죽는다');
  const undoAt = block.indexOf('command.undo(wasm)');
  const popAt = block.indexOf('this.undoStack.pop()');
  const discardAt = block.indexOf('command.discard?.(wasm)');
  assert.ok(undoAt !== -1 && undoAt < popAt && popAt < discardAt,
    'restore → undo stack 제거 → snapshot 폐기 순서여야 함');
  assert.doesNotMatch(block, /redoStack\.push/, '실패한 미commit 명령은 redo에 남기면 안 됨');
});
