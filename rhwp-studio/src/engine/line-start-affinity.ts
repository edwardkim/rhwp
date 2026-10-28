import type { CursorRect, LineInfo } from '@/core/types';

/**
 * 시각 줄 affinity 로 rect 를 다시 조회하는 데 필요한 최소 질의 집합.
 *
 * 두 질의 모두 **실패를 null 로 알린다**. 어느 한쪽이라도 예외를 던지면 호출부의 바깥
 * catch 로 빠져 조합 오버레이가 통째로 사라지는데, 그것은 `exact` 로 물러나는 것보다
 * 나쁜 결과다. 구현부가 wasm 예외를 삼켜 null 로 바꿔서 넘긴다.
 */
export interface LineAffinityLookup {
  /** `charOffset` 이 속한 시각 줄 정보. 조회할 수 없으면 null. */
  lineInfoAt(charOffset: number): LineInfo | null;
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
 * 조합 갱신마다 도는 경로다. `lineInfoAt` 은 매번 한 번 돌지만, 무거운 쪽인
 * `rectAtLineStart` 는 모호한 경계(= offset 이 두 번째 이후 줄의 시작)에서만 부른다.
 */
export function resolveGlyphStartRect(
  charOffset: number,
  exact: CursorRect,
  lookup: LineAffinityLookup,
): CursorRect {
  const line = lookup.lineInfoAt(charOffset);
  if (!line) return exact;
  // 첫 줄은 앞줄이 없어 모호하지 않다.
  if (line.lineIndex <= 0 || charOffset !== line.charStart) return exact;

  const onLine = lookup.rectAtLineStart(line.lineIndex);
  if (!onLine) return exact;

  // `cellBounds`(와 그 파생 `cellOverflowed`)는 그 rect 가 놓인 **쪽의** 셀 bbox 다.
  // getCursorRectOnLine 은 이 둘을 싣지 않으므로 같은 쪽일 때만 exact 것을 이어 쓴다 —
  // 쪽이 바뀌었는데 그대로 들고 가면 오버레이의 셀 클램프(#1951)가 다른 쪽 bbox 로
  // 좌표를 가둔다. 쪽이 다르면 클램프 없이 두는 편이 틀린 곳에 가두는 것보다 낫다.
  const samePage = onLine.pageIndex === exact.pageIndex;
  return {
    ...exact,
    pageIndex: onLine.pageIndex,
    x: onLine.x,
    y: onLine.y,
    height: onLine.height,
    cellBounds: samePage ? exact.cellBounds : undefined,
    cellOverflowed: samePage ? exact.cellOverflowed : undefined,
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
