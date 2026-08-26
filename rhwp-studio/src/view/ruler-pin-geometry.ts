import type { ParaProperties } from '@/core/types';

/**
 * 가로 눈금자 핀의 화면 좌표 ↔ 문서 값 변환, 그리고 각 핀이 무엇을 소유하는지의 판정.
 *
 * 핀을 그리는 식과 드롭 위치를 되돌리는 식을 한 모듈에 둔다 — 둘이 떨어져 있으면 한쪽만
 * 고쳐도 컴파일이 통과하고, 핀이 가리키는 자리와 실제로 바뀌는 값이 어긋난다. 실제로
 * △가 쪽 여백 자리에 그려지면서 ParaShape.marginLeft를 쓰고 있었다(문단 하나만 움직임).
 *
 * 좌표 단위: 화면 x는 눈금자 캔버스 px, 문서 값은 zoom=1 기준 px.
 * DOM에 의존하지 않는다 — 이 규칙은 캔버스 없이 검증된다.
 */

/** 가로 눈금자 핀 — 이름은 각 핀이 커밋하는 필드 그대로 */
export type HPinKind = 'paraIndent' | 'pageMarginLeft' | 'pageMarginRight';

/** 눈금자 핀 드래그가 만들어내는 문서 변경 — 종류에 따라 페이로드가 다르지만 커밋 콜백은 하나다 */
export type RulerPinCommit =
  | { kind: 'paraProps'; props: Partial<ParaProperties> }
  | { kind: 'pageMargin'; pageIdx: number; marginKind: 'top' | 'bottom' | 'left' | 'right'; hwpunit: number };

/** 드롭 위치를 문서 값으로 되돌리는 데 필요한, 마지막 프레임의 기준 좌표 */
export interface HPinDropContext {
  zoom: number;
  /** △가 커밋할 쪽 (구역 해석은 호출부) */
  pageIdx: number;
  /** 기준 쪽의 왼쪽 끝과 표시 너비 (화면 px) */
  pageLeft: number;
  pageDisplayWidth: number;
  /** ▽의 기준 왼쪽 (셀 안이면 셀, 다단이면 현재 단, 아니면 본문) 화면 px */
  refLeft: number;
  /** 커서 문단의 왼쪽 여백 (zoom=1 px) — ▽의 기준점이고, 눈금자는 이 값을 바꾸지 않는다 */
  paraMarginLeftPx: number;
  /** 제본 여백 (zoom=1 px). 본문 왼쪽 경계 = marginLeft + gutter 라서, 경계에서 되돌릴 때 뺀다. */
  pageGutterPx: number;
  /** 맞쪽 제본의 짝수 쪽인가 — 그 쪽은 좌우 여백이 뒤바뀌어 적용된다(model/page.rs). */
  bindingMirrored: boolean;
}

/** px (zoom=1, 96dpi) → HWPUNIT.
 *
 * 가까운 쪽으로 반올림한다. 0 방향 절삭(Rust `px_to_hwpunit`과 같은 방향)을 쓰면 안
 * 움직인 드래그도 값이 1씩 깎인다 — hwpunit→px(getPageInfo)가 딱 떨어지지 않아 되돌린 px가
 * 원값보다 1 ulp 작고, 절삭이 그 ulp를 매번 한 단위로 키운다. 실측: 127.4px 왕복이
 * 9555 → 9554. 저장 단위 하나(1/7200 inch)라 눈엔 안 보이지만 드래그마다 누적된다. */
export function pxToHwpunit(px: number): number {
  const HWPUNIT_PER_INCH = 7200;
  const dpi = 96;
  return Math.round((px * HWPUNIT_PER_INCH) / dpi);
}

/** px (zoom=1, 96dpi) → ParaShape 여백/들여쓰기 저장 단위 (2x HWPUNIT). PageDef 여백과
 * 스케일이 다르다 — para-shape-dialog.ts의 pxToRaw2x(px*150)와 동일 공식. */
export function pxToParaRaw(px: number): number {
  return Math.round(px * 150);
}

/** △ 핀 x — 쪽 여백 안쪽 경계(본문 시작/끝) */
export function pageMarginPinX(
  side: 'left' | 'right',
  pageLeft: number,
  pageDisplayWidth: number,
  pageMarginPx: number,
  zoom: number,
): number {
  return side === 'left'
    ? pageLeft + pageMarginPx * zoom
    : pageLeft + pageDisplayWidth - pageMarginPx * zoom;
}

/** ▽ 핀 x — 첫 줄 시작 위치 = marginLeft + max(indent, 0). 내어쓰기(indent<0)는 첫 줄이
 * 아니라 둘째 줄부터를 밀어내므로 첫 줄 핀은 marginLeft 자리에 그대로 있다
 * (renderer/layout/paragraph_layout.rs). */
export function paraIndentPinX(
  refLeft: number,
  paraMarginLeftPx: number,
  paraIndentPx: number,
  zoom: number,
): number {
  return refLeft + (paraMarginLeftPx + Math.max(paraIndentPx, 0)) * zoom;
}

/**
 * 가로 핀 드롭 → 문서 변경. 여백은 쪽(PageDef) 소유, 들여쓰기는 문단(ParaShape) 소유 —
 * 그 소유 규칙의 유일한 판정 지점이다. 위 pinX 식들의 역함수이기도 하다.
 */
export function horizontalPinCommit(
  kind: HPinKind,
  dropX: number,
  ctx: HPinDropContext,
): RulerPinCommit {
  if (kind === 'pageMarginLeft' || kind === 'pageMarginRight') {
    // 세로 눈금자의 쪽 위/아래 여백 핀과 같은 소유·같은 커밋 경로(PageDef).
    //
    // 핀은 본문 상자 경계에 서 있고, 그 경계는 PageDef 값과 두 가지로 다르다:
    //   본문 왼쪽 = marginLeft + gutter,  맞쪽 제본 짝수 쪽 = 좌우가 뒤바뀜 (model/page.rs)
    // 그래서 화면 쪽(왼쪽/오른쪽)과 실제로 쓸 필드가 갈린다. 뒤바뀐 쪽에서 왼쪽 핀을
    // 끌면 marginRight 를, 오른쪽 핀을 끌면 marginLeft 를 바꿔야 한다.
    const draggedLeftEdge = kind === 'pageMarginLeft';
    const edgeInsetPx = draggedLeftEdge
      ? (dropX - ctx.pageLeft) / ctx.zoom
      : (ctx.pageLeft + ctx.pageDisplayWidth - dropX) / ctx.zoom;
    // 제본 여백은 "본문 시작 쪽" 경계에만 더해져 있다. 뒤바뀐 쪽에서는 그 시작이 오른쪽이다.
    const gutterOnThisEdge = draggedLeftEdge !== ctx.bindingMirrored ? ctx.pageGutterPx : 0;
    // 0 이상으로 clamp — 음수 쪽 여백은 본문 영역을 종이 밖으로 밀어낸다.
    const marginPx = Math.max(0, edgeInsetPx - gutterOnThisEdge);
    const marginKind: 'left' | 'right' = draggedLeftEdge !== ctx.bindingMirrored ? 'left' : 'right';
    return {
      kind: 'pageMargin',
      pageIdx: ctx.pageIdx,
      marginKind,
      hwpunit: pxToHwpunit(marginPx),
    };
  }

  // ▽는 첫 줄 시작 위치 — 문단 왼쪽 여백을 기준으로 한 상대값(indent)만 바꾼다. 여백
  // 자체는 쪽 소유라 여기서 건드리지 않는다.
  // 0 이상으로 clamp — 첫 줄은 marginLeft 왼쪽으로 갈 수 없다. indent<0(내어쓰기)은
  // 첫 줄이 아니라 둘째 줄부터를 밀어내므로 첫 줄 핀으로는 만들 수 없다.
  const indentPx = Math.max(0, (dropX - ctx.refLeft) / ctx.zoom - ctx.paraMarginLeftPx);
  return { kind: 'paraProps', props: { indent: pxToParaRaw(indentPx) } };
}
