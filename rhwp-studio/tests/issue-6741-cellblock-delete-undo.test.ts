import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { codeOnly, functionBodyFrom } from './support/source-guard.ts';

// [#6741] F5 셀 블록에서 Delete / 되돌리기의 동작을 한컴에 맞춘다.
//
// 한글 2024 실측(`samples/table-001.hwp`, 화면 캡처 판정):
//
//  | 단계                 | 글자 수 | 셀 블록 |
//  |----------------------|---------|---------|
//  | 블록 선택            | 118     | 음영    |
//  | Delete               | 62      | 유지    |
//  | Ctrl+Z (undo)        | 118     | 복원    |
//  | Ctrl+Shift+Z (redo)  | 89      | 해제    |
//
// 종전 rhwp 는 방향키·Escape 외의 키를 "그 외 키"로 흘려 블록을 먼저 해제했으므로
// Delete 는 캐럿에서 한 글자만 지웠고, undo 시점에는 되살릴 선택이 남지 않았다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const source = (p: string) => readFileSync(join(rootDir, p), 'utf8');

/**
 * 셀 선택 모드 분기 본문만 잘라낸다.
 *
 * `exitCellSelectionMode()` 는 Escape 분기와 "그 외 키" 폴백 양쪽에 있으므로 첫 매치로
 * 순서를 재면 Escape 쪽에 걸린다. 폴백은 이 분기의 **마지막** 호출이므로 분기 끝을
 * 명시적으로 잘라 마지막 것을 본다.
 */
function cellSelectionBranch(kb: string): string {
  const at = kb.indexOf('if (this.cursor.isInCellSelectionMode()) {');
  assert.notEqual(at, -1, '셀 선택 모드 분기를 찾지 못했다');
  const end = kb.indexOf('handleNavigationShortcut.call(this, e)', at);
  assert.notEqual(end, -1, '분기 끝(내비게이션 단축키 처리)을 찾지 못했다');
  return kb.slice(at, end);
}

test('셀 블록 모드에서 Delete 는 블록을 유지한 채 선택 칸 내용을 지운다', () => {
  const block = cellSelectionBranch(codeOnly(source('src/engine/input-handler-keyboard.ts')));

  const deleteAt = block.indexOf("e.key === 'Delete'");
  const fallthroughAt = block.lastIndexOf('this.cursor.exitCellSelectionMode();');
  assert.notEqual(deleteAt, -1, 'Delete 처리가 없다 — 블록이 해제되고 한 글자만 지워진다');
  assert.ok(
    deleteAt < fallthroughAt,
    'Delete 처리가 "그 외 키" 해제보다 뒤에 있다 — 블록이 먼저 풀린다',
  );
  assert.match(
    block.slice(deleteAt, fallthroughAt),
    /this\.clearSelectedCellBlock\(\)/,
    'Delete 가 셀 블록 내용 지우기를 호출하지 않는다',
  );
});

test('되돌리기·다시실행은 셀 블록을 해제하지 않고 통과한다', () => {
  const kb = codeOnly(source('src/engine/input-handler-keyboard.ts'));
  assert.match(
    kb,
    /const CELL_BLOCK_GLOBAL_COMMANDS = new Set\(\[\s*'edit:undo',\s*'edit:redo',\s*\]\)/,
    '셀 블록에서 유지 통과할 명령 집합이 없다',
  );
  const block = cellSelectionBranch(kb);
  const passAt = block.indexOf('dispatchCellBlockGlobalShortcut.call(this, e)');
  const fallthroughAt = block.lastIndexOf('this.cursor.exitCellSelectionMode();');
  assert.notEqual(passAt, -1, '되돌리기 통과 배선이 없다');
  assert.ok(passAt < fallthroughAt, '통과 배선이 블록 해제보다 뒤에 있다');
});

test('셀 블록 내용 지우기는 스냅샷으로 기록하고 지우기 전 블록을 함께 싣는다', () => {
  const ih = codeOnly(source('src/engine/input-handler.ts'));
  const fn = functionBodyFrom(ih, 'private clearSelectedCellBlock()');

  assert.match(fn, /kind: 'snapshot'/,
    '내용 지우기는 문단·서식·인라인 컨트롤까지 지우므로 스냅샷이어야 한다(#3230 분류)');
  assert.match(fn, /this\.cursor\.captureCellSelection\(\)/, '지우기 전 셀 블록을 캡처하지 않는다');
  assert.match(fn, /selectionBefore:[\s\S]*mode: 'cellBlock'/,
    '캡처한 셀 블록을 selectionBefore 로 싣지 않는다 — undo 가 되살릴 근거가 없다');
  // [#2370] 이미 빈 칸만 골랐으면 문서가 그대로다 → 유령 undo 엔트리를 막는다.
  assert.match(fn, /if \(!changed\) return null;/,
    '무변경 시 null 을 돌려 기록을 취소해야 한다');
});

test('비운 칸 안에 있던 캐럿은 그 칸의 시작으로 내린다', () => {
  // `cursor.moveTo` 는 클램프하지 않는다(position 을 그대로 대입). 지우기 전 오프셋을
  // 사후 커서로 돌려주면 빈 칸에 범위 밖 위치가 남고, 이후 편집이 문단 길이 검사에 걸린다.
  // 문단이 여럿이던 칸은 cellParaIndex 도 범위 밖이 된다.
  const ih = codeOnly(source('src/engine/input-handler.ts'));
  const fn = functionBodyFrom(ih, 'private clearSelectedCellBlock()');

  assert.match(fn, /block\.cellIndices\.includes\(cursorBefore\.cellIndex\)/,
    '캐럿이 비운 칸 안이었는지 판정하지 않는다');
  assert.match(fn, /\{ \.\.\.cursorBefore, cellParaIndex: 0, charOffset: 0 \}/,
    '비운 칸 안 캐럿을 칸 시작으로 내리지 않는다 — charOffset 만 내리면 다문단 칸에서 여전히 범위 밖');
});

test('undo 는 셀 블록을 되살리고 redo 는 되살리지 않는다 — 한컴 실측', () => {
  const ih = codeOnly(source('src/engine/input-handler.ts'));

  const undoFn = functionBodyFrom(ih, 'private restoreSelectionAfterUndo(');
  assert.match(undoFn, /range\.mode === 'cellBlock'/, 'undo 복원에 셀 블록 분기가 없다');
  assert.match(undoFn, /this\.cursor\.restoreCellSelection\(range\.state\)/,
    'undo 가 캡처한 셀 블록으로 복원하지 않는다');

  // 한컴은 redo 에서 선택을 해제한다(#3416 본문 선택과 같은 규약) — 되살리면 안 된다.
  const redoFn = functionBodyFrom(ih, 'private restoreSelectionAfterRedo(');
  assert.doesNotMatch(redoFn, /cellBlock/,
    'redo 가 셀 블록을 되살린다 — 한컴은 redo 에서 해제한다');
});

test('셀 블록 복원은 표가 줄었으면 거절한다', () => {
  const cursor = codeOnly(source('src/engine/cursor.ts'));
  const fn = functionBodyFrom(cursor, 'restoreCellSelection(');

  // 복원 시점의 표에서 크기를 다시 읽어야 그 사이 행·열이 바뀐 경우를 걸러낸다.
  assert.match(fn, /getTableDimensions/, '복원 시점 표 크기를 다시 읽지 않는다');
  assert.match(fn, /if \(!inside\(state\.anchor\) \|\| !inside\(state\.focus\)\) return false;/,
    '범위 밖이면 거절해야 한다 — 유령 범위 금지(#2339)');

  // 캡처는 rowCount/colCount 를 담지 않는다(담으면 낡은 값으로 판정하게 된다).
  const cap = functionBodyFrom(cursor, 'captureCellSelection()');
  assert.doesNotMatch(cap, /rowCount/, '캡처가 표 크기를 담고 있다 — 복원 시점에 다시 읽어야 한다');
});
