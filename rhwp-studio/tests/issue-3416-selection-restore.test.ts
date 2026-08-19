import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createServer } from 'vite';
import { functionBodyFrom } from './support/source-guard.ts';

// [#3416] undo 뒤 "지우기 전 선택" 복원.
//
// 한컴 오피스 2024 실측(COM, `GetSelectedPosBySet`):
//
//  | 연산                | Undo 후 선택                          |
//  |---------------------|---------------------------------------|
//  | 선택 삭제(Delete)   | (0,0,18)~(0,0,22) — 지우기 전 범위 복원 |
//  | 선택 위 타이핑 대체 | 없음 — 캐럿만 선택 끝(0,0,19)          |
//  | 선택 서식(Bold)     | 애초에 유지(잃지 않음)                 |
//
//  Redo 는 선택을 해제한다(삭제 직후 상태).
//
// 그래서 복원은 **선택 삭제 계열만** 이고, 나머지는 #2339 의 해제가 그대로 최종 상태다.
// 그리고 복원 전에 현재 문서에서 유효한 범위인지 확인해야 한다 — #2339 가 해제를 넣은 이유가
// 유령 범위이기 때문이다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const src = (rel: string): string => readFileSync(join(rootDir, rel), 'utf8');

test('[#3416] 선택 삭제 커맨드가 지우기 전 범위를 들고 있다', async () => {
  const vite = await createServer({
    root: rootDir, appType: 'custom', logLevel: 'silent', server: { middlewareMode: true },
  });
  try {
    const { DeleteSelectionCommand } = await vite.ssrLoadModule('/src/engine/command.ts');
    const start = { sectionIndex: 0, paragraphIndex: 0, charOffset: 18 };
    const end = { sectionIndex: 0, paragraphIndex: 0, charOffset: 22 };
    const cmd: any = new DeleteSelectionCommand(start, end);

    assert.deepEqual(cmd.selectionBefore(), { start, end });

    // 호출부가 넘긴 객체를 그대로 들고 있으면 이후 변형이 기록을 오염시킨다.
    start.charOffset = 999;
    assert.equal(cmd.selectionBefore().start.charOffset, 18, '복사해서 보관해야 한다');
  } finally {
    await vite.close();
  }
});

test('[#3416] undo 경로가 선택을 되살린다 — redo 경로에는 없다', () => {
  const handler = src('src/engine/input-handler.ts');
  const undo = functionBodyFrom(handler, 'private handleUndo()');
  const redo = functionBodyFrom(handler, 'private handleRedo()');

  assert.match(undo, /this\.restoreSelectionAfterUndo\(/, 'undo 는 복원을 시도해야 한다');
  assert.doesNotMatch(redo, /restoreSelectionAfterUndo/, '한컴은 redo 에서 선택을 해제한다');

  // 해제가 먼저, 복원이 나중이어야 한다 — 순서가 뒤집히면 복원한 선택을 곧바로 지운다.
  const idxReset = undo.indexOf('resetDerivedStateAfterHistoryJump');
  const idxRestore = undo.indexOf('restoreSelectionAfterUndo');
  assert.ok(idxReset !== -1 && idxRestore !== -1 && idxReset < idxRestore,
    '해제 뒤에 복원해야 한다');
});

test('[#3416] 복원 전에 유효성을 확인한다 (#2339 유령 범위 방지)', () => {
  const handler = src('src/engine/input-handler.ts');
  const restore = functionBodyFrom(handler, 'private restoreSelectionAfterUndo');
  assert.match(restore, /isRestorableBodySelection\(/, '유효성 확인을 거쳐야 한다');
  // 확인보다 먼저 selectRange 를 부르면 유령 범위가 그대로 선다.
  const idxCheck = restore.indexOf('isRestorableBodySelection');
  const idxSelect = restore.indexOf('selectRange');
  assert.ok(idxCheck !== -1 && idxSelect !== -1 && idxCheck < idxSelect,
    '확인이 selectRange 보다 먼저여야 한다');
});

test('[#3416] 해제는 그대로 남는다 — 복원은 그 위에 얹는 향상이다', () => {
  const handler = src('src/engine/input-handler.ts');
  const reset = functionBodyFrom(handler, 'private resetDerivedStateAfterHistoryJump');
  assert.match(reset, /exitBlockSelectionMode\(\)/, '#2339 의 해제가 유지돼야 한다');
  assert.match(reset, /exitCellSelectionMode\(\)/);
  assert.match(reset, /exitPictureObjectSelection\(\)/);
});

test('[#3416] 범위 유효성 판정 — 실제 동작', async () => {
  const vite = await createServer({
    root: rootDir, appType: 'custom', logLevel: 'silent', server: { middlewareMode: true },
  });
  try {
    const mod = await vite.ssrLoadModule('/src/engine/input-handler.ts');
    const InputHandler: any = mod.InputHandler;
    // 판정은 wasm 조회에만 의존하므로 프로토타입 메서드를 최소 컨텍스트로 부른다.
    const check = InputHandler.prototype['isRestorableBodySelection'];
    assert.equal(typeof check, 'function', 'isRestorableBodySelection 이 있어야 함');

    const ctx = (paraCount: number, len: number) => ({
      wasm: {
        getParagraphCount: () => paraCount,
        getParagraphLength: () => len,
      },
    });
    const p = (para: number, off: number) => ({ sectionIndex: 0, paragraphIndex: para, charOffset: off });

    assert.equal(check.call(ctx(3, 10), p(0, 2), p(0, 6)), true, '문서 안 범위는 복원 가능');
    assert.equal(check.call(ctx(3, 10), p(0, 2), p(5, 6)), false, '문단이 사라졌으면 불가');
    assert.equal(check.call(ctx(3, 10), p(0, 2), p(0, 99)), false, '오프셋이 길이를 넘으면 불가');
    assert.equal(
      check.call(ctx(3, 10), p(0, 2), { sectionIndex: 1, paragraphIndex: 0, charOffset: 1 }),
      false,
      '구역이 다르면 불가',
    );
    // 셀 안 범위는 확인 축이 달라 복원하지 않는다.
    assert.equal(
      check.call(ctx(3, 10), { ...p(0, 2), parentParaIndex: 1 }, p(0, 6)),
      false,
      '셀 범위는 복원 대상이 아니다',
    );
    // 조회가 던지면 유효하다고 말할 수 없다.
    const throwing = { wasm: { getParagraphCount: () => { throw new Error('gone'); } } };
    assert.equal(check.call(throwing, p(0, 2), p(0, 6)), false, '조회 실패는 불가로 판정');
  } finally {
    await vite.close();
  }
});
