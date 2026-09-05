// [Issue #6553] IME 조합 중인 글자가 soft-wrap 으로 다음 줄로 넘어가면, 조합 오버레이가
// 넘어가기 전 위치인 이전 줄 끝에 그려져 같은 글자가 두 곳에 보였다.
//
// 좌표는 devel 에서 실측한 값이다 — samples/143E433F503322BD33.hwp 구역 0 / 문단 1,
// wrap 경계 offset 22:
//   getCursorRect(0, 1, 22) = { x: 394.0, y: 125.8 }   (이전 줄 끝 — 줄 affinity 없는 exact 조회)
//   getCursorRect(0, 1, 23) = { x: 134.9, y: 147.1 }   (조합 중 캐럿)
//   getCursorRectOnLine(0, 1, 1, at_end=false) = { x: 121.6, y: 146.5 }  (글자가 놓인 줄의 시작)
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import { resolveGlyphStartRect } from '../src/engine/line-start-affinity.ts';
import { codeOnly, functionBodyFrom } from './support/source-guard.ts';
import type { CursorRect, LineInfo } from '../src/core/types.ts';

/** 이전 줄 끝 — 줄 affinity 없는 exact 조회 결과. */
const EXACT_PREV_LINE_END: CursorRect = { pageIndex: 0, x: 394.0, y: 125.8, height: 21.3 };
/** 조합 중 캐럿 — 글자가 실제로 놓인 다음 줄. */
const CARET_ON_NEXT_LINE: CursorRect = { pageIndex: 0, x: 134.9, y: 147.1, height: 21.3 };
/** 글자가 놓인 줄의 시작. */
const NEXT_LINE_START: CursorRect = { pageIndex: 0, x: 121.6, y: 146.5, height: 21.3 };

/** offset 22 가 두 번째 줄(lineIndex 1)의 시작인 문단. */
const WRAP_BOUNDARY_LINE: LineInfo = { lineIndex: 1, lineCount: 2, charStart: 22, charEnd: 45 };

function lookup(
  line: LineInfo,
  onLine: CursorRect | null,
): { calls: { lineInfoAt: number[]; rectAtLineStart: number[] } } & Parameters<typeof resolveGlyphStartRect>[2] {
  const calls = { lineInfoAt: [] as number[], rectAtLineStart: [] as number[] };
  return {
    calls,
    lineInfoAt(charOffset: number) {
      calls.lineInfoAt.push(charOffset);
      return line;
    },
    rectAtLineStart(lineIndex: number) {
      calls.rectAtLineStart.push(lineIndex);
      return onLine;
    },
  };
}

test('soft-wrap 경계 offset 은 글자가 놓인 줄의 시작으로 해석된다', () => {
  const deps = lookup(WRAP_BOUNDARY_LINE, NEXT_LINE_START);
  const resolved = resolveGlyphStartRect(22, EXACT_PREV_LINE_END, deps);

  assert.deepEqual(deps.calls.lineInfoAt, [22]);
  assert.deepEqual(deps.calls.rectAtLineStart, [1], '모호한 경계에서는 시각 줄을 명시해 다시 조회한다');
  assert.equal(resolved.x, NEXT_LINE_START.x);
  assert.equal(resolved.y, NEXT_LINE_START.y);
  assert.equal(resolved.pageIndex, NEXT_LINE_START.pageIndex);
  assert.notEqual(resolved.y, EXACT_PREV_LINE_END.y, '이전 줄에 남으면 안 된다');
});

test('경계에서 조합 오버레이 폭이 음수에서 실제 글자 폭으로 바뀐다', () => {
  // caret-renderer 의 clampCompositionBox 는 음수 폭을 Math.max(charWidth, height*0.6) 로
  // 조용히 삼켜, 틀린 줄에 그럴듯한 크기의 박스를 그렸다.
  const before = CARET_ON_NEXT_LINE.x - EXACT_PREV_LINE_END.x;
  assert.ok(before < 0, `수정 전 charWidth 는 음수였다: ${before}`);

  const resolved = resolveGlyphStartRect(22, EXACT_PREV_LINE_END, lookup(WRAP_BOUNDARY_LINE, NEXT_LINE_START));
  const after = CARET_ON_NEXT_LINE.x - resolved.x;
  assert.ok(after > 0, `수정 후 charWidth 는 양수여야 한다: ${after}`);
  assert.ok(after < CARET_ON_NEXT_LINE.height, `한 글자 폭이어야 한다: ${after}`);
});

test('줄 중간 offset 은 exact 를 그대로 쓰고 줄 조회를 추가하지 않는다', () => {
  const deps = lookup({ lineIndex: 1, lineCount: 2, charStart: 22, charEnd: 45 }, NEXT_LINE_START);
  const resolved = resolveGlyphStartRect(30, EXACT_PREV_LINE_END, deps);

  assert.deepEqual(resolved, EXACT_PREV_LINE_END);
  assert.deepEqual(deps.calls.rectAtLineStart, [], '모호하지 않으면 추가 질의를 하지 않는다');
});

test('첫 줄 시작은 앞줄이 없어 추가 질의 없이 exact 를 쓴다', () => {
  const deps = lookup({ lineIndex: 0, lineCount: 2, charStart: 0, charEnd: 22 }, NEXT_LINE_START);
  const resolved = resolveGlyphStartRect(0, EXACT_PREV_LINE_END, deps);

  assert.deepEqual(resolved, EXACT_PREV_LINE_END);
  assert.deepEqual(deps.calls.rectAtLineStart, []);
});

test('줄 시작 rect 를 조회할 수 없으면 exact 동작을 유지한다', () => {
  const resolved = resolveGlyphStartRect(22, EXACT_PREV_LINE_END, lookup(WRAP_BOUNDARY_LINE, null));
  assert.deepEqual(resolved, EXACT_PREV_LINE_END);
});

test('셀 오버레이 클램프용 cellBounds 는 줄 재조회 뒤에도 보존된다', () => {
  // getCursorRectOnLine 은 cellBounds 를 싣지 않는다. 잃어버리면 #1951 의 셀 밖 이탈이 되살아난다.
  const exactInCell: CursorRect = {
    ...EXACT_PREV_LINE_END,
    cellBounds: { x: 100, y: 120, w: 300, h: 60 },
  };
  const resolved = resolveGlyphStartRect(22, exactInCell, lookup(WRAP_BOUNDARY_LINE, NEXT_LINE_START));

  assert.deepEqual(resolved.cellBounds, exactInCell.cellBounds);
  assert.equal(resolved.x, NEXT_LINE_START.x);
});

test('updateCaret 의 조합 분기가 폭 계산 전에 줄 affinity 를 적용한다', () => {
  const source = codeOnly(readFileSync(new URL('../src/engine/input-handler.ts', import.meta.url), 'utf8'));
  const updateCaret = functionBodyFrom(source, 'private updateCaret(');

  const applied = updateCaret.indexOf('startRect = this.compositionOverlayStartRect(anchor, startRect);');
  const width = updateCaret.indexOf('const charWidth = rect.x - startRect.x;');
  assert.ok(applied >= 0, '조합 오버레이 원점이 줄 affinity 를 거쳐야 한다');
  assert.ok(width >= 0);
  assert.ok(applied < width, '폭 계산 전에 원점을 확정해야 한다');

  const resolver = functionBodyFrom(source, 'private compositionOverlayStartRect(');
  assert.match(resolver, /resolveGlyphStartRect\(anchor\.charOffset, exact,/);
  assert.match(
    resolver,
    /if \(this\.cursor\.isInHeaderFooter\(\) \|\| this\.cursor\.isInFootnote\(\)\) return exact;/,
    '머리말・꼬리말·각주는 getCursorRectOnLine 대상이 아니라 exact 를 유지한다',
  );
  assert.match(
    resolver,
    /if \(\(anchor\.cellPath\?\.length \?\? 0\) > 1\) return exact;/,
    '2단 이상 중첩 셀은 getCursorRectOnLine 이 문단을 지목할 수 없어 exact 를 유지한다',
  );
  assert.match(resolver, /this\.wasm\.getCursorRectOnLine\(/);
});
