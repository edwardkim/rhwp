import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createServer } from 'vite';
import { functionBodyFrom } from './support/source-guard.ts';

// [#5690] F3 블록 선택 삭제를 undo 하면 **확장 단계까지** 되돌아와야 한다.
//
// 한컴 오피스 2024 실측(COM, `HAction.Run("Select")` = F3):
//
//  | 단계                | 캐럿 | 선택            |
//  |---------------------|------|-----------------|
//  | F3 x2 (단어 단계)   | 27   | 16~27           |
//  | 삭제 후             | 16   | 없음            |
//  | Undo 후             | 27   | 16~27 복원      |
//  | **Undo 후 F3 한 번**| 44   | **16~44 문단**  |
//  | Redo 후             | 16   | 없음            |
//
// 마지막 줄이 판정이다 — 단어에서 **문단으로 이어서** 확장했다. 단계가 초기화됐다면 다시
// 단어 범위(16~27)에 머물렀을 것이다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const src = (rel: string): string => readFileSync(join(rootDir, rel), 'utf8');

test('[#5690] 삭제 커맨드가 블록 확장 단계를 함께 들고 있다', async () => {
  const vite = await createServer({
    root: rootDir, appType: 'custom', logLevel: 'silent', server: { middlewareMode: true },
  });
  try {
    const { DeleteSelectionCommand } = await vite.ssrLoadModule('/src/engine/command.ts');
    const start = { sectionIndex: 0, paragraphIndex: 0, charOffset: 2 };
    const end = { sectionIndex: 0, paragraphIndex: 0, charOffset: 12 };

    const block: any = new DeleteSelectionCommand(start, end, 1);
    assert.equal(block.selectionBefore().blockPhase, 1, 'F3 블록은 단계를 보관한다');

    // 드래그 선택은 블록이 아니다. 0 으로 적으면 되살릴 때 블록 모드로 부활한다.
    const drag: any = new DeleteSelectionCommand(start, end);
    assert.equal(drag.selectionBefore().blockPhase, null, '블록이 아니면 null');
  } finally {
    await vite.close();
  }
});

test('[#5690] 호출부가 커서에서 단계를 읽어 커맨드에 넘긴다', () => {
  const handler = src('src/engine/input-handler.ts');
  const del = functionBodyFrom(handler, 'private deleteSelection()');
  assert.match(del, /new DeleteSelectionCommand\(sel\.start, sel\.end, this\.cursor\.blockSelectionPhase\(\)\)/,
    '삭제 시점의 블록 단계를 기록해야 한다');

  const restore = functionBodyFrom(handler, 'private restoreSelectionAfterUndo');
  assert.match(restore, /selectRange\(range\.start, range\.end, range\.blockPhase\)/,
    '되살릴 때 범위와 단계를 같은 호출로 넘겨야 한다');
});

test('[#5690] selectRange 가 범위와 블록 상태를 한 번에 세운다', async () => {
  const vite = await createServer({
    root: rootDir, appType: 'custom', logLevel: 'silent', server: { middlewareMode: true },
  });
  try {
    const { CursorState } = await vite.ssrLoadModule('/src/engine/cursor.ts');
    const ctx = () => Object.assign(Object.create(CursorState.prototype), {
      anchor: null, position: null, _blockSelectionMode: false, _expandPhase: 0,
      wasm: { getParagraphCount: () => 3, getParagraphLength: () => 20 },
      updateRect() { /* 기하 갱신은 이 판정과 무관 */ },
    });
    const p = (off: number) => ({ sectionIndex: 0, paragraphIndex: 0, charOffset: off });

    const blk = ctx();
    assert.equal(CursorState.prototype.selectRange.call(blk, p(2), p(12), 2), true);
    assert.equal(blk._blockSelectionMode, true, '단계를 주면 블록 모드로 선다');
    assert.equal(blk._expandPhase, 2, '그 단계 그대로 — 다음 F3 가 이어서 확장한다');
    assert.equal(CursorState.prototype.blockSelectionPhase.call(blk), 2);

    const drag = ctx();
    assert.equal(CursorState.prototype.selectRange.call(drag, p(2), p(12)), true);
    assert.equal(drag._blockSelectionMode, false, 'null 이면 블록 모드가 아니다');
    assert.equal(CursorState.prototype.blockSelectionPhase.call(drag), null);

    // 0 단계(F5 진입 직후)와 "블록 아님"은 다르다.
    const zero = ctx();
    CursorState.prototype.selectRange.call(zero, p(2), p(12), 0);
    assert.equal(CursorState.prototype.blockSelectionPhase.call(zero), 0, '0 과 null 을 구분한다');

    // 거절되는 범위는 블록 상태도 남기지 않는다 — 반쯤 선 상태가 곧 유령 범위다.
    const bad = ctx();
    assert.equal(CursorState.prototype.selectRange.call(bad, p(2), p(99), 3), false);
    assert.equal(bad._blockSelectionMode, false, '거절이면 블록 모드도 세우지 않는다');
    assert.equal(bad._expandPhase, 0);
    assert.equal(bad.anchor, null);
  } finally {
    await vite.close();
  }
});

test('[#5690] #2339 의 해제는 그대로 남는다 — 복원이 그 위에 얹힌다', () => {
  const handler = src('src/engine/input-handler.ts');
  const reset = functionBodyFrom(handler, 'private resetDerivedStateAfterHistoryJump');
  assert.match(reset, /exitBlockSelectionMode\(\)/, '해제는 유지되어야 한다');

  const undo = functionBodyFrom(handler, 'private handleUndo()');
  const redo = functionBodyFrom(handler, 'private handleRedo()');
  const idxReset = undo.indexOf('resetDerivedStateAfterHistoryJump');
  const idxRestore = undo.indexOf('restoreSelectionAfterUndo');
  assert.ok(idxReset !== -1 && idxRestore !== -1 && idxReset < idxRestore, '해제 뒤에 복원한다');
  assert.doesNotMatch(redo, /restoreSelectionAfterUndo/, '한컴은 redo 에서 해제한다');
});
