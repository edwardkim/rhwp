import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [Task #2339] history-jumped 이벤트 소스 가드.
//
// undo/redo(히스토리 점프)가 위치 기반 파생 상태를 일괄 무효화하는지 정적으로 검증한다.
// node --test 는 strip-only TS 라 engine 클래스를 실행할 수 없어(이 저장소 undo 테스트
// 관례) 소스 배선을 핀한다. 행위 증명은 브라우저 실동작으로 별도 수행(PR 검증 섹션).

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const source = (rel: string): string => readFileSync(join(rootDir, rel), 'utf8');

const inputHandler = source('src/engine/input-handler.ts');
const findDialog = source('src/ui/find-dialog.ts');

/** 메서드 본문(시그니처 ~ 다음 최상위 메서드 전)을 추출. */
function methodBlock(src: string, signature: string): string {
  const start = src.indexOf(signature);
  assert.notEqual(start, -1, `${signature} not found`);
  const rel = src.slice(start + signature.length)
    .search(/\n {2}(?:public |private |protected |static |async |get |set )*[a-zA-Z][\w]*\s*\(/);
  return rel === -1 ? src.slice(start) : src.slice(start, start + signature.length + rel);
}

test('handleUndo/handleRedo 는 resetDerivedStateAfterHistoryJump 로 파생 상태를 무효화한다', () => {
  const undo = methodBlock(inputHandler, 'private handleUndo()');
  const redo = methodBlock(inputHandler, 'private handleRedo()');
  assert.match(undo, /this\.resetDerivedStateAfterHistoryJump\(\)/, 'handleUndo 가 호출해야 함');
  assert.match(redo, /this\.resetDerivedStateAfterHistoryJump\(\)/, 'handleRedo 가 호출해야 함');
  // 구 헬퍼명이 잔존하면(부분 이관) 실패.
  assert.doesNotMatch(inputHandler, /exitObjectSelectionAfterHistoryJump/,
    '구 메서드명이 남아있음 — resetDerivedStateAfterHistoryJump 로 일반화됐어야 함');
});

test('resetDerivedStateAfterHistoryJump 는 3종 파생 상태 해제 + history-jumped emit 한다', () => {
  const block = methodBlock(inputHandler, 'private resetDerivedStateAfterHistoryJump()');
  // 개체/표 선택(#2303) 해제 유지.
  assert.match(block, /exitPictureObjectSelection\(\)/, '개체 선택 해제(회귀 방지)');
  assert.match(block, /exitTableObjectSelection\(\)/, '표 선택 해제(회귀 방지)');
  // [#2339] 텍스트 선택 + 본문 블록 선택 단계 + 셀 블록 선택(+렌더러) 해제.
  // exitBlockSelectionMode 는 _blockSelectionMode/_expandPhase 를 초기화하고 내부에서
  // clearSelection 도 수행하므로, 선택만 지우던 clearSelection 을 대체한다(stale F3/F5 단계 방지).
  assert.match(block, /this\.cursor\.exitBlockSelectionMode\(\)/,
    '텍스트 선택 + 본문 블록 선택 단계 해제(유령 범위·stale 확장 단계 방지)');
  assert.match(block, /this\.cursor\.exitCellSelectionMode\(\)/, 'F5 셀 블록 선택 해제');
  assert.match(block, /this\.cellSelectionRenderer\?\.clear\(\)/,
    'F5 셀 하이라이트 렌더러 clear(고스트 오버레이 제거)');
  // 외부 구독자용 이벤트 emit.
  assert.match(block, /this\.eventBus\.emit\('history-jumped'\)/, 'history-jumped emit(확장점)');
});

test("emit('history-jumped') 은 커서 이동(moveTo) 앞에 위치한다", () => {
  // 구독자가 복원된 문서를 보고 정리하도록, 파생 상태 리셋은 커서 재배치보다 먼저 일어나야
  // 한다. handleUndo 에서 reset 호출이 moveTo 보다 앞인지 확인(#2337 병합 시 restoreEditContext
  // 로 바뀌어도 상대 순서 불변).
  const undo = methodBlock(inputHandler, 'private handleUndo()');
  const idxReset = undo.indexOf('resetDerivedStateAfterHistoryJump');
  const idxCursor = undo.search(/this\.cursor\.moveTo\(newPos\)|restoreEditContextAfterHistory/);
  assert.ok(idxReset !== -1, 'reset 호출 존재');
  assert.ok(idxCursor === -1 || idxReset < idxCursor, 'reset 은 커서 재배치보다 앞이어야 함');
});

test('find-dialog 는 history-jumped 를 구독해 currentHit 을 무효화하고, 닫을 때 해제한다', () => {
  const show = methodBlock(findDialog, 'show()');
  const hide = methodBlock(findDialog, 'hide()');
  // show 에서 구독 → currentHit = null.
  assert.match(show, /eventBus\.on\('history-jumped',\s*\(\)\s*=>\s*\{\s*this\.currentHit = null;?\s*\}\)/,
    'show 에서 history-jumped 구독 → currentHit 무효화');
  assert.match(show, /this\.historyJumpOff\s*=/, '해제 핸들 저장');
  // hide 에서 해제(리스너 누수 방지).
  assert.match(hide, /this\.historyJumpOff\?\.\(\)/, 'hide 에서 구독 해제 호출');
  assert.match(hide, /this\.historyJumpOff = null/, 'hide 에서 핸들 정리');
});
