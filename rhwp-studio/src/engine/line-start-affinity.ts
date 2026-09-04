import type { CursorRect, LineInfo } from '@/core/types';

/** 시각 줄 affinity 로 rect 를 다시 조회하는 데 필요한 최소 질의 집합. */
export interface LineAffinityLookup {
  /** `charOffset` 이 속한 시각 줄 정보. */
  lineInfoAt(charOffset: number): LineInfo;
  /** `lineIndex` 줄의 시작 rect. 조회할 수 없으면 null. */
  rectAtLineStart(lineIndex: number): CursorRect | null;
}

/**
 * `charOffset` 에서 시작하는 **글자가 실제로 그려지는 자리**의 rect 를 돌려준다.
 *
 * soft-wrap 줄 경계 offset 은 "이전 줄 끝"과 "다음 줄 시작"을 동시에 뜻한다(#785).
 * 줄 affinity 인자가 없는 `getCursorRect` 는 render tree 를 시각 순서로 훑다 이전 줄의
 * TextRun 에 먼저 매치돼 그 줄 끝을 돌려준다. 캐럿에는 그 affinity 가 맞지만, 그 offset 의
 * 글자를 덮어 그리는 오버레이에는 한 줄 위를 가리키는 값이 된다(#6553).
 *
 * 모호한 경계(= offset 이 두 번째 이후 줄의 시작)일 때만 시각 줄을 명시해 다시 조회하고,
 * 그 밖에는 `exact` 를 그대로 돌려준다 — 조합 갱신마다 도는 경로라 불필요한 질의를 늘리지
 * 않는다.
 */
export function resolveGlyphStartRect(
  charOffset: number,
  exact: CursorRect,
  lookup: LineAffinityLookup,
): CursorRect {
  const line = lookup.lineInfoAt(charOffset);
  // 첫 줄은 앞줄이 없어 모호하지 않다.
  if (line.lineIndex <= 0 || charOffset !== line.charStart) return exact;

  const onLine = lookup.rectAtLineStart(line.lineIndex);
  if (!onLine) return exact;

  // getCursorRectOnLine 은 셀 bbox 를 싣지 않는다 — 오버레이의 셀 클램프(#1951)가 쓰는
  // cellBounds 는 exact 쪽 값을 유지한다.
  return {
    ...exact,
    pageIndex: onLine.pageIndex,
    x: onLine.x,
    y: onLine.y,
    height: onLine.height,
  };
}

/**
 * 조합 오버레이의 시작 rect 와 캐럿이 **한 줄 안에** 있는지 — 즉 단일 사각형으로 그릴 수
 * 있는지 판정한다.
 *
 * y 비교는 쓸 수 없다. 같은 줄이라도 글꼴 크기가 섞이면 캐럿 y 가 run 마다 다르다(캐럿 y 는
 * baseline 기준으로 잡힌다). 대신 한 줄 안에서 반드시 성립해야 하는 관계를 본다 — 같은 쪽이고,
 * 시작이 캐럿보다 오른쪽에 있지 않아야 한다.
 *
 * [Issue #6738] 줄 affinity 를 물을 수 없는 문맥(머리말/꼬리말·각주·2단계 이상 중첩 셀)에서는
 * 조합 글자가 줄을 넘어가도 시작 좌표를 바로잡을 수 없다. 그 상태로 그리면 폭이 음수가 되고
 * `clampCompositionBox` 의 `height * 0.6` 폴백에 삼켜져 이전 줄에 그럴듯한 박스가 남는다.
 */
export function isCompositionBoxRepresentable(start: CursorRect, caret: CursorRect): boolean {
  return start.pageIndex === caret.pageIndex && start.x <= caret.x;
}
