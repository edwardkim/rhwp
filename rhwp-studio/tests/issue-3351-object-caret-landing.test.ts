import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createServer } from 'vite';
import { balancedFrom, functionBodyFrom } from './support/source-guard.ts';

// [#3351] 메뉴 개체 조작의 undo/redo 캐럿이 조작과 무관한 자리로 착지하던 결함.
//
// 개체 선택(`enterPictureObjectSelectionDirect`)은 `cursor.position` 을 옮기지 않는다. 그래서
// 기록 시점에 `getCursorPosition()` 을 잡으면 **개체를 선택하기 직전 캐럿**이 남고, 문단 0 에서
// 스크롤해 문단 10 의 개체를 지우면 undo/redo 가 문서 상단으로 점프한다.
//
// 한컴 오피스 2024 실측(COM):
//  - `FindCtrl` 로 개체를 선택하면 캐럿이 앵커로 이동한다((0,0,16) → (0,10,5)).
//  - 그 뒤 캐럿을 다른 문단으로 옮기면 **개체 선택이 풀린다**(이후 Delete 가 개체를 못 지움).
//    즉 "캐럿은 딴 곳, 개체는 선택" 이라는 상태가 한컴에는 없다.
//  - 삭제 → Undo → Redo 내내 캐럿은 개체 인접 문단((0,10,0))에 머문다.
//
// 따라서 정답은 "개체 인접" 이고, Delete 키 경로(`performDelete`)가 이미 그렇게 한다.
// 여기서 고정하는 것은 메뉴 경로와 클릭 z순서 경로도 같은 자리를 기록한다는 것이다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const studioRoot = rootDir;
const src = (rel: string): string => readFileSync(join(rootDir, rel), 'utf8');

test('[#3351] 인접 위치 규칙은 한 곳에만 있다', () => {
  const cursor = src('src/engine/cursor.ts');

  // 규칙 본체는 ref 를 받는 쪽이다.
  const rule = functionBodyFrom(cursor, 'positionOutsideObject(sec: number, ppi: number)');
  assert.match(rule, /ppi \+ 1 < paraCount/, '다음 문단 우선');
  assert.match(rule, /getParagraphLength\(sec, ppi - 1\)/, '마지막 문단이면 이전 문단 끝');

  // 선택 기반 조회는 그 규칙에 위임한다 — 복제하면 Delete 키 경로와 착지가 어긋난다.
  const bySelection = functionBodyFrom(cursor, 'positionOutsideSelectedPicture()');
  assert.match(bySelection, /this\.positionOutsideObject\(sec, ppi\)/, '규칙을 위임해야 한다');
  assert.doesNotMatch(bySelection, /getParagraphCount/, '규칙을 복제하면 안 된다');

  // 커서를 실제로 옮기는 쪽도 같은 규칙을 쓴다.
  const move = functionBodyFrom(cursor, 'moveOutOfSelectedPicture()');
  assert.match(move, /this\.positionOutsideSelectedPicture\(\)/, '이동도 같은 규칙');
  assert.match(move, /this\.exitPictureObjectSelection\(\)/, '이동은 선택도 푼다(종전 동작)');
});

test('[#3351] 조회는 커서를 옮기지도 선택을 풀지도 않는다', () => {
  const cursor = src('src/engine/cursor.ts');
  const rule = functionBodyFrom(cursor, 'positionOutsideObject(sec: number, ppi: number)');
  const bySelection = functionBodyFrom(cursor, 'positionOutsideSelectedPicture()');
  for (const [name, body] of [['positionOutsideObject', rule], ['positionOutsideSelectedPicture', bySelection]] as const) {
    assert.doesNotMatch(body, /this\.position\s*=/, `${name} 는 커서를 옮기면 안 된다`);
    assert.doesNotMatch(body, /exitPictureObjectSelection/, `${name} 는 선택을 풀면 안 된다`);
  }
});

test('[#3351] 메뉴 개체 조작은 개체 인접을 기록한다', () => {
  const insert = src('src/command/commands/insert.ts');
  const body = functionBodyFrom(insert, 'function recordObjectMutation');
  assert.match(
    body,
    /getPositionOutsideSelectedPicture\(\)/,
    '개체 선택은 cursor.position 을 옮기지 않는다 — 그냥 읽으면 선택 직전 캐럿이 기록된다',
  );
});

test('[#3351] 클릭 z순서도 개체 인접을 기록한다', () => {
  const mouse = src('src/engine/input-handler-mouse.ts');
  const body = balancedFrom(mouse, 'function bringShapeToFront', '{');
  assert.match(
    body,
    /positionOutsideObject\(picHit\.sec, picHit\.ppi\)/,
    '선택 진입 전이라 ref 로 위치를 구해야 한다',
  );
});

test('[#3351] 인접 문단 계산 — 실제 규칙 동작', async () => {
  const vite = await createServer({
    root: studioRoot,
    appType: 'custom',
    logLevel: 'silent',
    server: { middlewareMode: true },
  });
  try {
    const { CursorState } = await vite.ssrLoadModule('/src/engine/cursor.ts');
    const make = (paraCount: number, lengths: Record<number, number>) => {
      const wasm: any = {
        getParagraphCount: () => paraCount,
        getParagraphLength: (_s: number, p: number) => lengths[p] ?? 0,
      };
      return new CursorState(wasm) as any;
    };

    // 개체가 문단 10, 뒤에 문단이 더 있으면 다음 문단 시작.
    assert.deepEqual(
      make(13, {}).positionOutsideObject(0, 10),
      { sectionIndex: 0, paragraphIndex: 11, charOffset: 0 },
    );
    // 개체가 마지막 문단이면 이전 문단 끝.
    assert.deepEqual(
      make(11, { 9: 7 }).positionOutsideObject(0, 10),
      { sectionIndex: 0, paragraphIndex: 9, charOffset: 7 },
    );
    // 문단이 하나뿐이면 인접이 없다.
    assert.equal(make(1, {}).positionOutsideObject(0, 0), null);
  } finally {
    await vite.close();
  }
});
