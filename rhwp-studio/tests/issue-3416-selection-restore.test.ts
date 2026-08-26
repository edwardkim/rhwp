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
// 그리고 실재하지 않는 범위는 서지 않아야 한다 — #2339 가 해제를 넣은 이유가 유령 범위이기
// 때문이다. 그 판정은 anchor/focus 소유자인 `CursorState.selectRange` 가 하고, 호출부는
// "어디까지 맞출지"(구역·커맨드 종류)만 정한다.

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

    assert.deepEqual(cmd.selectionBefore(), { start, end, blockPhase: null });

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

test('[#3416] 유효성은 호출부가 아니라 selectRange 안에서 판정된다', () => {
  const cursor = src('src/engine/cursor.ts');
  const select = functionBodyFrom(cursor, 'selectRange(');

  // 확인이 대입보다 먼저여야 한다 — 뒤면 유령 범위가 이미 선 뒤다.
  const idxCheck = select.indexOf('isVerifiedBodyPosition');
  const idxAssign = select.indexOf('this.anchor =');
  assert.ok(idxCheck !== -1 && idxAssign !== -1 && idxCheck < idxAssign,
    '두 끝을 확인한 뒤에 anchor/position 을 세워야 한다');
  assert.match(select, /return false/, '세우지 못하면 그 사실을 돌려줘야 한다');

  // 호출부에 같은 판정을 두면 계약이 두 곳으로 갈라진다 — 그때부터 한쪽만 고쳐진다.
  const handler = src('src/engine/input-handler.ts');
  const restore = functionBodyFrom(handler, 'private restoreSelectionAfterUndo');
  assert.doesNotMatch(restore, /getParagraphCount|getParagraphLength|parentParaIndex/,
    '실재 판정을 호출부가 되풀이하면 안 된다');
});

test('[#3416] 해제는 그대로 남는다 — 복원은 그 위에 얹는 향상이다', () => {
  const handler = src('src/engine/input-handler.ts');
  const reset = functionBodyFrom(handler, 'private resetDerivedStateAfterHistoryJump');
  assert.match(reset, /exitBlockSelectionMode\(\)/, '#2339 의 해제가 유지돼야 한다');
  assert.match(reset, /exitCellSelectionMode\(\)/);
  assert.match(reset, /exitPictureObjectSelection\(\)/);
});

test('[#3416] selectRange 는 실재하지 않는 범위를 거절한다 — 실제 동작', async () => {
  const vite = await createServer({
    root: rootDir, appType: 'custom', logLevel: 'silent', server: { middlewareMode: true },
  });
  try {
    const { CursorState } = await vite.ssrLoadModule('/src/engine/cursor.ts');
    const selectRange = CursorState.prototype.selectRange;

    // 판정은 wasm 조회에만 의존한다. 생성자는 렌더러까지 요구하므로 프로토타입 위에
    // 최소 상태만 얹어 부른다(내부 helper 도 프로토타입에서 찾아야 하므로 Object.create).
    const ctx = (paraCount: number, len: number) => Object.assign(
      Object.create(CursorState.prototype),
      {
        anchor: null, position: null,
        wasm: { getParagraphCount: () => paraCount, getParagraphLength: () => len },
        updateRect() { /* 기하 갱신은 이 판정과 무관 */ },
      },
    );
    const p = (para: number, off: number) => ({ sectionIndex: 0, paragraphIndex: para, charOffset: off });

    const ok = ctx(3, 10);
    assert.equal(selectRange.call(ok, p(0, 2), p(0, 6)), true, '문서 안 범위는 세운다');
    assert.deepEqual(ok.anchor, p(0, 2), 'anchor 는 start');
    assert.deepEqual(ok.position, p(0, 6), '커서는 end');

    // 거절할 때는 **아무것도 바꾸지 않아야** 한다 — 반쯤 세우면 그게 유령 범위다.
    for (const [why, start, end] of [
      ['문단이 사라졌으면', p(0, 2), p(5, 6)],
      ['오프셋이 길이를 넘으면', p(0, 2), p(0, 99)],
      ['셀 안 위치는', { ...p(0, 2), parentParaIndex: 1 }, p(0, 6)],
    ] as Array<[string, any, any]>) {
      const c = ctx(3, 10);
      assert.equal(selectRange.call(c, start, end), false, `${why} 거절한다`);
      assert.equal(c.anchor, null, `${why} anchor 를 건드리지 않는다`);
      assert.equal(c.position, null, `${why} 커서를 건드리지 않는다`);
    }

    // 조회가 던지면 실재한다고 말할 수 없다.
    const throwing = Object.assign(Object.create(CursorState.prototype), {
      anchor: null, position: null,
      wasm: { getParagraphCount: () => { throw new Error('gone'); } },
      updateRect() {},
    });
    assert.equal(selectRange.call(throwing, p(0, 2), p(0, 6)), false, '조회 실패는 거절');
    assert.equal(throwing.anchor, null, '조회 실패에도 상태를 건드리지 않는다');
  } finally {
    await vite.close();
  }
});

test('[#3416] 구역을 걸치는 범위는 복원 대상이 아니다 (실측 범위 밖)', () => {
  const handler = src('src/engine/input-handler.ts');
  const restore = functionBodyFrom(handler, 'private restoreSelectionAfterUndo');
  assert.match(restore, /sectionIndex !== range\.end\.sectionIndex/, '구역 정책은 호출부가 갖는다');
  const idxPolicy = restore.indexOf('sectionIndex !==');
  const idxSelect = restore.indexOf('selectRange');
  assert.ok(idxPolicy !== -1 && idxPolicy < idxSelect, '정책 판정이 먼저다');
});
